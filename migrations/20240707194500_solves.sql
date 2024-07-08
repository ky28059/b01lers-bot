-- Add migration script here

CREATE TABLE solves (
    id INT NOT NULL,
    competition_id INT NOT NULL,
    challenge_name TEXT NOT NULL,
    -- Indicates if challenge is like, pwn, web, crypto, etc
    challenge_type INT NOT NULL,
    flag TEXT NOT NULL,
    -- If this solve has been approved yet
    approved BOOL NOT NULL,
    PRIMARY KEY(id),
    FOREIGN KEY(competition_id) REFERENCES competition(channel_id)
);