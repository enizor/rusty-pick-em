use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database, diesel};

use crate::games::*;
use crate::models::*;
use crate::schema;
use crate::session::*;
use chrono::prelude::*;
use diesel::prelude::*;

use time::Duration;

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
        .map(|msg| {
            let (name, m) = msg.into_inner();
            format!("{}: {}", name, m)
        })
        .unwrap_or_else(|| "Login".to_string());
    Template::render("login", &Message { flash: msg })
}

#[derive(FromForm, Serialize)]
pub struct AuthUser {
    name: String,
    password: String,
}

#[post("/login", data = "<auth_user>")]
pub async fn auth_user(
    auth_user: Form<AuthUser>,
    cookies: &CookieJar<'_>,
    conn: DbConn,
) -> Flash<Redirect> {
    if let Some(token) = conn.run(move |c| create_session(&auth_user.name, &c)).await {
        cookies.add_private(Cookie::new("token", token));
        Flash::success(Redirect::to("/games"), "Bienvenue")
    } else {
        Flash::error(Redirect::to("/login"), "Identifiants incorrects")
    }
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, conn: DbConn) -> Flash<Redirect> {
    if let Some(cookie) = cookies.get_private("token") {
        conn.run(move |c| {
            if let Ok(user) = get_session(cookie.value(), &c) {
                delete_session(&user.name, &c);
            }
        })
        .await;
        cookies.remove_private(Cookie::named("token"));
        return Flash::success(Redirect::to("/login"), "Vous avez bien été déconnnecté");
    }
    Flash::error(
        Redirect::to("/login"),
        "Votre session a expiré. Merci de vous authentifier.",
    )
}

#[derive(Serialize)]
pub struct GamesContext {
    next_day: String,
    previous_day: String,
    flash: String,
    username: String,
    games: Vec<GameDetails>,
    day: String,
}

#[get("/games?<date>")]
pub async fn games_with_date(
    date: &str,
    conn: DbConn,
    flash: Option<FlashMessage<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Template, Flash<Redirect>> {
    if let Some(cookie) = cookies.get_private("token") {
        if let Ok(user) = conn.run(move |c| get_session(cookie.value(), &c)).await {
            let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .unwrap_or_else(|_| Utc::today().naive_utc());
            let previous_date = parsed_date.checked_sub_signed(Duration::days(1)).unwrap();
            let new_date = conn.run(move |c| find_date(&previous_date, &c)).await;
            if parsed_date != new_date {
                return Err(Flash::warning(
                    Redirect::to(format!("/games?date={}", new_date.format("%Y-%m-%d"))),
                    "Vous avez été redirigé vers un jour avec des matchs",
                ));
            }
            let user_id = user.id;
            let next_day = conn
                .run(move |c| next_date(&parsed_date, &c))
                .await
                .map_or("".to_string(), |d| d.format("%Y-%m-%d").to_string());
            let previous_day = conn
                .run(move |c| prev_date(&parsed_date, &c))
                .await
                .map_or("".to_string(), |d| d.format("%Y-%m-%d").to_string());
            let context = GamesContext {
                next_day,
                previous_day,
                flash: flash
                    .map(|msg| msg.message().to_string())
                    .unwrap_or_else(|| "".to_string()),
                games: conn
                    .run(move |c| upcoming_games(&c, parsed_date, user_id))
                    .await,
                username: user.name,
                day: parsed_date.format("%Y-%m-%d").to_string(),
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
pub async fn games(
    conn: DbConn,
    _flash: Option<FlashMessage<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Flash<Redirect>> {
    if let Some(cookie) = cookies.get_private("token") {
        if conn
            .run(move |c| get_session(cookie.value(), &c))
            .await
            .is_ok()
        {
            let parsed_date = Utc::today().naive_utc();
            let new_date = conn
                .run(move |c| {
                    find_date(
                        &parsed_date.checked_sub_signed(Duration::days(1)).unwrap(),
                        &c,
                    )
                })
                .await;
            return Ok(Redirect::to(format!(
                "/games?date={}",
                new_date.format("%Y-%m-%d")
            )));
        }
    }
    Err(Flash::error(
        Redirect::to("/login"),
        "Vous avez été déconnecté. Merci de vous authentifier.",
    ))
}

#[derive(Debug, FromForm, Serialize)]
pub struct PlaceBet {
    game_id: i32,
    score1: i32,
    score2: i32,
}

#[post("/games", data = "<bet>")]
pub async fn postbet(
    bet: Form<PlaceBet>,
    conn: DbConn,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    if let Some(cookie) = cookies.get_private("token") {
        if let Ok(user) = conn.run(move |c| get_session(cookie.value(), &c)).await {
            // Update if exists
            use schema::games::dsl::*;
            #[allow(non_snake_case)]
            let game_ID = bet.game_id;
            #[allow(non_snake_case)]
            let user_ID = dbg!(user.id);
            let bet_score1 = bet.score1;
            let bet_score2 = bet.score2;

            let has_game = conn
                .run(move |c| {
                    games
                        .filter(time.gt(Utc::now().naive_utc()))
                        .find(game_ID)
                        .first::<Game>(c)
                })
                .await
                .is_ok();
            if has_game {
                use schema::bets::dsl::*;
                let result = conn
                    .run(move |c| {
                        diesel::update(bets.filter(game_id.eq(game_ID)).filter(user_id.eq(user_ID)))
                            .set((score1.eq(bet_score1), score2.eq(bet_score2)))
                            .execute(c)
                    })
                    .await;

                // Else insert the bet
                if result.is_err() || result == Ok(0) {
                    conn.run(move |c| {
                        diesel::insert_into(bets)
                            .values((
                                user_id.eq(user_ID),
                                game_id.eq(game_ID),
                                score1.eq(bet_score1),
                                score2.eq(bet_score2),
                            ))
                            .execute(c)
                    })
                    .await
                    .expect("Error saving new bet");
                }
                return Flash::success(
                    Redirect::to(format!("/game/{}", game_ID)),
                    "Pari enregistré",
                );
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
pub async fn game_detail(
    id: i32,
    conn: DbConn,
    flash: Option<FlashMessage<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Template, Flash<Redirect>> {
    if let Some(cookie) = cookies.get_private("token") {
        if let Ok(user) = conn.run(move |c| get_session(cookie.value(), c)).await {
            if let Some(game) = conn.run(move |c| get_game(id, c)).await {
                let user_id = user.id;
                let day = game.time.format("%Y-%m-%d").to_string();
                let context = GameContext {
                    flash: flash
                        .map(|msg| msg.message().to_string())
                        .unwrap_or_default(),
                    username: user.name,
                    game: conn.run(move |c| game.to_context(user_id, c)).await,
                    admin: user.isAdmin,
                    day,
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
pub async fn postresult(
    res: Form<PostResults>,
    conn: DbConn,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    if let Some(cookie) = cookies.get_private("token") {
        if let Ok(user) = conn.run(move |c| get_session(cookie.value(), c)).await {
            if user.isAdmin {
                // Update if exists
                use schema::games::dsl::*;
                #[allow(non_snake_case)]
                let game_ID = res.game_id;
                if let Ok(game) = conn
                    .run(move |c| games.find(game_ID).first::<Game>(c))
                    .await
                {
                    conn.run(move |c| {
                        diesel::update(games.filter(id.eq(game.id)))
                            .set((score1.eq(&res.score1), score2.eq(&res.score2)))
                            .execute(c)
                    })
                    .await
                    .expect("Error setting game result");
                    let result: Game = conn
                        .run(move |c| games.find(game_ID).first(c))
                        .await
                        .expect("Error getting game result");
                    conn.run(move |c| result.update_bets(c)).await;
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

#[database("euro21")] // Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(diesel::SqliteConnection);
