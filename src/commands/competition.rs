use anyhow::Context;
use serenity::all::{Builder, ChannelFlags, ChannelId, ChannelType, CreateChannel, CreateEmbed, CreateForumTag, CreateMessage, EditThread, EmojiId, ForumEmoji, ForumTag};
use serenity::builder::CreateForumPost;

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
    let forum = CreateChannel::new(&name)
        .category(CTF_CATEGORY_ID)
        .position(0)
        .kind(ChannelType::Forum)
        .default_reaction_emoji(ForumEmoji::Id(EmojiId::new(1257157847612129351))) // :blobsalute:
        .topic(&format!("Channel for {name}; please check pinned post for shared credentials."))
        .execute(ctx, B01LERS_GUILD_ID)
        .await?;

    // Create post with credentials
    let credentials_embed = CreateEmbed::new()
        .color(0xc22026)
        .title(&format!("{name} credentials"))
        .description(&format!("ctf url: {url}"))
        .field("Username", username, false)
        .field("Password", password, false);

    let message = CreateMessage::new().add_embed(credentials_embed);
    let mut creds_channel = forum.create_forum_post(ctx, CreateForumPost::new("Login credentials", message))
        .await?;

    // Pin and lock credentials post
    creds_channel.edit_thread(ctx, EditThread::new()
        .flags(ChannelFlags::PINNED)
        .locked(true)
    ).await?;

    let competition = Competition {
        channel_id: forum.id,
        name: name.clone(),
        bingo: BingoSquare::Free.into(),
    };
    ctx.data().db.create_competition(competition).await?;

    ctx.say(format!("Created channel for **{name}**: {forum}"))
        .await?;

    Ok(())
}

/// Gets the competition in the channel the command was invoked from.
pub async fn get_competition_from_ctx(ctx: &CmdContext<'_>) -> Result<Competition, Error> {
    let channel_id = ctx.channel_id();
    get_competition_from_id(ctx, channel_id).await
}

/// Gets the competition in the given channel.
pub async fn get_competition_from_id(ctx: &CmdContext<'_>, channel_id: ChannelId) -> Result<Competition, Error> {
    let competition = ctx
        .data()
        .db
        .get_competition(channel_id)
        .await
        .with_context(|| "Not in a competition channel.")?;

    Ok(competition)
}
