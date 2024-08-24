-- Add migration script here

CREATE TABLE users (
    -- Discord id of the user
    id INT NOT NULL,
    -- purdue email used to verify the user, null if not verified
    email TEXT,
    -- points earned from sending messages (not solving challenges)
    points INT NOT NULL,
    PRIMARY KEY(id)
);