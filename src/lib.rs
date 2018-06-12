#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate chrono;


pub mod schema;
pub mod models;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use chrono::prelude::*;
use std::env;
use models::User;

pub enum AuthError {
    NoToken,
    WrongToken,
    TokenExpired,
    WrongUsernameOrPassword
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn check_token(given_token: String, connection: PgConnection) -> Result<(User), AuthError> {
    use schema::users::dsl::*;

    let user: User = users.filter(token.eq(&given_token)).first(&connection)
        .expect("AuthError loading user while checking token");
    if user.token.is_some() && user.tokenExpireAt.is_some() {
        if user.token.as_ref().unwrap() == &given_token {
            if user.tokenExpireAt.unwrap() > Utc::now() {
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
}
