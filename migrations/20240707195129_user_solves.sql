-- Add migration script here

CREATE TABLE user_solves (
    user_id INT NOT NULL,
    solve_Id INT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(solve_id) REFERENCES solves(id)
)