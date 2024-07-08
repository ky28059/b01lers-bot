use anyhow::Context;
use serenity::all::{Builder, ChannelFlags, ChannelType, CreateChannel, CreateEmbed, CreateForumTag, CreateMessage, EditChannel, EditThread, EmojiId, ForumEmoji, ReactionType};
use serenity::builder::CreateForumPost;

use crate::{B01LERS_GUILD_ID, CTF_CATEGORY_ID};
use crate::db::{BingoSquare, Competition};

use super::{CmdContext, Error, has_perms};

/// Creates a new ctf competition channel.
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

    // Defer response because channel setup may take longer than 3 seconds
    ctx.defer().await?;

    if !has_perms(&ctx).await {
        return Err(anyhow::anyhow!(
            "You do not have permissions to create a competition."
        ));
    }

    // TODO: prettier error
    // Create forum channel
    let creds_str = &format!("**{name}**\n{url}\n\n**Username**: {username}\n**Password**: {password}");
    let mut forum = CreateChannel::new(&name)
        .category(CTF_CATEGORY_ID)
        .position(0)
        .kind(ChannelType::Forum)
        .default_reaction_emoji(ForumEmoji::Id(EmojiId::new(1257157847612129351))) // :blobsalute:
        .topic(creds_str) // Post guidelines for forum channel
        .execute(ctx, B01LERS_GUILD_ID)
        .await?;

    // Add category and solved tags to forum channel
    let tags = vec![
        CreateForumTag::new("web").emoji(ReactionType::Unicode("🌐".to_string())),
        CreateForumTag::new("crypto").emoji(ReactionType::Unicode("🧮".to_string())),
        CreateForumTag::new("pwn").emoji(ReactionType::Unicode("💥".to_string())),
        CreateForumTag::new("rev").emoji(ReactionType::Unicode("🛠️".to_string())),
        CreateForumTag::new("misc").emoji(ReactionType::Unicode("⚙️".to_string())),

        CreateForumTag::new("forensics").emoji(ReactionType::Unicode("🔍".to_string())),
        CreateForumTag::new("osint").emoji(ReactionType::Unicode("🕵️".to_string())),
        CreateForumTag::new("blockchain").emoji(ReactionType::Unicode("⛓".to_string())),

        CreateForumTag::new("unsolved").emoji(ReactionType::Unicode("❌".to_string())),
        CreateForumTag::new("solved").emoji(ReactionType::Unicode("✅".to_string())),
    ];
    forum.edit(ctx, EditChannel::new().available_tags(tags)).await?;

    // Create post with credentials
    let credentials_embed = CreateEmbed::new()
        .color(0xc22026)
        .title(&format!("{name} credentials"))
        .description(url)
        .field("Username", username, false)
        .field("Password", password, false);

    let mut creds_channel = forum.create_forum_post(ctx, CreateForumPost::new("Credentials + general discussion", CreateMessage::new().add_embed(credentials_embed)))
        .await?;

    // Pin credentials / general discussion post
    creds_channel.edit_thread(ctx, EditThread::new()
        .flags(ChannelFlags::PINNED)
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

pub async fn get_competition_id_from_ctx(ctx: &CmdContext<'_>) -> Result<ChannelId, Error> {
    let Some(thread_channel) = ctx.guild_channel().await else {
        Err(anyhow::anyhow!("You are not inside a competition channel."))?
    };

    // For a forum channel, the competition channel will be the command channel's parent.
    let Some(channel_id) = thread_channel.parent_id else {
        Err(anyhow::anyhow!("You are not inside a competition channel."))?
    };

    Ok(channel_id)
}

/// Gets the competition in the channel the command was invoked from.
pub async fn get_competition_from_ctx(ctx: &CmdContext<'_>) -> Result<Competition, Error> {
    let channel_id = get_competition_id_from_ctx(ctx).await?;

    let competition = ctx
        .data()
        .db
        .get_competition(channel_id)
        .await
        .with_context(|| "You are not inside a competition channel.")?;

    Ok(competition)
}