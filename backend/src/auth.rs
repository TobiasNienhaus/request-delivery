use crate::{AUTH_HEADER, MY_EPOCH};
use nanoid::nanoid;
use rocket::{fairing::AdHoc, Build, Rocket};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_db_pools::{
    sqlx::{self, sqlite::SqliteRow, FromRow, Row},
    Connection, Database,
};
use serde::{Deserialize, Serialize};

fn custom_timestamp() -> i64 {
    println!("{:?}", chrono::offset::Local::now().naive_local());
    let duration = MY_EPOCH.signed_duration_since(chrono::offset::Local::now().naive_local());
    -duration.num_seconds()
}

#[derive(Database)]
#[database("auth")]
pub struct AuthDb(sqlx::SqlitePool);

pub fn attach_db(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.attach(AuthDb::init());

    rocket.attach(AdHoc::try_on_ignite("AuthDB Init", db_setup_internal))
}

async fn db_setup_internal(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let db = AuthDb::fetch(&rocket).unwrap();
    if let Err(e) = sqlx::query("DROP TABLE IF EXISTS auth;")
        .execute(&**db)
        .await
        .and(
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS auth (
                id TEXT PRIMARY KEY,
                token TEXT,
                ts INTEGER)",
            )
            .execute(&**db)
            .await,
        )
    {
        eprintln!("Could not execute setup for AuthDB: {e:?}");
        Err(rocket)
    } else {
        Ok(rocket)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Auth {
    id: String,
    #[serde(default)]
    token: String,
}

impl Auth {
    fn rnd() -> Self {
        Auth {
            id: nanoid!(),
            token: nanoid!(),
        }
    }
}

impl FromRow<'_, SqliteRow> for Auth {
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
        if let Ok(auth) = sqlx::query_as::<_, Auth>("SELECT id, token FROM auth WHERE id = ?;")
            .bind(id)
            .fetch_one(&mut **self.db)
            .await
        {
            println!("=> [{}] has token [{}]", id, auth.token);
            return if auth.token.is_empty() || auth.token.eq(&self.token) {
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

pub struct NewAuthService {
    db: Connection<AuthDb>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for NewAuthService {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = match req.guard::<Connection<AuthDb>>().await.succeeded() {
            Some(db) => db,
            None => return Outcome::Forward(Status::InternalServerError),
        };

        Outcome::Success(NewAuthService { db })
    }
}

impl NewAuthService {
    pub async fn save_random(self) -> Result<Auth, Status> {
        self.save(Auth::rnd()).await
    }

    pub async fn save(mut self, mut auth: Auth) -> Result<Auth, Status> {
        if auth.id.len() < 8 {
            #[allow(unused_parens)]
            {
                auth.id += &nanoid!((8 - auth.id.len()));
            }
        }
        sqlx::query("INSERT OR FAIL INTO auth (id, token, ts) VALUES (?, ?, ?);")
            .bind(&auth.id)
            .bind(&auth.token)
            .bind(custom_timestamp())
            .execute(&mut **self.db)
            .await
            .map_err(|e| {
                eprintln!("SQL: {e:?}");
                Status::Unauthorized
            })?;
        Ok(auth)
    }
}
