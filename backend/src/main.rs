#[macro_use]
extern crate rocket;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures_channel::mpsc::{channel, Sender};
use futures_concurrency::prelude::*;
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::{Method, Status};
use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;
use uuid::{Error, Uuid};
// use std::sync::mpsc::{Sender, channel};
use multimap::MultiMap;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::{Host, Origin};
use rocket::outcome::Outcome::Success;
use rocket::request::{FromParam, FromRequest};
use rocket::serde::Serialize;
use rocket::{Data, Request, State};
use ws::Message;

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

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
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

        let body = futures::executor::block_on(data.open(512.kibibytes()).into_string()).ok();
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

type ThingMap = DashMap<Uuid, Vec<Sender<RequestData>>>;

#[get("/send/<id>", data = "<input>")]
async fn get(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[put("/send/<id>", data = "<input>")]
async fn put(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[post("/send/<id>", data = "<input>")]
async fn post(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[delete("/send/<id>", data = "<input>")]
async fn delete(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[head("/send/<id>", data = "<input>")]
async fn head(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[options("/send/<id>", data = "<input>")]
async fn options(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

#[patch("/send/<id>", data = "<input>")]
async fn patch(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    handle(id, map, input).await
}

async fn handle(id: ID, map: &State<ThingMap>, input: RequestData) -> Status {
    if let Some(senders) = map.get_mut(&id.0) {
        for sender in senders.value() {
            sender.clone().send(input.clone()).await;
        }
        Status::Accepted
    } else {
        Status::NotFound
    }
}

enum MyMessage {
    In(ws::result::Result<Message>),
    Out(RequestData),
}

#[get("/connect/<id>")]
fn echo_stream(id: ID, ws: ws::WebSocket, map: &State<ThingMap>) -> ws::Stream!['static] {
    if !map.contains_key(&id.0) {
        map.insert(id.0, vec![]);
    }
    let (sender, receiver) = channel(8);
    map.alter(&id.0, move |_, mut v| {
        v.push(sender);
        v
    });
    ws::Stream! { ws =>
        let re = receiver.map(MyMessage::Out);
        let w = ws.map(MyMessage::In);
        for await message in (re, w).merge() {
            match message {
                MyMessage::In(_) => {
                    println!("IN!!!")
                },
                MyMessage::Out(rd) => yield serde_json::to_string(&rd).unwrap_or("ERROR".to_string()).into()
            }
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(ThingMap::new()).mount(
        "/",
        routes![get, put, post, delete, head, options, patch, echo_stream],
    )
}
