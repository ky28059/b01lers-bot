use serenity::all::EditChannel;

use crate::ARCHIVED_CTF_CATEGORY_ID;
use crate::commands::{CmdContext, Error, has_perms};
use crate::commands::competition::get_competition_from_ctx;

/// Archives the current competition channel.
#[poise::command(slash_command)]
pub async fn archive(ctx: CmdContext<'_>) -> Result<(), Error> {
    if !has_perms(&ctx).await {
        return Err(anyhow::anyhow!(
            "You do not have permissions to archive a competition."
        ));
    }

    // Ensure command is being run within a competition channel, and the competition is not already archived.
    let competition = get_competition_from_ctx(&ctx).await?;

    let mut channel = competition
        .channel_id
        .to_channel(ctx)
        .await?
        .guild()
        .expect("You are not inside a competition channel.");

    if channel.parent_id.is_some_and(|id| id == ARCHIVED_CTF_CATEGORY_ID) {
        return Err(anyhow::anyhow!(
            "This competition is already archived!"
        ))
    }

    // Move the channel to the archived category.
    channel
        .edit(ctx, EditChannel::new().category(ARCHIVED_CTF_CATEGORY_ID))
        .await?;

    ctx.say(format!("Archived **{}**.", competition.name))
        .await?;

    Ok(())
}
