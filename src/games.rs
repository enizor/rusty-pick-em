extern crate chrono;
extern crate diesel;
extern crate serde;


use self::diesel::prelude::*;
use models::{Game, Team, Bet};
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;

#[derive(Serialize)]
pub struct GameContext {
    pub id: i32,
    pub time: i64,
    pub score1: i32,
    pub score2: i32,
    pub team1: Team,
    pub team2: Team,
    pub bet: Bet
}

impl Game {
    pub fn to_context(&self, user: i32, conn: &PgConnection) -> GameContext {

        use schema::teams::dsl::teams;
        let team1 = if self.team1.is_some() {
            teams.find(self.team1.unwrap())
                .get_result::<Team>(conn)
                .expect("Unable to get team")
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
                .expect("Unable to get team")
            } else {
                Team {
                    id: 0,
                    name: "TBD".into(),
                    abbr: "TBD".into(),
                    flag: "".into()
                }
            };

        use schema::bets::dsl::{bets, game_id, user_id};
        let mut bet = bets.filter(game_id.eq(self.id))
        .filter(user_id.eq(user))
        .first::<Bet>(conn);

        if bet.is_err() {
            bet = Ok(Bet {
                id: 0,
                user_id: 0,
                game_id: 0,
                score1: 0,
                score2: 0,
                points: 0
            })
        }

        GameContext {
            id: self.id,
            time: self.time.timestamp_millis(),
            score1: self.score1.unwrap_or(0),
            score2: self.score2.unwrap_or(0),
            team1: team1,
            team2: team2,
            bet: bet.expect("should NOT happen ")
        }
    }
}

pub fn upcoming_games(conn: &PgConnection, number: i64, offset: i32, user: i32) -> Vec<GameContext> {
    use schema::games::dsl::{games, time};
    games.filter(time.gt(Local::now()))
        .order(time.asc()).limit(number).load::<Game>(conn)
        .expect("Unable to find next games")
        .iter().map(|game| game.to_context(user, conn)).collect()
}
