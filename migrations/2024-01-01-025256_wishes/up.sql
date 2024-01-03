-- Your SQL goes here
CREATE TABLE wishes (
    wish_id SERIAL PRIMARY KEY,
    description VARCHAR NOT NULL,
    access_level VARCHAR NOT NULL,
    user_id VARCHAR NOT NULL references users(user_id)
)