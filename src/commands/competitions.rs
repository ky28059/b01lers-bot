use anyhow::Context;
use poise::CreateReply;
use serenity::all::{Builder, CreateAttachment, CreateChannel, CreateEmbed, CreateMessage};

use crate::{db::{BingoSquare, Competition}, B01LERS_GUILD_ID, CTF_CATEGORY_ID};
use super::{has_perms, CmdContext, Error};

/// Creates a new ctf competition thread
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
        return Err(anyhow::anyhow!("You do not have permissions to create a competition"));
    }

    // TODO: prettier error
    // Create channel
    let channel = CreateChannel::new(&name)
        .category(CTF_CATEGORY_ID)
        .position(0)
        .topic(&format!("Channel for {name}"))
        .execute(ctx, B01LERS_GUILD_ID).await?;

    // Send message with credentials
    let embed = CreateEmbed::new()
        .title(&format!("{name} credentials"))
        .description(&format!("ctf url: {url}"))
        .field("Username", username, false)
        .field("Password", password, false);

    let message = CreateMessage::new()
        .add_embed(embed);

    channel.send_message(&ctx, message).await?;

    let competition = Competition {
        channel_id: channel.id,
        name: name.clone(),
        bingo: BingoSquare::Free.into(),
    };
    ctx.data().db.create_competiton(competition).await?;

    ctx.say(format!("Created channel for {name}: {channel}")).await?;

    Ok(())
}

/// Gets the competition based on the channel the command was invoked from
async fn get_competition_from_channel(ctx: &CmdContext<'_>) -> Result<Competition, Error> {
    let channel_id = ctx.channel_id();

    let competition = ctx.data().db.get_competition(channel_id)
        .await.with_context(|| "Not in a competition channel")?;

    Ok(competition)
}

async fn send_bingo_image(ctx: &CmdContext<'_>, image: &[u8]) -> Result<(), Error> {
    let attachment = CreateAttachment::bytes(image, "bingo_squares.png");

    let reply = CreateReply::default()
        .attachment(attachment);

    ctx.send(reply).await?;

    Ok(())
}

// This can never be called, just needed for bingo subcommands
#[poise::command(slash_command, subcommands("add", "status", "remove"))]
pub async fn bingo(_ctx: CmdContext<'_>) -> Result<(), Error> { Ok(()) }

/// Displays the current status of the badctf bingo squares
#[poise::command(slash_command)]
pub async fn status(
    ctx: CmdContext<'_>,
) -> Result<(), Error> {
    let competition = get_competition_from_channel(&ctx).await?;

    send_bingo_image(&ctx, &competition.get_bingo_picture_png_bytes()?).await?;

    Ok(())
}

/// Checks off a badctf bingo square
#[poise::command(slash_command)]
pub async fn add(
    ctx: CmdContext<'_>,
    #[description = "The bingo square to check off"] square: BingoSquare,
) -> Result<(), Error> {
    let mut competition = get_competition_from_channel(&ctx).await?;

    competition.bingo |= square;

    send_bingo_image(&ctx, &competition.get_bingo_picture_png_bytes()?).await?;

    ctx.data().db.update_competition(competition).await?;

    Ok(())
}

/// Unmarks a badctf bingo square
#[poise::command(slash_command)]
pub async fn remove(
    ctx: CmdContext<'_>,
    #[description = "The bingo square to uncheck"] square: BingoSquare,
) -> Result<(), Error> {
    let mut competition = get_competition_from_channel(&ctx).await?;

    competition.bingo.remove(square);

    send_bingo_image(&ctx, &competition.get_bingo_picture_png_bytes()?).await?;

    ctx.data().db.update_competition(competition).await?;

    Ok(())
}