extern crate rusty_pick_em;
extern crate diesel;
extern crate chrono;

use self::rusty_pick_em::*;
use self::rusty_pick_em::models::*;
use self::diesel::prelude::*;
use chrono::prelude::*;
use chrono::Duration;

fn main() {
    use rusty_pick_em::schema::games::dsl::*;
    use rusty_pick_em::schema::teams::dsl::*;

    let connection = establish_connection();

    let results = games.filter(time.lt(Local::now() + Duration::seconds(3600*24*7))).limit(50)
        .load::<Game>(&connection)
        .expect("Error loading games");

    println!("Displaying {} games", results.len());
    for game in results {
        let team1_name: String = teams.select(name).find(game.team1.unwrap()).first(&connection)
            .expect("Error loading team1");
        let team2_name: String = teams.select(name).find(game.team2.unwrap()).first(&connection)
            .expect("Error loading team2");
        // println!("The score was {}-{}", game.score1.unwrap_or(0), game.score2.unwrap_or(0));
        println!("The game is {:?} VS {:?}", team1_name, team2_name);
        // println!("It takes place the {:?} at {:?}", game.time.date(), game.time.time() )
    }
}
