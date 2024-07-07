# Running

To set up the bot, first install the `sqlx` CLI with 
```bash
cargo install sqlx-cli
```
Then, create a `.env` file defining 2 variables `DISCORD_TOKEN` and `DATABASE_URL` like so:
```env
DATABASE_URL="..."
DISCORD_TOKEN="..."
```
Run
```bash
sqlx database create
```
to create a new local competition database.

Use `cargo run` to run the bot.
