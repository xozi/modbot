use crate::{db::DB_Handler, discord::commandtypes::ModbotCmd};

use serenity::{
    async_trait, 
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, CreateChannel}, 
    futures::{io::empty, *}, 
    model::{
        application::{Command, Interaction}, 
        channel::*, 
        gateway::Ready, 
        permissions::Permissions,
        guild, 
        id::{ChannelId, CommandId, GuildId, UserId}, 
        user::CurrentUser,
    }, 
    prelude::*
};
pub struct ClientHandler {
    db: DB_Handler
}

impl ClientHandler {
    pub fn new(dhandle: DB_Handler) -> Self {
        ClientHandler{
            db : dhandle
        }
    }

    async fn create_log(ctx: &Context, guild: GuildId) -> Result<GuildChannel,SerenityError> {
        let gch = guild.channels(&ctx.http).await?;  
        if let Some(channel) = gch.values().find(|channel| channel.name == "modbot-log") {
                Ok(channel.clone())
            } else {
                let logbuilder = CreateChannel::new("modbot-log")
                .kind(ChannelType::Forum)
                .topic("Moderation Profiles of Users")
                .permissions(vec![
                            PermissionOverwrite {
                                allow: Permissions::empty(), 
                                deny: Permissions::VIEW_CHANNEL 
                                    | Permissions::SEND_MESSAGES 
                                    | Permissions::READ_MESSAGE_HISTORY, 
                                kind: PermissionOverwriteType::Role(guild.everyone_role()), 
                            }]);
                let channel = guild.create_channel(&ctx.http, logbuilder).await?;
                println!("Created mod-log channel: {}", channel.id);
                Ok(channel.clone())
            }
    }
    

    async fn permission_check(ctx: &Context, guild: GuildId) -> Result<bool, SerenityError> {
        let bot_id = ctx.cache.current_user().id;
        let member = guild.member(&ctx.http, bot_id).await?;
        if member.permissions(&ctx.cache).map_or(false, |p| p.administrator()) {
                println!("Registering commands for guild: {}", guild);
                let commands = vec![
                    ModbotCmd::Punishment.build(), 
                    ModbotCmd::FetchProfile.build()
                ];  
                guild.set_commands(&ctx.http, commands).await?;
                Ok(true)  
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl EventHandler for ClientHandler {
     async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId> ) {
        for guild in guilds {
            match ClientHandler::permission_check(&ctx, guild).await {
                Ok(true) => {
                    match ClientHandler::create_log(&ctx, guild).await {
                        Ok(log_channel) => {

                        },
                        Err(e) => {
                            continue;
                        }
                    }
                },
                Ok(false) => {
                    eprintln!("Do not have adminstrative privelges in {}", guild);
                    continue
                }
                Err(e) => {
                    eprintln!("Error during permission check for guild {}: {}", guild, e);
                    continue
                }
            }
        };
    }
}

