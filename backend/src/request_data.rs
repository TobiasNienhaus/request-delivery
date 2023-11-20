use std::{
    net::{IpAddr, SocketAddr},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use multimap::MultiMap;
use rocket::{
    data::Outcome,
    data::{FromData, ToByteUnit},
    http::{
        ext::IntoOwned,
        uri::{Host, Origin},
        Method,
    },
    serde, Data, Request,
};
use serde::{Deserialize, Serialize};

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

unsafe impl Send for RequestData {}
unsafe impl Sync for RequestData {}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    host: Option<Host<'static>>,
    remote_ip: Option<SocketAddr>,
    header_ip: Option<IpAddr>,
    client_ip: Option<IpAddr>,
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

        Outcome::Success(RequestData {
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

fn current_iso() -> String {
    let now: DateTime<Utc> = SystemTime::now().into();
    now.to_rfc3339()
}
