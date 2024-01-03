-- Your SQL goes here
CREATE TABLE friendships (
    friendship_id SERIAL NOT NULL PRIMARY KEY,
    user_one VARCHAR NOT NULL REFERENCES users(user_id),
    user_two VARCHAR NOT NUll REFERENCES users(user_id),
    status VARCHAR NOT NULL
)