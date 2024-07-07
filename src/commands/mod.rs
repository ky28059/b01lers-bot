use serenity::all::Member;

use crate::{db::DbContext, B01LERS_GUILD_ID, OFFICER_ROLE};

pub mod competition;
pub mod bingo;
pub mod archive;
pub mod challenge;

pub struct CommandContext {
    db: DbContext,
}

impl CommandContext {
    pub fn new(db: DbContext) -> Self {
        CommandContext { db }
    }
}

type Error = anyhow::Error;
type CmdContext<'a> = poise::Context<'a, CommandContext, Error>;

pub async fn say_error(ctx: &CmdContext<'_>, message: &str) -> Result<(), Error> {
    ctx.say(&format!("Error: {message}")).await?;

    Ok(())
}

pub async fn is_officer(ctx: &CmdContext<'_>, member: &Member) -> bool {
    // TODO: method chain version?
    let roles = match ctx.cache().guild(B01LERS_GUILD_ID).map(|g| g.roles.clone()) {
        Some(roles) => roles,
        None => {
            println!("cache miss");
            let Ok(roles) = B01LERS_GUILD_ID.roles(ctx).await else {
                return false;
            };
            roles
        }
    };

    member
        .roles
        .iter()
        .map(|role_id| match roles.get(role_id) {
            Some(role) => role.name == OFFICER_ROLE,
            None => false,
        })
        .fold(false, |a, b| a || b)
}

pub async fn has_perms(ctx: &CmdContext<'_>) -> bool {
    match ctx.author_member().await {
        // make sure a privileged command is being used on b01lers server
        Some(member) => {
            member.guild_id == B01LERS_GUILD_ID && is_officer(ctx, member.as_ref()).await
        }
        None => false,
    }
}
