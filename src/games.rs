extern crate chrono;
extern crate diesel;
extern crate serde;


use self::diesel::prelude::*;
use models::{Game, Team, Bet, User};
use chrono::prelude::*;
use chrono::{Utc};
use diesel::pg::PgConnection;

#[derive(Serialize)]
pub struct GameDetails {
    pub id: i32,
    pub time: i64,
    pub score1: i32,
    pub score2: i32,
    pub team1: Team,
    pub team2: Team,
    pub bet: Bet, // bet from the concerned user
    pub finished: bool,
    pub started: bool,
    pub bets: Vec<BetDetails> // ordered by total points per user
}

#[derive(Queryable, Serialize)]
pub struct BetDetails {
    pub username: String,
    pub score1: i32,
    pub score2: i32,
    pub bet_points: i32,
    pub user_points: i32
}

impl Game {
    pub fn to_context(&self, user: i32, conn: &PgConnection) -> GameDetails {

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

        use schema::bets::dsl::*;
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

        use schema::users;
        let bets_vec = bets.filter(game_id.eq(self.id))
        .inner_join(users::dsl::users)
        .select((users::dsl::name, score1, score2, points, users::dsl::points))
        .order(users::dsl::points.desc())
        .load::<BetDetails>(conn).expect("unable to load bets");

        GameDetails {
            id: self.id,
            time: self.time.timestamp_millis(),
            score1: self.score1.unwrap_or(0),
            score2: self.score2.unwrap_or(0),
            team1: team1,
            team2: team2,
            finished: self.is_finished(),
            started: self.is_started(),
            bets: bets_vec,
            bet: bet.expect("should NOT happen ")
        }
    }

    fn is_finished(&self) -> bool {
        self.score1.is_some()
    }

    fn is_started(&self) -> bool {
        self.time < Utc::now()
    }

    pub fn update_bets(&self, conn: &PgConnection) {
        use schema::bets::dsl::*;
        let bets_vec = bets.filter(game_id.eq(self.id))
        .load::<Bet>(&*conn)
        .expect("Unable to load bets");

        for bet in bets_vec.iter() {
            let points_won = self.points_for_bet(&bet);
            diesel::update(bets.find(bet.id))
            .set(points.eq(points_won))
            .get_result::<Bet>(&*conn)
            .expect("error updating bet's points");
        }

        update_users_points(conn);
    }

    fn points_for_bet(&self, ref bet: &Bet) -> i32 {
        if self.score1.is_none() || self.score2.is_none() {
            return 0
        }
        let s1 = self.score1.unwrap();
        let s2 = self.score2.unwrap();
        if s1 == bet.score1 && s2 == bet.score2 {
            3
        } else if s1 == s2 {
            if bet.score1 == bet.score2 {
                1
            } else {
                0
            }
        } else if s1 > s2 {
            if bet.score1 > bet.score2 {
                1
            } else {
                0
            }
        } else {
            if bet.score1 < bet.score2 {
                1
            } else {
                0
            }
        }
    }
}

pub fn upcoming_games(conn: &PgConnection, number: i64, offset: i32, user: i32) -> Vec<GameDetails> {
    use schema::games::dsl::{games, time};
    games.filter(time.gt(Local::now()))
        .order(time.asc()).limit(number).load::<Game>(conn)
        .expect("Unable to find next games")
        .iter().map(|game| game.to_context(user, conn)).collect()
}

pub fn get_game(id: i32, conn: &PgConnection) -> Option<Game> {
    use schema::games::dsl::{games};
    games.find(id)
        .first::<Game>(conn).ok()
}

pub fn update_users_points(conn: &PgConnection) {
        use schema::users::dsl::*;
        let users_vec = users.select(id).load::<i32>(&*conn)
        .expect("Unable to load users");

        for current_id in users_vec.iter() {
            let mut total_points = 0;
            {
                use schema::bets::dsl::*;
                use diesel::dsl::sum;
                total_points = bets.select(sum(points))
                .filter(user_id.eq(current_id))
                .first::<Option<i64>>(&*conn)
                .unwrap_or(Some(0))
                .unwrap();
            }
            diesel::update(users.find(current_id))
            .set(points.eq(total_points as i32))
            .get_result::<User>(&*conn)
            .expect("error updating user's points");
        }
}
