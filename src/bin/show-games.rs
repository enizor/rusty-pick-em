extern crate rusty_pick_em;
extern crate diesel;

use self::rusty_pick_em::*;
use self::rusty_pick_em::models::*;
use self::diesel::prelude::*;

fn main() {
    use rusty_pick_em::schema::games::dsl::*;

    let connection = establish_connection();
    let results = games.limit(5)
        .load::<Game>(&connection)
        .expect("Error loading games");

    println!("Displaying {} games", results.len());
    for game in results {
        println!("The score was {}-{}", game.score1, game.score2);
        println!("It took place the {:?} at {:?}", game.time.unwrap().date(), game.time.unwrap().time() )
    }
}
