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

use self::rusty_pick_em::games::*;
use self::rusty_pick_em::models::*;
use self::rusty_pick_em::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::ops::Deref;
use chrono::prelude::*;

use std::fs::File;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/games")
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
fn authUser(auth_user: Form<AuthUser>, mut cookies: Cookies, conn: DbConn) -> Flash<Redirect> {
    if let Some(token) = create_session(&auth_user.get().name, &*conn) {
        cookies.add_private(Cookie::new("token", token));
        Flash::success(
        Redirect::to("/games"),
        "Bienvenue")
    } else {
        Flash::error(
        Redirect::to("/login"),
        "Identifiants incorrects")
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies, conn: DbConn) -> Result<&'static str, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            delete_session(&user.name, &*conn);
            cookies.remove_private(Cookie::named("token"));
            return Ok("You successfully logged out");
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    ))
}

#[get("/test")]
fn test(mut cookies: Cookies, conn: DbConn) -> Result<&'static str, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if get_session(cookie.value(), &*conn).is_ok() {
            return Ok("Acces granted!");
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    ))
}

#[derive(Serialize)]
struct GamesContext {
    flash: String,
    username: String,
    games: Vec<GameContext>,
}

#[get("/games")]
fn games(
    conn: DbConn,
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
) -> Result<Template, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            let context = GamesContext {
                flash: flash
                    .map(|msg| msg.msg().to_string())
                    .unwrap_or("".to_string()),
                username: user.name,
                games: upcoming_games(&*conn, 15, 0, user.id),
            };
            return Ok(Template::render("games", &context));
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Vous avez été déconnecté. Merci de vous authentifier.",
    ))
}

#[derive(FromForm, Serialize)]
struct PlaceBet {
    game_id: i32,
    score1: i32,
    score2: i32,
}

#[post("/games", data = "<bet>")]
fn postbet(bet: Form<PlaceBet>, conn: DbConn, mut cookies: Cookies) -> Flash<Redirect> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            // Update if exists
            use self::schema::games::dsl::*;
            if games.filter(time.gt(Local::now()))
            .find(&bet.get().game_id).first::<Game>(&*conn).is_ok() {

                use self::schema::bets::dsl::*;
                let result = diesel::update(
                    bets.filter(game_id.eq(&bet.get().game_id))
                        .filter(user_id.eq(user.id)),
                ).set((score1.eq(&bet.get().score1), score2.eq(&bet.get().score2)))
                    .get_result::<Bet>(&*conn);

                // Else insert the bet
                if result.is_err() {
                    diesel::insert_into(bets)
                    .values((
                        user_id.eq(user.id),
                        game_id.eq(&bet.get().game_id),
                        score1.eq(&bet.get().score1),
                        score2.eq(&bet.get().score2)
                    ))
                    .execute(&*conn)
                    .expect("Error saving new post");
                }
                return Flash::success(Redirect::to("/games"), "Pari enregistré");
            }
        }
    }
    Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    )
}

// /* check cookie

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![index, login, authUser, logout, test, games, postbet],
        )
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
