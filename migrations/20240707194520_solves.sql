-- Add migration script here

CREATE TABLE solves (
    id INTEGER PRIMARY KEY,
    challenge_id INT NOT NULL,
    -- message id of approval message
    approval_message_id INT NOT NULL,
    flag TEXT NOT NULL,
    -- Approval status of this solve
    -- 0: pending
    -- 1: accepted
    -- 2: declined
    approval_status INT NOT NULL,
    FOREIGN KEY(challenge_id) REFERENCES challenges(id)
);