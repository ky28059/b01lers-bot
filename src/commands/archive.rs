use serenity::all::EditChannel;

use crate::ARCHIVED_CTF_CATEGORY_ID;
use crate::commands::{CmdContext, Error, has_perms};
use crate::commands::competition::get_competition_from_id;

/// Archives the current competition channel.
#[poise::command(slash_command)]
pub async fn archive(ctx: CmdContext<'_>) -> Result<(), Error> {
    if !has_perms(&ctx).await {
        return Err(anyhow::anyhow!(
            "You do not have permissions to archive a competition."
        ));
    }

    // For a forum channel, the competition channel will be the command channel's parent.
    let mut channel = ctx
        .guild_channel()
        .await
        .expect("You are not inside a competition channel.")
        .parent_id
        .expect("You are not inside a competition channel.")
        .to_channel(ctx)
        .await?
        .guild()
        .expect("You are not inside a competition channel.");

    // Ensure command is being run within a competition channel.
    let competition = get_competition_from_id(&ctx, channel.id).await?;

    if channel.parent_id.is_some_and(|id| id == ARCHIVED_CTF_CATEGORY_ID) {
        return Err(anyhow::anyhow!(
            "This competition is already archived!"
        ))
    }

    // Move the channel to the archived category.
    channel
        .edit(ctx, EditChannel::new().category(ARCHIVED_CTF_CATEGORY_ID))
        .await?;

    ctx.say(format!("Archived channel for **{}**.", competition.name))
        .await?;

    Ok(())
}
