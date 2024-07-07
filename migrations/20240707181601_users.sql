-- Add migration script here

CREATE TABLE users (
    -- Discord id of the user
    id INT NOT NULL,
    -- purdue email used to verify the user
    email TEXT NOT NULL,
    PRIMARY KEY(id)
);