#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate chrono;
#[macro_use]
extern crate serde_derive;
extern crate rand;

pub mod schema;
pub mod models;
pub mod games;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
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


pub fn get_session(given_token: &str, conn: &PgConnection) -> Result<(User), AuthError> {
    use schema::users::dsl::*;

    let user: User = users.filter(token.eq(&given_token)).first(conn)
        .expect("AuthError loading user while checking token");
    if user.token.is_some() && user.tokenExpireAt.is_some() {
        if user.token.as_ref().unwrap() == &given_token {
            if user.tokenExpireAt.unwrap() > Utc::now() {
                Ok(user)
            } else {
                println!("token expired");
                Err(AuthError::TokenExpired)
            }
        } else {
                println!("wrong token");
            Err(AuthError::WrongToken)
        }
    } else {
        println!("No token");
        Err(AuthError::NoToken)
    }
}

pub fn create_session(username: &str, conn: &PgConnection) -> Option<String> {
    use schema::users::dsl::*;
    println!("{}", username);
    let new_token = format!("{:x}", rand::random::<u64>());

    let user_updated = diesel::update(users.filter(name.eq(&username)))
        .set( (token.eq(&new_token),
            tokenexpireat.eq(Utc::now().checked_add_signed(chrono::Duration::days(90)))))
        .get_result::<User>(conn);
    if user_updated.is_ok() {
        Some(new_token)
    } else {
        None
    }
}

pub fn delete_session(username: &str, conn: &PgConnection) {
    use schema::users::dsl::*;

    let user = users.filter(name.eq(&username));

    let user_updated = diesel::update(users)
        .set( (token.eq(&""),
            tokenexpireat.eq(Utc::now())))
        .get_result::<User>(conn)
        .expect("Unable to update");
}
