
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rusty_pick_em;
use self::rusty_pick_em::routes::*;
use self::rusty_pick_em::*;

extern crate rocket;
extern crate rocket_contrib;
use rocket_contrib::Template;

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![index, login, authUser, logout, test, games, games_with_date, postbet, game_detail, postresult],
        )
        .attach(Template::fairing())
        .manage(init_pool())
        .launch();
}
