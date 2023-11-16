#[macro_use]
extern crate rocket;

use chrono::{DateTime, Utc};
use config::{Config, File as ConfigFile, FileFormat};
use dashmap::DashMap;
use futures_channel::mpsc::{channel, Sender};
use futures_concurrency::prelude::*;
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::fairing::AdHoc;
use rocket::fs::{FileServer, NamedFile};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::{Method, Status};
use rocket::serde::json::Json;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::{Error, Uuid};
// use std::sync::mpsc::{Sender, channel};
use lazy_static::lazy_static;
use multimap::MultiMap;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::{Host, Origin};
use rocket::outcome::Outcome::Success;
use rocket::request::FromParam;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Data, Request, State};
use ws::Message;

mod auth_db;
use auth_db::{AuthGuardDb, NewAuth};

lazy_static! {
    static ref CONFIG: Config = load_config().unwrap();
    static ref UI_PATH: String = CONFIG.get("ui.path").unwrap();
    static ref ANGULAR_INDEX: PathBuf = Path::new(UI_PATH.as_str()).join("index.html");
}

#[derive(Clone, Copy)]
pub struct ID(Uuid);

impl<'a> FromParam<'a> for ID {
    type Error = Error;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(param) {
            Ok(id) => Ok(ID(id)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestData {
    method: Method,
    content_type: Option<String>,
    body: Option<String>,
    complete: Option<bool>,
    headers: MultiMap<String, String>,
    cookies: MultiMap<String, String>,
    uri: Origin<'static>,
    remote: RemoteInfo,
    // accepts: Option<> // TODO
    time: String,
}

#[derive(Clone)]
enum WsMessage {
    Shutdown,
    Formatted(InOutMessage),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum InOutMessage {
    Auth { token: String },
    AuthResponse { ok: bool },
    Request(RequestData),
}

unsafe impl Send for WsMessage {}
unsafe impl Sync for WsMessage {}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    host: Option<Host<'static>>,
    remote_ip: Option<SocketAddr>,
    header_ip: Option<IpAddr>,
    client_ip: Option<IpAddr>,
}

unsafe impl Send for RequestData {}
unsafe impl Sync for RequestData {}

fn current_iso() -> String {
    let now: DateTime<Utc> = SystemTime::now().into();
    now.to_rfc3339()
}

#[rocket::async_trait]
impl<'r> FromData<'r> for RequestData {
    type Error = (); // TODO

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let time = current_iso();
        let mut headers = MultiMap::new();

        for header in req.headers().iter() {
            headers.insert(header.name.to_string(), header.value.to_string());
        }

        let body = futures::executor::block_on(data.open(16.kibibytes()).into_string()).ok();
        let mut complete = None;

        if let Some(ref b) = body {
            complete = Some(b.is_complete());
        }

        let mut cookies = MultiMap::new();

        for c in req.cookies().iter() {
            cookies.insert(c.name().to_string(), c.value().to_string());
        }

        Success(RequestData {
            method: req.method(),
            content_type: req
                .content_type()
                .map(|mt| format!("{}/{}", mt.0.top(), mt.0.sub())),
            body: body.map(|b| b.value),
            complete,
            headers,
            cookies,
            uri: req.uri().clone().into_owned(),
            remote: RemoteInfo {
                host: req.host().cloned().into_owned(),
                remote_ip: req.remote(),
                header_ip: req.real_ip(),
                client_ip: req.client_ip(),
            },
            time,
        })
    }
}

type ThingMap = DashMap<String, Vec<Sender<WsMessage>>>;

#[get("/send/<id>", data = "<input>")]
async fn get(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[put("/send/<id>", data = "<input>")]
async fn put(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[post("/send/<id>", data = "<input>")]
async fn post(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[delete("/send/<id>", data = "<input>")]
async fn delete(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[head("/send/<id>", data = "<input>")]
async fn head(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[options("/send/<id>", data = "<input>")]
async fn options(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[patch("/send/<id>", data = "<input>")]
async fn patch(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[post("/register")]
async fn register(new_auth: NewAuth) -> (Status, Json<NewAuth>) {
    (Status::Created, Json(new_auth))
}

// TODO clear out sockets and auths after some time

#[head("/validate/<id>")]
async fn validate(id: &str, auth: AuthGuardDb) -> Result<(), Status> {
    if !auth.id_matches(id) {
        return Err(Status::Unauthorized);
    }
    Ok(())
}

async fn handle(id: &str, auth: AuthGuardDb, map: &State<ThingMap>, input: RequestData) -> Status {
    if !auth.id_matches(&id) {
        return Status::Unauthorized;
    }
    let mut has_sent = false;
    if let Some(mut senders) = map.get_mut(id) {
        for sender in senders.value() {
            if !sender.is_closed() {
                has_sent = true;
                if let Err(e) = sender
                    .clone()
                    .send(WsMessage::Formatted(InOutMessage::Request(input.clone())))
                    .await
                {
                    eprintln!("Send Error (closing channel): {}", e);
                    sender.clone().close_channel();
                };
            }
        }

        senders.value_mut().retain(|sender| sender.is_closed());
    } else {
        return Status::NotFound;
    }
    if has_sent {
        return Status::Accepted;
    }
    println!("Removing, as nothing has been sent");
    map.remove(id);
    Status::NotFound
}

enum MyMessage {
    In(ws::result::Result<Message>),
    Out(WsMessage),
}

#[get("/connect/<id>")]
fn websocket<'r>(
    id: &'r str,
    auth: AuthGuardDb,
    ws: ws::WebSocket,
    map: &'r State<ThingMap>,
) -> ws::Stream!['r] {
    let (sender, receiver) = channel(8);

    ws::Stream! { ws =>
        if auth.id_matches(id) {
            if !map.contains_key(id) {
                map.insert(id.to_owned(), vec![]);
            }
            map.alter(id, |_, mut v| {
                v.push(sender.clone());
                v
            });

            let w = ws.map(MyMessage::In);
            let re = receiver.map(MyMessage::Out);
            for await message in (re, w).merge() {
                match message {
                    MyMessage::In(msg) => {
                        match msg {
                            Ok(msg) => {
                                match msg {
                                    Message::Close(_) => {
                                        println!("Websocket closed");
                                        break;
                                    },
                                    _ => println!("Websocket Message: {:?}", msg)
                                }
                            },
                            Err(e) => {
                                eprintln!("Error in WS Reception: {e}");
                                break;
                            }
                        }
                    },
                    MyMessage::Out(rd) => {
                        match rd {
                            WsMessage::Shutdown => {
                                yield Message::Close(None);
                                break;
                            }
                            WsMessage::Formatted(msg) => yield serde_json::to_string(&msg).unwrap_or("ERROR".to_string()).into()
                        }
                    }
                }
            }
        } else {
            yield "Unauthorized".into();
        }
    }
}

fn load_config() -> Result<Config, ()> {
    let builder = Config::builder().add_source(ConfigFile::new("config.ini", FileFormat::Ini));

    match builder.build() {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("Could not load config! {}", e);
            Err(())
        }
    }
}

#[get("/<_..>")]
async fn ui() -> NamedFile {
    println!("Request");
    NamedFile::open(ANGULAR_INDEX.as_path()).await.unwrap()
}

#[launch]
fn rocket() -> _ {
    let r = rocket::build()
        .manage(ThingMap::new())
        .mount(
            "/",
            routes![get, put, post, delete, head, options, patch, websocket, register, validate],
        )
        .mount("/ui", FileServer::from(UI_PATH.as_str()).rank(-5))
        .mount("/ui", routes![ui]);
    auth_db::attach_db(r).attach(AdHoc::on_shutdown("Close Websockets", |r| {
        Box::pin(async move {
            if let Some(thingies) = r.state::<ThingMap>() {
                for mut i in thingies.iter_mut() {
                    let key = i.key().clone();
                    for s in i.value_mut() {
                        if !s.is_closed() {
                            if let Err(e) = s.send(WsMessage::Shutdown).await {
                                eprintln!("Could not shut down {key}: {e}")
                            } else {
                                println!("Shutdown for {key} successful")
                            }
                        }
                    }
                }
            }
        })
    }))
}
