// extern crate ring;
// extern crate rand;
// extern crate diesel;
// extern crate chrono;

// use ring::{digest, pbkdf2};
// use std::collections::HashMap;
// use diesel::prelude::*;
// use models::User;
// use chrono::{DateTime, Utc, FixedOffset};


// static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA256;
// const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
// pub type Credential = [u8; CREDENTIAL_LEN];

// enum AuthError {
//     NoToken,
//     WrongToken,
//     TokenExpired,
//     WrongUsernameOrPassword
// }

// struct PasswordDatabase {
//     pbkdf2_iterations: u32,
//     db_salt_component: [u8; 16],

//     // Normally this would be a persistent database.
//     storage: HashMap<String, Credential>,
// }

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



// impl User {


    // pub fn store_password(&mut self, username: &str, password: &str) {
    //     let salt = self.salt(username);
    //     let mut to_store: Credential = [0u8; CREDENTIAL_LEN];
    //     pbkdf2::derive(DIGEST_ALG, self.pbkdf2_iterations, &salt,
    //                    password.as_bytes(), &mut to_store);
    //     self.storage.insert(String::from(username), to_store);
    // }

    // pub fn verify_password(&self, username: &str, attempted_password: &str)
    //                        -> Result<(), Error> {
    //     match self.storage.get(username) {
    //        Some(actual_password) => {
    //            let salt = self.salt(username);
    //            pbkdf2::verify(DIGEST_ALG, self.pbkdf2_iterations, &salt,
    //                           attempted_password.as_bytes(),
    //                           actual_password)
    //                 .map_err(|_| Error::WrongUsernameOrPassword)
    //        },

    //        None => Err(Error::WrongUsernameOrPassword)
    //     }
    // }

    // // The salt should have a user-specific component so that an attacker
    // // cannot crack one password for multiple users in the database. It
    // // should have a database-unique component so that an attacker cannot
    // // crack the same user's password across databases in the unfortunate
    // // but common case that the user has used the same password for
    // // multiple systems.
    // fn salt(&self, username: &str) -> Vec<u8> {
    //     let mut salt = Vec::with_capacity(self.db_salt_component.len() +
    //                                       username.as_bytes().len());
    //     salt.extend(self.db_salt_component.as_ref());
    //     salt.extend(username.as_bytes());
    //     salt
    // }
// }
