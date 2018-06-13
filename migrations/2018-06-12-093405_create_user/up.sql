CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    passwd VARCHAR(256) NOT NULL,
    token VARCHAR(250),
    tokenExpireAt TIMESTAMP WITH TIME ZONE,
    isAdmin BOOLEAN NOT NULL DEFAULT FALSE
)
