use std::collections::HashMap;

use anyhow::Context;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, aead::OsRng};
use serenity::all::{Member, Role, RoleId, UserId};
use tracing::info;

use crate::{db::DbContext, email::EmailClient, B01LERS_GUILD_ID, OFFICER_ROLE};

pub mod competition;
pub mod bingo;
pub mod archive;
pub mod solve;
pub mod verify;
pub mod challenge;
pub mod stats;

pub struct CommandContext {
    db: DbContext,
    verify_token_cipher: XChaCha20Poly1305,
    email_client: EmailClient,
}

impl CommandContext {
    pub fn new(db: DbContext, email_client: EmailClient) -> Self {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        CommandContext {
            db,
            verify_token_cipher: XChaCha20Poly1305::new(&key),
            email_client,
        }
    }
}

type Error = anyhow::Error;
type CmdContext<'a> = poise::Context<'a, CommandContext, Error>;

pub async fn get_all_roles(ctx: &CmdContext<'_>) -> anyhow::Result<HashMap<RoleId, Role>> {
    // TODO: method chain version?
    match ctx.cache().guild(B01LERS_GUILD_ID).map(|g| g.roles.clone()) {
        Some(roles) => Ok(roles),
        None => {
            info!("role cache miss");
            let roles = B01LERS_GUILD_ID.roles(ctx).await?;
            Ok(roles)
        }
    }
}

pub async fn is_officer(ctx: &CmdContext<'_>, member: &Member) -> bool {
    let Ok(roles) = get_all_roles(ctx).await else {
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

pub async fn role_id_for_role_name(ctx: &CmdContext<'_>, role_name: &str) -> anyhow::Result<Option<RoleId>> {
    let roles = get_all_roles(ctx).await?;

    for (role_id, role) in roles.iter() {
        if role.name == role_name {
            return Ok(Some(*role_id))
        }
    }

    Ok(None)
}

/// Adds the given role name to the user in b01lers discord server
pub async fn add_role_to_user(ctx: &CmdContext<'_>, user_id: UserId, role_name: &str) -> anyhow::Result<()> {
    let member_role_id = role_id_for_role_name(&ctx, role_name).await?
        .ok_or_else(|| anyhow::anyhow!("Role `{role_name}` does not exist"))?;

    let guild_member_id = B01LERS_GUILD_ID.member(ctx, user_id).await
        .with_context(|| format!("Could not add role `{role_name}`, you are not in the b01lers discord server"))?;

    guild_member_id.add_role(ctx, member_role_id).await
        .with_context(|| format!("Could not add role `{role_name}`"))?;

    Ok(())
}