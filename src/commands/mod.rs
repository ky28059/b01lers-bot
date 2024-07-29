use chacha20poly1305::{KeyInit, XChaCha20Poly1305, aead::OsRng};
use serenity::all::Member;

use crate::{db::DbContext, B01LERS_GUILD_ID, OFFICER_ROLE};

pub mod competition;
pub mod bingo;
pub mod archive;
pub mod solve;
pub mod verify;

pub struct CommandContext {
    pub db: DbContext,
    verify_token_cipher: XChaCha20Poly1305,
}

impl CommandContext {
    pub fn new(db: DbContext) -> Self {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        CommandContext {
            db,
            verify_token_cipher: XChaCha20Poly1305::new(&key),
        }
    }
}

type Error = anyhow::Error;
type CmdContext<'a> = poise::Context<'a, CommandContext, Error>;

pub async fn say_error(ctx: &CmdContext<'_>, message: &str) -> Result<(), Error> {
    ctx.say(&format!("Error: {message}")).await?;

    Ok(())
}

pub async fn is_officer(ctx: &CmdContext<'_>, member: &Member) -> bool {
    let Ok(roles) = B01LERS_GUILD_ID.roles(ctx).await else {
        return false;
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
