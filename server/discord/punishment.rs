use crate::{db::PunishmentRecord, discord::commands::PunishmentType};

use serenity::{
    all::User,
    model::id::GuildId, prelude::*,
};



pub async fn apply_punishment(ctx: &Context, guild: GuildId, record: &PunishmentRecord, target: &User) -> Result<(), SerenityError> {
    match record.punishment {

        PunishmentType::Ban => {
            guild.ban(&ctx.http, target, 0).await?;
        }
        PunishmentType::Mute => {
            if let Some(role) = guild.roles(&ctx.http).await?.values().find(|role| role.name == "Muted") {
                guild.member(&ctx.http, target).await?
                    .add_role(&ctx.http, role.id)
                    .await?;
            } else {
                return Err(SerenityError::Other("Mute role not found."));
            }
        }
        PunishmentType::Timeout => {
            guild.member(&ctx.http, target).await?
                .disable_communication_until_datetime(&ctx.http, record.punished_for.1).await?;
        }
        PunishmentType::Warn => {
        
        }
    }
    Ok(())
}

pub async fn remove_punishment(ctx: &Context, guild: GuildId, record: &PunishmentRecord, target: &User) -> Result<(), SerenityError> {
    match record.punishment {
        PunishmentType::Ban => {
            guild.unban(&ctx.http, target).await?;
        }
        PunishmentType::Mute => {
            if let Some(role) = guild.roles(&ctx.http).await?.values().find(|role| role.name == "Muted") {
                guild.member(&ctx.http, target).await?
                    .remove_role(&ctx.http, role.id)
                    .await?;
            } else {
                return Err(SerenityError::Other("Mute role not found."));
            }
        }
        PunishmentType::Timeout => {
            guild.member(&ctx.http, target).await?
                .enable_communication(&ctx.http).await?;
        }
        PunishmentType::Warn => {
        
        }
    }
    Ok(())
}