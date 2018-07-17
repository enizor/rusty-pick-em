#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate chrono;
extern crate time;

use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{self, FlashMessage, Form, FromRequest};
use rocket::response::{Flash, NamedFile, Redirect};
use rocket::{Outcome, Request, State};

use rocket_contrib::Template;

use games::*;
use models::*;
use *;
use schema::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::ops::Deref;
use chrono::prelude::*;

use time::Duration;

use std::fs::File;
use std::path::{Path, PathBuf};

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to("/games")
}

#[derive(Serialize)]
pub struct Message {
    flash: String,
}

#[get("/login")]
pub fn login(flash: Option<FlashMessage>) -> Template {
    let msg = flash
        .map(|msg| format!("{}: {}", msg.name(), msg.msg()))
        .unwrap_or_else(|| "Login".to_string());
    Template::render("login", &Message { flash: msg })
}

#[derive(FromForm, Serialize)]
pub struct AuthUser {
    name: String,
    password: String,
}

#[post("/login", data = "<auth_user>")]
pub fn authUser(auth_user: Form<AuthUser>, mut cookies: Cookies, conn: DbConn) -> Flash<Redirect> {
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
pub fn logout(mut cookies: Cookies, conn: DbConn) -> Flash<Redirect> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            delete_session(&user.name, &*conn);
            cookies.remove_private(Cookie::named("token"));
            return Flash::success(Redirect::to("/login"), "Vous avez bien été déconnnecté");
        }
    }
    Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.")
}

#[get("/test")]
pub fn test(mut cookies: Cookies, conn: DbConn) -> Result<&'static str, Flash<Redirect>> {
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
pub struct GamesContext {
    next_day: String,
    previous_day: String,
    flash: String,
    username: String,
    games: Vec<GameDetails>,
}

#[derive(FromForm)]
pub struct CustomDate {
    date: Option<String>
}
#[get("/games?<date>")]
pub fn games_with_date(
    date: CustomDate,
    conn: DbConn,
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
) -> Result<Template, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            let parsed_date = NaiveDate::parse_from_str(&date.date.unwrap_or("LOL NOPE".to_string()), "%Y-%m-%d").unwrap_or(Local::today().naive_local());
            let new_date = find_date(parsed_date.checked_sub_signed(Duration::days(1)).unwrap(), &conn);
            if parsed_date != new_date {
                return Err(Flash::warning(
                    Redirect::to(&format!("/games?date={}", new_date.format("%Y-%m-%d"))), "Vous avez été redirigé vers un jour avec des matchs"))
            }
            let context = GamesContext {
                next_day: next_date(parsed_date, &conn).map_or("".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                previous_day: prev_date(parsed_date, &conn).map_or("".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                flash: flash
                    .map(|msg| msg.msg().to_string())
                    .unwrap_or("".to_string()),
                username: user.name,
                games: upcoming_games(&*conn, parsed_date, user.id)
            };
            return Ok(Template::render("games", &context));
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Vous avez été déconnecté. Merci de vous authentifier.",
    ))
}

#[get("/games")]
pub fn games(
    conn: DbConn,
    flash: Option<FlashMessage>,
    mut cookies: Cookies,
) -> Result<Redirect, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            let parsed_date = Local::today().naive_local();
            let new_date = find_date(parsed_date.checked_sub_signed(Duration::days(1)).unwrap(), &conn);
            return Ok(Redirect::to(&format!("/games?date={}", new_date.format("%Y-%m-%d"))));
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Vous avez été déconnecté. Merci de vous authentifier.",
    ))
}

#[derive(FromForm, Serialize)]
pub struct PlaceBet {
    game_id: i32,
    score1: i32,
    score2: i32,
}

#[post("/games", data = "<bet>")]
pub fn postbet(bet: Form<PlaceBet>, conn: DbConn, mut cookies: Cookies) -> Flash<Redirect> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            // Update if exists
            use schema::games::dsl::*;
            if games.filter(time.gt(Local::now()))
            .find(&bet.get().game_id).first::<Game>(&*conn).is_ok() {

                use schema::bets::dsl::*;
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
                return Flash::success(Redirect::to(&format!("/game/{}", &bet.get().game_id)), "Pari enregistré");
            }
        }
    }
    Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    )
}

#[derive(Serialize)]
pub struct GameContext {
    flash: String,
    username: String,
    game: GameDetails,
    admin: bool,
    day: String,
}

#[get("/game/<id>")]
pub fn game_detail(id: i32, conn: DbConn, flash: Option<FlashMessage>, mut cookies: Cookies )
-> Result<Template, Flash<Redirect>> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            if let Some(game) = get_game(id, &*conn) {
                let context = GameContext {
                    flash: flash
                        .map(|msg| msg.msg().to_string())
                        .unwrap_or("".to_string()),
                    username: user.name,
                    game: game.to_context(user.id, &*conn),
                    admin: user.isAdmin,
                    day: game.time.format("%Y-%m-%d").to_string()
                };
                return Ok(Template::render("game", &context));
            }
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Vous avez été déconnecté. Merci de vous authentifier.",
    ))
}

#[derive(FromForm, Serialize)]
pub struct PostResults {
    game_id: i32,
    score1: i32,
    score2: i32,
}

#[post("/result", data = "<res>")]
pub fn postresult(res: Form<PostResults>, conn: DbConn, mut cookies: Cookies) -> Flash<Redirect> {
    if let Some(ref cookie) = cookies.get_private("token") {
        if let Ok(user) = get_session(cookie.value(), &*conn) {
            if user.isAdmin {
                // Update if exists
                use schema::games::dsl::*;
                if let Ok(game) = games.find(&res.get().game_id).first::<Game>(&*conn) {

                    let result = diesel::update(
                        games.filter(id.eq(game.id))
                    ).set((score1.eq(&res.get().score1), score2.eq(&res.get().score2)))
                    .get_result::<Game>(&*conn)
                    .expect("Error setting game result");

                    result.update_bets(&conn);
                    return Flash::success(Redirect::to("/games"), "Résultat enregistré");
                }
            }
        }
    }
    Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    )
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
