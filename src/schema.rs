table! {
    bets (id) {
        id -> Integer,
        user_id -> Integer,
        game_id -> Integer,
        score1 -> Integer,
        score2 -> Integer,
        points -> Integer,
    }
}

table! {
    games (id) {
        id -> Integer,
        time -> Timestamp,
        score1 -> Nullable<Integer>,
        score2 -> Nullable<Integer>,
        finished -> Bool,
        team1 -> Nullable<Integer>,
        team2 -> Nullable<Integer>,
    }
}

table! {
    teams (id) {
        id -> Integer,
        name -> Text,
        abbr -> Text,
        flag -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        passwd -> Text,
        token -> Nullable<Text>,
        tokenExpireAt -> Nullable<Timestamp>,
        isAdmin -> Bool,
        points -> Integer,
    }
}

joinable!(bets -> games (game_id));
joinable!(bets -> users (user_id));

allow_tables_to_appear_in_same_query!(bets, games, teams, users,);
