use crate::models::User;
use crate::schema;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
pub enum AuthError {
    NoToken,
    WrongToken,
    TokenExpired,
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
