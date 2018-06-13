#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate chrono;
#[macro_use]
extern crate serde_derive;

pub mod schema;
pub mod models;
pub mod games;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
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

// An alias to the type for a pool of Diesel Pg connections.
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Initializes a database pool.
pub fn init_pool() -> PgPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(manager).expect("db pool")
}


pub fn get_seesion(given_token: String, connection: PgConnection) -> Result<(User), AuthError> {
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
