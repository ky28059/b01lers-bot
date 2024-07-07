use serenity::all::EditChannel;

use crate::ARCHIVED_CTF_CATEGORY_ID;
use crate::commands::{CmdContext, Error, has_perms};
use crate::commands::competition::get_competition_from_channel;

/// Archives the current competition channel.
#[poise::command(slash_command)]
pub async fn archive(ctx: CmdContext<'_>) -> Result<(), Error> {
    if !has_perms(&ctx).await {
        return Err(anyhow::anyhow!(
            "You do not have permissions to archive a competition."
        ));
    }

    // Ensure we're in a competition channel, then move it to the archived category.
    get_competition_from_channel(&ctx).await?;
    let mut channel = ctx
        .guild_channel()
        .await
        .expect("You are not inside a guild");

    if channel.parent_id.is_some_and(|id| id == ARCHIVED_CTF_CATEGORY_ID) {
        return Err(anyhow::anyhow!(
            "This competition is already archived!"
        ))
    }

    channel
        .edit(ctx, EditChannel::new().category(ARCHIVED_CTF_CATEGORY_ID))
        .await?;

    Ok(())
}
