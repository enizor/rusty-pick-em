table! {
    games (id) {
        id -> Int4,
        time -> Timestamptz,
        score1 -> Nullable<Int4>,
        score2 -> Nullable<Int4>,
        finished -> Bool,
    }
}

table! {
    teams (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        abbr -> Nullable<Bpchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    games,
    teams,
);
