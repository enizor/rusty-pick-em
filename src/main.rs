#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate rand;
extern crate rocket_dyn_templates;
extern crate time;

mod games;
mod models;
mod routes;
mod schema;
mod session;

use routes::*;

use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                login,
                auth_user,
                logout,
                routes::games,
                games_with_date,
                postbet,
                game_detail,
                postresult
            ],
        )
        .mount("/images", FileServer::from("/static/images"))
        .mount("/static", FileServer::from("/static"))
        .attach(DbConn::fairing())
        .attach(Template::fairing())
}
