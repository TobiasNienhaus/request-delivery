#[macro_use]
extern crate rocket;

use chrono::NaiveDateTime;
use dashmap::DashMap;
use futures_channel::mpsc::{channel, Sender};
use futures_concurrency::prelude::*;
use rocket::fairing::AdHoc;
use rocket::fs::{FileServer, NamedFile};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use std::path::PathBuf;
use std::sync::OnceLock;
use ws::frame::{CloseCode, CloseFrame};

use lazy_static::lazy_static;
use rocket::request::FromParam;
use rocket::{Request, State};
use shared::{read_config, Config};
use uuid::{Error, Uuid};
use ws::Message;

mod auth;
use auth::{Auth, AuthService, NewAuthService};
mod request_data;
use request_data::RequestData;
mod cleanup;

static AUTH_HEADER: &'static str = "X-Auth";

static CONFIG_PATH: OnceLock<String> = OnceLock::new();
// static CONFIG: OnceLock<Config> = OnceLock::new();

lazy_static! {
    static ref CONFIG: Config =
        read_config(CONFIG_PATH.get().unwrap()).expect("Could not parse config");
    static ref ANGULAR_INDEX: PathBuf = CONFIG.ui_path().join("index.html");
    static ref MY_EPOCH: NaiveDateTime = CONFIG.get_epoch();
    static ref CLEANUP_TOKEN: String = nanoid::nanoid!(64);
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

#[derive(Clone)]
pub enum WsMessage {
    Shutdown,
    ServerShutdown,
    TokenExpired,
    Request(RequestData),
}

unsafe impl Send for WsMessage {}
unsafe impl Sync for WsMessage {}

type ThingMap = DashMap<String, Vec<Sender<WsMessage>>>;

#[catch(default)]
fn default_catcher(_: Status, _: &Request) {}

#[get("/send/<id>", data = "<input>")]
async fn get(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[put("/send/<id>", data = "<input>")]
async fn put(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[post("/send/<id>", data = "<input>")]
async fn post(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[delete("/send/<id>", data = "<input>")]
async fn delete(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[head("/send/<id>", data = "<input>")]
async fn head(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[options("/send/<id>", data = "<input>")]
async fn options(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[patch("/send/<id>", data = "<input>")]
async fn patch(id: &str, auth: AuthService, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, auth, map, input).await
}

#[post("/register/random")]
async fn register_random(auth_service: NewAuthService) -> Result<(Status, Json<Auth>), Status> {
    match auth_service.save_random().await {
        Ok(auth) => Ok((Status::Ok, Json(auth))),
        Err(s) => Err(s),
    }
}

#[post("/register", format = "json", data = "<auth>")]
async fn register(
    auth: Json<Auth>,
    auth_service: NewAuthService,
) -> Result<(Status, Json<Auth>), Status> {
    match auth_service.save(auth.0).await {
        Ok(auth) => Ok((Status::Ok, Json(auth))),
        Err(s) => Err(s),
    }
}

// TODO clear out sockets and auths after some time

#[head("/validate/<id>")]
async fn validate(id: &str, mut auth: AuthService) -> Result<(), Status> {
    auth.check(id).await
}

async fn handle(
    id: &str,
    mut auth: AuthService,
    map: &State<ThingMap>,
    input: RequestData,
) -> Status {
    if let Err(s) = auth.check(id).await {
        return s;
    }
    let mut has_sent = false;
    if let Some(mut senders) = map.get_mut(id) {
        println!("Found senders");
        for sender in senders.value() {
            println!("Trying send");
            if !sender.is_closed() {
                println!("Sending");
                has_sent = true;
                if let Err(e) = sender.clone().send(WsMessage::Request(input.clone())).await {
                    eprintln!("Send Error (closing channel): {}", e);
                    sender.clone().close_channel();
                };
            }
        }

        senders.value_mut().retain(|sender| !sender.is_closed());
    }
    if !has_sent {
        println!("Removing, as nothing has been sent");
        map.remove(id);
    }
    Status::Accepted
}

enum MyMessage {
    In(ws::result::Result<Message>),
    Out(WsMessage),
}

#[get("/connect/<id>")]
fn websocket<'r>(
    id: &'r str,
    mut auth: AuthService<true>,
    ws: ws::WebSocket,
    map: &'r State<ThingMap>,
) -> ws::Stream!['r] {
    let (sender, receiver) = channel(8);

    ws::Stream! { ws =>
        if auth.check_bool(id).await {
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
                            },
                            WsMessage::ServerShutdown => {
                                yield Message::Close(Some(CloseFrame {
                                    code: CloseCode::Away,
                                    reason: std::borrow::Cow::Borrowed("SERVER")
                                }))
                            },
                            WsMessage::TokenExpired => {
                                yield Message::Close(Some(CloseFrame {
                                    code: CloseCode::Library(4001),
                                    reason: std::borrow::Cow::Borrowed("SERVER")
                                }))
                            },
                            WsMessage::Request(req) => yield serde_json::to_string(&req).unwrap_or("ERROR".to_string()).into()
                        }
                    }
                }
            }
        } else {
            yield "Unauthorized".into();
        }
    }
}

#[get("/<_..>")]
async fn ui() -> NamedFile {
    println!("Request");
    NamedFile::open(ANGULAR_INDEX.as_path()).await.unwrap()
}

// #[derive(Deserialize)]
// struct Config {
//     ui_path: PathBuf,
//     my_epoch: String,
// }

#[launch]
fn rocket() -> _ {
    let r = rocket::build()
        .manage(ThingMap::new())
        .mount(
            "/",
            routes![
                get,
                put,
                post,
                delete,
                head,
                options,
                patch,
                websocket,
                register,
                validate,
                register_random
            ],
        )
        .mount("/admin", routes![cleanup::cleanup_tokens])
        .register("/", catchers![default_catcher])
        .mount("/ui", routes![ui]);

    let f = r.figment();
    let config_path = f
        .find_value("req.config_path")
        .expect("Missing Config Path!");
    let config_path = config_path
        .into_string()
        .expect("Config Path is no string of mine");
    CONFIG_PATH.get_or_init(move || config_path);

    let r = r.mount("/ui", FileServer::from(CONFIG.ui_path()).rank(-5));

    cleanup::init();

    auth::attach_db(r).attach(AdHoc::on_shutdown("Close Websockets", |r| {
        Box::pin(async move {
            if let Some(thingies) = r.state::<ThingMap>() {
                for mut i in thingies.iter_mut() {
                    let key = i.key().clone();
                    for s in i.value_mut() {
                        if !s.is_closed() {
                            println!("Shutting down {key}");
                            if let Err(e) = s.send(WsMessage::ServerShutdown).await {
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
