-- Add migration script here

CREATE TABLE solves (
    id INTEGER PRIMARY KEY,
    competition_id INT NOT NULL,
    -- message id of approval message
    approval_message_id INT NOT NULL,
    challenge_name TEXT NOT NULL,
    -- Indicates if challenge is like, pwn, web, crypto, etc
    challenge_type INT NOT NULL,
    flag TEXT NOT NULL,
    -- If this solve has been approved, declined, or pending approval
    -- 0 is pending
    -- 1 is approved
    -- 2 is declined
    approved INT NOT NULL,
    FOREIGN KEY(competition_id) REFERENCES competition(channel_id)
);