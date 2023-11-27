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

use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestData {
    method: Method,
    content_type: Option<String>,
    body: Option<Body>,
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
pub struct Body {
    raw: String,
    base64: String,
}

impl Body {
    fn from_bytes(bytes: &[u8]) -> Self {
        let raw = String::from_utf8_lossy(bytes).into_owned();
        let base64 = Base64.encode(bytes);
        Self { raw, base64 }
    }
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

        let body = match data.open(16.kibibytes()).into_bytes().await {
            Ok(s) => Some(s),
            Err(e) => {
                println!("Error! {e}");
                None
            }
        };
        let mut complete = None;

        let body = match body {
            Some(b) => {
                println!("body: {:?}", b);
                complete = Some(b.is_complete());
                Some(Body::from_bytes(&b.value))
            }
            None => None,
        };

        let mut cookies = MultiMap::new();

        for c in req.cookies().iter() {
            cookies.insert(c.name().to_string(), c.value().to_string());
        }

        Outcome::Success(RequestData {
            method: req.method(),
            content_type: req
                .content_type()
                .map(|mt| format!("{}/{}", mt.0.top(), mt.0.sub())),
            body,
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
