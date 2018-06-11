extern crate chrono;
use self::chrono::{DateTime, Utc};

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub time: DateTime<Utc>,
    pub score1: Option<i32>,
    pub score2: Option<i32>,
    pub finished: bool,
    pub team1: Option<i32>,
    pub team2: Option<i32>
}

#[derive(Queryable)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub abbr: String,
    pub flag: String,
}
