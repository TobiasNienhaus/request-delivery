use crate::{auth_db::AuthDb, AUTH_HEADER};
use futures::TryFutureExt;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_db_pools::{
    sqlx::{self, sqlite::SqliteRow, FromRow, Row},
    Connection, Database,
};

#[derive(Clone, Debug)]
struct AuthDbEntry {
    id: String,
    token: String,
}

impl FromRow<'_, SqliteRow> for AuthDbEntry {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            token: row.try_get("token")?,
        })
    }
}

pub struct AuthService<const ALLOW_QUERY: bool = false> {
    token: String,
    db: Connection<AuthDb>,
}

#[rocket::async_trait]
impl<'r, const ALLOW_QUERY: bool> FromRequest<'r> for AuthService<ALLOW_QUERY> {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = match req.guard::<Connection<AuthDb>>().await.succeeded() {
            Some(db) => db,
            None => return Outcome::Forward(Status::InternalServerError),
        };

        let token = if let Some(token) = Self::get_token(req) {
            token
        } else {
            println!("Found no token!");
            return Outcome::Success(AuthService {
                token: "".to_owned(),
                db,
            });
        };
        Outcome::Success(AuthService {
            token: token.to_string(),
            db,
        })
    }
}

impl<'r, const ALLOW_QUERY: bool> AuthService<ALLOW_QUERY> {
    fn get_token(req: &'r Request<'_>) -> Option<&'r str> {
        if ALLOW_QUERY {
            let token = if let Some(token) = req.headers().get(AUTH_HEADER).next() {
                token
            } else if let Some(Ok(token)) = req.query_value("token") {
                token
            } else {
                return None;
            };
            Some(token)
        } else {
            req.headers().get(AUTH_HEADER).next()
        }
    }

    pub async fn check(&mut self, id: &str) -> Result<(), Status> {
        println!("Checking token [{}] and id [{}]", self.token, id);
        if let Ok(auth) =
            sqlx::query_as::<_, AuthDbEntry>("SELECT id, token FROM auth WHERE id = ?;")
                .bind(id)
                .fetch_one(&mut **self.db)
                .await
        {
            println!("=> [{}] has token [{}]", id, auth.token);
            return if auth.token.eq(&self.token) {
                Ok(())
            } else {
                Err(Status::Unauthorized)
            };
        };
        Err(Status::NotFound)
    }

    pub async fn check_bool(&mut self, id: &str) -> bool {
        self.check(id).await.is_ok()
    }
}
