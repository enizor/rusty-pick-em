CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL,
    passwd VARCHAR(256) NOT NULL,
    token VARCHAR(250),
    tokenExpireAt DATETIME,
    isAdmin BOOLEAN NOT NULL DEFAULT FALSE
)
