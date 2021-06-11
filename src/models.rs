extern crate chrono;

use self::chrono::{NaiveDateTime};

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub time: NaiveDateTime,
    pub score1: Option<i32>,
    pub score2: Option<i32>,
    pub finished: bool,
    pub team1: Option<i32>,
    pub team2: Option<i32>
}

#[derive(Queryable, Serialize)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub abbr: String,
    pub flag: String,
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: String,
    pub token: Option<String>,
    pub tokenExpireAt: Option<NaiveDateTime>,
    pub isAdmin: bool,
    pub points: i32
}

#[derive(Queryable, Serialize)]
pub struct Bet {
    pub id: i32,
    pub user_id: i32,
    pub game_id: i32,
    pub score1: i32,
    pub score2: i32,
    pub points: i32
}
