#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate rusty_pick_em;

use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{self, FlashMessage, Form, FromRequest};
use rocket::response::{Flash, NamedFile, Redirect};
use rocket::{Outcome, Request, State};

use rocket_contrib::Template;

use self::rusty_pick_em::*;
use self::rusty_pick_em::games::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::ops::Deref;

use std::fs::File;
use std::path::{Path, PathBuf};

#[get("/<file..>", rank = 5)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("vue/dist/").join(file)).ok()
}

#[derive(Serialize)]
struct Message {
    flash: String,
}

#[get("/login")]
fn login(flash: Option<FlashMessage>) -> Template {
    let msg = flash
        .map(|msg| format!("{}: {}", msg.name(), msg.msg()))
        .unwrap_or_else(|| "Login".to_string());
    Template::render("login", &Message { flash: msg })
}

#[derive(FromForm, Serialize)]
struct AuthUser {
    name: String,
    password: String,
}

#[post("/login", data = "<auth_user>")]
fn authUser(auth_user: Form<AuthUser>, mut cookies: Cookies) -> Template {
    let token = "TEST";
    cookies.add_private(Cookie::new("token", token));
    Template::render("test", &auth_user.get())
    // CHECK passwd
    // redirect to login or main
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> &'static str {
    cookies.remove_private(Cookie::named("token"));
    "You successfully logged out"
}

#[get("/test")]
fn test(mut cookies: Cookies) -> Result<&'static str, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if cookie.value() == "TEST" {
            return Ok("You successfully logged out");
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    ))
}

#[derive(Serialize)]
struct GamesContext {
    games: Vec<GameContext>,
}

#[get("/games")]
fn games(conn: DbConn) -> Template {
    let context = GamesContext {
        games: upcoming_games(&*conn, 5, 0),
    };
    Template::render("games", &context)
}

// /* check cookie

fn main() {
    rocket::ignite()
        .mount("/", routes![files, login, authUser, logout, test, games])
        .attach(Template::fairing())
        .manage(init_pool())
        .launch();
}

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<PgPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &DbConn as an &PgConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
