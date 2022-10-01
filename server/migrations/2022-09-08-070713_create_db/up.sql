-- Your SQL goes here
CREATE TABLE game_state (
    id TEXT PRIMARY KEY NOT NULL ,
    board TEXT,
    user_1 TEXT,
    user_2 TEXT,
    winner TEXT,
    last_user_id TEXT,
    history TEXT
);

CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL ,
    user_name TEXT NOT NULL
);

INSERT INTO user VALUES ('1', 'user1');
INSERT INTO user VALUES ('2', 'user2');