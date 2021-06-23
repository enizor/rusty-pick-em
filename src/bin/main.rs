

extern crate rusty_pick_em;
use self::rusty_pick_em::routes::*;

#[macro_use] extern crate rocket;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![index, login, authUser, logout, games, games_with_date, postbet, game_detail, postresult],
        )
        .mount("/images", FileServer::from("/static/images"))
        .mount("/static", FileServer::from("/static"))
        .attach(DbConn::fairing())
        .attach(Template::fairing())
}
