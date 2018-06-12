table! {
    games (id) {
        id -> Int4,
        time -> Timestamptz,
        score1 -> Nullable<Int4>,
        score2 -> Nullable<Int4>,
        finished -> Bool,
        team1 -> Nullable<Int4>,
        team2 -> Nullable<Int4>,
    }
}

table! {
    teams (id) {
        id -> Int4,
        name -> Varchar,
        abbr -> Bpchar,
        flag -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        passwd -> Varchar,
        token -> Nullable<Varchar>,
        tokenexpireat -> Nullable<Timestamptz>,
        isadmin -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    games,
    teams,
    users,
);
