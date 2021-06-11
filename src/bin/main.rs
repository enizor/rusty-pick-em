

extern crate rusty_pick_em;
use self::rusty_pick_em::routes::*;

#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![index, login, authUser, logout, games, games_with_date, postbet, game_detail, postresult],
        )
        .attach(DbConn::fairing())
        .attach(Template::fairing())
}
