extern crate chrono;
extern crate diesel;
extern crate serde;


use self::diesel::prelude::*;
use models::{Game, Team};
use chrono::{DateTime, FixedOffset, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env::args;

#[derive(Serialize)]
pub struct GameContext {
    pub time: DateTime<Utc>,
    pub score1: i32,
    pub score2: i32,
    pub team1: Team,
    pub team2: Team, // TODO: add related bet
}

impl Game {
    pub fn to_context(&self, conn: &PgConnection) -> GameContext {

        use schema::teams::dsl::teams;
        let team1 = if self.team1.is_some() {
            teams.find(self.team1.unwrap())
                .get_result::<Team>(conn)
                .expect("Unable to update")
            } else {
                Team {
                    id: 0,
                    name: "TBD".into(),
                    abbr: "TBD".into(),
                    flag: "".into()
                }
            };
        let team2 = if self.team2.is_some() {
            teams.find(self.team2.unwrap())
                .get_result::<Team>(conn)
                .expect("Unable to update")
            } else {
                Team {
                    id: 0,
                    name: "TBD".into(),
                    abbr: "TBD".into(),
                    flag: "".into()
                }
            };
        GameContext {
            time: self.time,
            score1: self.score1.unwrap_or(0),
            score2: self.score2.unwrap_or(0),
            team1: team1,
            team2: team2
        }
    }
}

pub fn upcoming_games(conn: &PgConnection, number: i32, offset: i32) -> Vec<GameContext> {
    use schema::games::dsl::{games, time};
    games.order(time.asc()).limit(5).load::<Game>(conn)
        .expect("Unable to find next games")
        .iter().map(|game| game.to_context(conn)).collect()
}
