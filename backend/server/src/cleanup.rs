use std::fs::File;
use std::io::prelude::*;

use futures::SinkExt as _;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use rocket_db_pools::{sqlx, Connection};
use shared::custom_timestamp;

use crate::{
    auth::{Auth, AuthDb},
    ThingMap, WsMessage, AUTH_HEADER, MY_EPOCH,
};

use super::{CLEANUP_TOKEN, CONFIG};

pub(crate) fn init() {
    let mut f = File::create(CONFIG.secret_path()).expect("Could not create secret file");
    f.write_all(CLEANUP_TOKEN.as_bytes())
        .expect("Could not write token");
}

pub struct ConfigurationAuth();

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ConfigurationAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = if let Some(token) = req.headers().get(AUTH_HEADER).next() {
            token
        } else {
            return Outcome::Forward(Status::Unauthorized);
        };
        if !token.eq(&*CLEANUP_TOKEN) {
            return Outcome::Forward(Status::Forbidden);
        }
        Outcome::Success(ConfigurationAuth())
    }
}

#[delete("/cleanup")]
pub async fn cleanup_tokens(
    _ca: ConfigurationAuth,
    mut db: Connection<AuthDb>,
    map: &State<ThingMap>,
) -> Status {
    let ts = custom_timestamp(*MY_EPOCH) - CONFIG.max_age();
    match sqlx::query_as::<_, Auth>("SELECT id, token FROM auth WHERE ts < ?;")
        .bind(ts)
        .fetch_all(&mut **db)
        .await
    {
        Ok(res) => {
            println!("{} Tokens expired", res.len());
            for auth in res {
                if let Some((_, senders)) = map.remove(auth.id()) {
                    for sender in senders {
                        if !sender.is_closed() {
                            if let Err(e) = sender.clone().send(WsMessage::TokenExpired).await {
                                eprintln!(
                                    "Send Error (closing channel due to token expiration): {}",
                                    e
                                );
                                sender.clone().close_channel();
                            };
                        }
                    }
                }
            }
            if let Err(e) = sqlx::query("DELETE FROM auth WHERE ts < ?;")
                .bind(ts)
                .execute(&mut **db)
                .await
            {
                eprintln!("Could not delete expired tokens from DB: {e}");
            }
            Status::Accepted
        }
        Err(e) => {
            eprintln!("Could not clear tokens! {e}");
            Status::InternalServerError
        }
    }
}
