-- Add migration script here

CREATE TABLE competition (
    -- Id of the competition channel
    channel_id INT NOT NULL,
    -- Name of the ctf
    name TEXT NOT NULL,
    -- bitfield specifying which of the bad ctf bingos have been achieved
    bingo INT NOT NULL,
    PRIMARY KEY(channel_id)
);
