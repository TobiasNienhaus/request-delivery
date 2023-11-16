use dashmap::DashMap;
use rocket::{
    http::Status,
    outcome::Outcome::Success,
    request::{self, FromRequest},
    Build, Request, Rocket, State,
};
use serde::Serialize;
use uuid::Uuid;

static AUTH_HEADER: &'static str = "X-Auth";

#[derive(Clone)]
pub struct AuthMap {
    ids: DashMap<Uuid, String>,
    tokens: DashMap<String, Uuid>,
}

unsafe impl Send for AuthMap {}
unsafe impl Sync for AuthMap {}

impl AuthMap {
    pub fn contains_id(&self, id: Uuid) -> bool {
        self.ids.contains_key(&id)
    }

    pub fn contains_value(&self, val: &str) -> bool {
        self.tokens.contains_key(val)
    }

    pub fn add(&self, id: Uuid, token: String) -> bool {
        if self.ids.contains_key(&id) || self.tokens.contains_key(&token) {
            false
        } else {
            self.ids.insert(id, token.clone());
            self.tokens.insert(token, id);
            true
        }
    }

    pub fn remove(&self, id: Uuid) -> bool {
        if let Some((k, v)) = self.ids.remove(&id) {
            let tval = self.tokens.get(&v);
            tval.map_or(false, |v| {
                let v = v.eq(&id);
                if !v {
                    eprintln!("IDS NOT EQUAL")
                }
                v
            })
        } else {
            false
        }
    }

    pub fn clone_token(&self, id: Uuid) -> Option<String> {
        self.ids.get(&id).map(|s| s.clone())
    }

    pub fn new() -> Self {
        AuthMap {
            ids: DashMap::new(),
            tokens: DashMap::new(),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Auth(pub String);
unsafe impl Send for Auth {}
unsafe impl Sync for Auth {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Success(v) = req.guard::<&State<AuthMap>>().await {
            for h in req.headers().get(AUTH_HEADER) {
                if v.contains_value(h) {
                    return request::Outcome::Success(Auth(h.to_owned()));
                }
            }
        } else {
            eprintln!("No AuthMap!!!");
            return request::Outcome::Error((Status::InternalServerError, ()));
        }
        request::Outcome::Error((Status::Unauthorized, ()))
    }
}

pub fn setup_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.manage(AuthMap::new())
}
