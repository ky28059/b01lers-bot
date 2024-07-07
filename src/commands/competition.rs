use anyhow::Context;
use serenity::all::{Builder, CreateChannel, CreateEmbed, CreateMessage};

use crate::{B01LERS_GUILD_ID, CTF_CATEGORY_ID};
use crate::db::{BingoSquare, Competition};

use super::{CmdContext, Error, has_perms};

/// Creates a new ctf competition thread.
#[poise::command(slash_command)]
pub async fn competition(
    ctx: CmdContext<'_>,
    #[description = "Name of the ctf"] name: String,
    #[description = "Url of ctf website"] url: String,
    //#[description = "Description of the ctf"] description: Option<String>,
    #[description = "Team username"] username: String,
    #[description = "Team password or login url"] password: String,
) -> Result<(), Error> {
    // TODO: figure out how to get all channels in a category, so we can check duplicate names

    if !has_perms(&ctx).await {
        return Err(anyhow::anyhow!(
            "You do not have permissions to create a competition."
        ));
    }

    // TODO: prettier error
    // Create channel
    let channel = CreateChannel::new(&name)
        .category(CTF_CATEGORY_ID)
        .position(0)
        .topic(&format!("Channel for {name}; please check pinned message for shared credentials."))
        .execute(ctx, B01LERS_GUILD_ID)
        .await?;

    // Send message with credentials
    let embed = CreateEmbed::new()
        .title(&format!("{name} credentials"))
        .description(&format!("ctf url: {url}"))
        .field("Username", username, false)
        .field("Password", password, false);

    let message = CreateMessage::new().add_embed(embed);

    channel.send_message(&ctx, message).await?;

    let competition = Competition {
        channel_id: channel.id,
        name: name.clone(),
        bingo: BingoSquare::Free.into(),
    };
    ctx.data().db.create_competition(competition).await?;

    ctx.say(format!("Created channel for **{name}**: {channel}"))
        .await?;

    Ok(())
}

/// Gets the competition based on the channel the command was invoked from.
pub async fn get_competition_from_channel(ctx: &CmdContext<'_>) -> Result<Competition, Error> {
    let channel_id = ctx.channel_id();

    let competition = ctx
        .data()
        .db
        .get_competition(channel_id)
        .await
        .with_context(|| "Not in a competition channel.")?;

    Ok(competition)
}
