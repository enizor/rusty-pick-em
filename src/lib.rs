#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate dotenv;
extern crate rand;
extern crate rocket_dyn_templates;
extern crate time;

pub mod games;
pub mod models;
pub mod routes;
pub mod schema;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use models::User;
use std::env;

pub enum AuthError {
    NoToken,
    WrongToken,
    TokenExpired,
    WrongUsernameOrPassword,
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_session(given_token: &str, conn: &SqliteConnection) -> Result<User, AuthError> {
    use schema::users::dsl::*;
    if let Ok(user) = users.filter(token.eq(&given_token)).first::<User>(conn) {
        if user.token.is_some() && user.tokenExpireAt.is_some() {
            if user.token.as_ref().unwrap() == given_token {
                if user.tokenExpireAt.unwrap() > Utc::now().naive_utc() {
                    Ok(user)
                } else {
                    Err(AuthError::TokenExpired)
                }
            } else {
                Err(AuthError::WrongToken)
            }
        } else {
            Err(AuthError::NoToken)
        }
    } else {
        Err(AuthError::NoToken)
    }
}

pub fn create_session(username: &str, conn: &SqliteConnection) -> Option<String> {
    use schema::users::dsl::*;
    let new_token = format!("{:x}", rand::random::<u64>());
    let user_updated = diesel::update(users.filter(name.eq(&username)))
        .set((
            token.eq(&new_token),
            tokenExpireAt.eq(Utc::now()
                .naive_utc()
                .checked_add_signed(chrono::Duration::days(90))),
        ))
        .execute(conn);
    if user_updated.is_ok() {
        Some(new_token)
    } else {
        None
    }
}

pub fn delete_session(username: &str, conn: &SqliteConnection) {
    use schema::users::dsl::*;

    diesel::update(users.filter(name.eq(&username)))
        .set((token.eq(&""), tokenExpireAt.eq(Utc::now().naive_utc())))
        .execute(conn)
        .expect("Unable to update");
}
