extern crate chrono;
use self::chrono::{DateTime, Utc};

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub time: Option<DateTime<Utc>>,
    pub score1: i32,
    pub score2: i32,
}
