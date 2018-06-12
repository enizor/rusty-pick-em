extern crate rusty_pick_em;
extern crate diesel;
extern crate chrono;

use self::diesel::prelude::*;
use self::rusty_pick_em::*;
use self::rusty_pick_em::models::Game;
use std::env::args;
use chrono::{DateTime, Utc, FixedOffset};

fn main() {
    use rusty_pick_em::schema::games::dsl::{games, time};

    let strtime = args().nth(1).expect("update_game requires a time");

    let new_time = DateTime::<FixedOffset>::parse_from_rfc3339(&strtime)
    .expect("Please format according to RFC3339").with_timezone(&Utc);
    let connection = establish_connection();

    let game = diesel::update(games.find(1))
        .set(time.eq(new_time))
        .get_result::<Game>(&connection)
        .expect("Unable to update");
    println!("Game set to {} : {}",game.time.date(), game.time.time());
}
