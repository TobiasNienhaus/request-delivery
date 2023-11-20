use crate::AUTH_HEADER;
use nanoid::nanoid;
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{self, FromRequest},
    Build, Request, Rocket,
};
use rocket_db_pools::{
    sqlx::{self, sqlite::SqliteRow, FromRow, Row},
    Connection, Database,
};
use serde::Serialize;

#[derive(Database)]
#[database("auth")]
pub struct AuthDb(sqlx::SqlitePool);

pub fn attach_db(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.attach(AuthDb::init());

    rocket.attach(AdHoc::try_on_ignite("AuthDB Init", db_setup_internal))
}

async fn db_setup_internal(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let db = AuthDb::fetch(&rocket).unwrap();
    if let Err(e) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS auth (
        id TEXT PRIMARY KEY,
        token TEXT UNIQUE)",
    )
    .execute(&**db)
    .await
    .and(sqlx::query("DELETE FROM auth;").execute(&**db).await)
    {
        eprintln!("Could not execute setup for AuthDB: {e:?}");
        Err(rocket)
    } else {
        Ok(rocket)
    }
}

#[derive(Clone, Debug)]
pub struct AuthGuardDb {
    id: String,
    token: String,
}

impl AuthGuardDb {
    pub fn id_matches(&self, id: &str) -> bool {
        self.id.eq(id)
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

impl FromRow<'_, SqliteRow> for AuthGuardDb {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            token: row.try_get("token")?,
        })
    }
}

unsafe impl Send for AuthGuardDb {}
unsafe impl Sync for AuthGuardDb {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuardDb {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        for c in req.cookies().iter() {
            println!("{}: {}", c.name(), c.value());
        }
        let token = if let Some(token) = req.headers().get(AUTH_HEADER).next() {
            token
        } else if let Some(Ok(token)) = req.query_value("token") {
            token
        } else {
            return request::Outcome::Forward(Status::Unauthorized);
        };
        if let request::Outcome::Success(mut db) = req.guard::<Connection<AuthDb>>().await {
            if let Ok(auth) =
                sqlx::query_as::<_, AuthGuardDb>("SELECT id, token FROM auth WHERE token = ?;")
                    .bind(token)
                    .fetch_one(&mut **db)
                    .await
            {
                return request::Outcome::Success(auth);
            }
        }
        request::Outcome::Forward(Status::Unauthorized)
    }
}

#[derive(Clone, Serialize)]
pub struct NewAuth {
    id: String,
    token: String,
}

unsafe impl Send for NewAuth {}
unsafe impl Sync for NewAuth {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for NewAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let id = nanoid!();
        let token = nanoid!();
        if let request::Outcome::Success(mut db) = req.guard::<Connection<AuthDb>>().await {
            if let Err(e) = sqlx::query("INSERT OR FAIL INTO auth (id, token) VALUES (?, ?);")
                .bind(&id)
                .bind(&token)
                .execute(&mut **db)
                .await
            {
                eprintln!("Could not insert: {e:?}");
                return request::Outcome::Forward(Status::InternalServerError);
            }
        }
        request::Outcome::Success(NewAuth { id, token })
    }
}
