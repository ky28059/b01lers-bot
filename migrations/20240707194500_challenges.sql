-- Add migration script here

CREATE TABLE challenges (
    -- Also the id of the discord channel for this challenge
    id INTEGER PRIMARY KEY,
    competition_id INT NOT NULL,
    name TEXT NOT NULL,
    -- Indicates if challenge is like, pwn, web, crypto, etc
    category INT NOT NULL,
    FOREIGN KEY(competition_id) REFERENCES competition(channel_id)
);