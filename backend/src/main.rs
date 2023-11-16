#[macro_use]
extern crate rocket;

use chrono::{DateTime, Utc};
use config::{Config, File as ConfigFile, FileFormat};
use dashmap::DashMap;
use futures_channel::mpsc::{channel, Sender};
use futures_concurrency::prelude::*;
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::fs::{FileServer, NamedFile};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::{Method, Status};
use rocket::response::content;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::{Error, Uuid};
// use std::sync::mpsc::{Sender, channel};
use lazy_static::lazy_static;
use multimap::MultiMap;
use nanoid::nanoid;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::{Host, Origin};
use rocket::outcome::Outcome::Success;
use rocket::request::FromParam;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Data, Request, State};
use ws::Message;

mod auth;
mod auth_db;
use auth::{Auth, AuthMap};

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

type ThingMap = DashMap<Uuid, Vec<Sender<WsMessage>>>;

#[get("/send/<id>", data = "<input>")]
async fn get(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[put("/send/<id>", data = "<input>")]
async fn put(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[post("/send/<id>", data = "<input>")]
async fn post(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[delete("/send/<id>", data = "<input>")]
async fn delete(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[head("/send/<id>", data = "<input>")]
async fn head(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[options("/send/<id>", data = "<input>")]
async fn options(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[patch("/send/<id>", data = "<input>")]
async fn patch(id: ID, _a: Auth, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[derive(Clone, Serialize)]
struct RegisterResult {
    id: Uuid,
    auth: String,
}

impl RegisterResult {
    fn new() -> RegisterResult {
        RegisterResult {
            id: Uuid::new_v4(),
            auth: nanoid!(),
        }
    }
}

#[post("/register")]
async fn register(auth_map: &State<AuthMap>) -> Result<content::RawJson<String>, Status> {
    let res = RegisterResult::new();
    if auth_map.contains_id(res.id) {
        Err(Status::Unauthorized)
    } else {
        auth_map.add(res.id, res.auth.clone());
        match serde_json::to_string(&res) {
            Ok(s) => Ok(content::RawJson(s)),
            Err(_) => Err(Status::Unauthorized),
        }
    }
}

// TODO clear out sockets and auths after some time

#[head("/validate/<id>")]
async fn validate(id: ID, auth: Auth, auth_map: &State<AuthMap>) -> Result<(), Status> {
    if let Some(existing) = auth_map.clone_token(id.0) {
        if auth.0.ne(&existing) {
            Err(Status::Unauthorized)
        } else {
            Ok(())
        }
    } else {
        Err(Status::NotFound)
    }
}

async fn handle(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    let mut has_sent = false;
    if let Some(mut senders) = map.get_mut(&id.0) {
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
    map.remove(&id.0);
    Status::NotFound
}

enum MyMessage {
    In(ws::result::Result<Message>),
    Out(WsMessage),
}

#[get("/connect/<id>")]
fn websocket<'r>(
    id: ID,
    ws: ws::WebSocket,
    map: &State<ThingMap>,
    auth_map: &'r rocket::State<AuthMap>,
) -> ws::Stream!['r] {
    if !map.contains_key(&id.0) {
        map.insert(id.0, vec![]);
    }
    let (sender, receiver) = channel(8);
    map.alter(&id.0, move |_, mut v| {
        v.push(sender);
        v
    });
    ws::Stream! { ws =>
        let mut authenticated = false;

        let w = ws.map(MyMessage::In);
        let re = receiver.map(MyMessage::Out);
        for await message in (re, w).merge() {
            match message {
                MyMessage::In(msg) => {
                    if let Ok(msg) = msg {
                        if !authenticated {
                            let auth_result = handle_ws_auth(id.clone(), msg, auth_map);
                            if !auth_result {
                                yield serde_json::to_string(&InOutMessage::AuthResponse{ ok: false }).unwrap_or("AAA".to_string()).into();
                                break;
                            }
                            authenticated = true;
                            yield serde_json::to_string(&InOutMessage::AuthResponse{ ok: true }).unwrap_or("ERROR".to_string()).into();
                        } else {
                            match msg {
                                Message::Close(_) => {
                                    println!("Websocket closed");
                                    break;
                                },
                                _ => println!("Websocket Message: {:?}", msg)
                            }
                        }
                    } else {
                        println!("Error in WS Reception");
                        break;
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
    }
}

fn handle_ws_auth(id: ID, msg: Message, auth_map: &State<AuthMap>) -> bool {
    if let Message::Text(txt) = msg {
        match serde_json::from_str(&txt) {
            Ok(des) => {
                if let Some(InOutMessage::Auth { token: auth }) = des {
                    if let Some(t) = auth_map.clone_token(id.0) {
                        return t.eq(&auth);
                    }
                }
            }
            Err(e) => {
                eprintln!("Could not deserialize: {e}");
            }
        }
    }
    false
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
    let r = auth::setup_rocket(r);
    auth_db::attach_db(r)
}
