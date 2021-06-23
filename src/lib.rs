#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
extern crate dotenv;
extern crate chrono;
extern crate rand;
extern crate time;
extern crate rocket_dyn_templates;

pub mod schema;
pub mod models;
pub mod games;
pub mod routes;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
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

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn get_session(given_token: &str, conn: &SqliteConnection) -> Result<User, AuthError> {
    use schema::users::dsl::*;
    if let Ok(user) = users.filter(token.eq(&given_token)).first::<User>(conn) {
        if user.token.is_some() && user.tokenExpireAt.is_some() {
            if user.token.as_ref().unwrap() == &given_token {
                if user.tokenExpireAt.unwrap() > Utc::now().naive_utc() {
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
    } else {
        Err(AuthError::NoToken)
    }
}

pub fn create_session(username: &str, conn: &SqliteConnection) -> Option<String> {
    use schema::users::dsl::*;
    println!("{}", username);
    let new_token = format!("{:x}", rand::random::<u64>());
    let user_updated = diesel::update(users.filter(name.eq(&username)))
        .set( (token.eq(&new_token),
            tokenExpireAt.eq(Utc::now().naive_utc().checked_add_signed(chrono::Duration::days(90)))))
        .execute(conn);
    if user_updated.is_ok() {
        Some(new_token)
    } else {
        None
    }
}

pub fn delete_session(username: &str, conn: &SqliteConnection) {
    use schema::users::dsl::*;

    let user = users.filter(name.eq(&username));

    let user_updated = diesel::update(users)
        .set( (token.eq(&""),
            tokenExpireAt.eq(Utc::now().naive_utc())))
        .execute(conn)
        .expect("Unable to update");
}
