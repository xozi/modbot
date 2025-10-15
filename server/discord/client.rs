use crate::{db::*, discord::commands::ModbotCmd};

use serenity::{
    async_trait, 
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, CreateChannel}, 
    futures::{io::empty, *}, 
    model::{
        application::{Command, Interaction}, 
        channel::*,
        permissions::Permissions,
        guild, 
        id::{ChannelId, CommandId, GuildId, UserId}, 
        user::CurrentUser,
    }, 
    prelude::*
};
use tokio::sync::mpsc::Sender;
pub struct ClientHandler {
    sender: Sender<DBRequest>,
}

impl ClientHandler {
    pub fn new(sender: Sender<DBRequest>) -> Self {
        ClientHandler{
            sender,
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
            Ok(true)  
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl EventHandler for ClientHandler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId> ) {
        if let Err(e) = &self.sender.send(
            DBRequest {
                request_type: DBRequestType::IntializeLog,
                usercommand: None,
                commandupdate: None,
                init: Some(DBIntialization {
                    http: Some(ctx.http.clone()),
                    guildlog: None,
                }),
            }
        ).await {
            eprintln!("Error sending HTTP access to DB {}", e);
        }
        for guild in guilds {
            match ClientHandler::permission_check(&ctx, guild).await {
                Ok(true) => {
                    println!("Bot has permissions in connected guild {}", guild);
                    match ClientHandler::create_log(&ctx, guild).await {
                        Ok(log) => {
                            println!("Log channel access for guild {} established", guild);
                            if let Err(e) = &self.sender.send(
                                DBRequest {
                                    request_type: DBRequestType::IntializeLog,
                                    usercommand: None,
                                    commandupdate: None,
                                    init: Some(DBIntialization {
                                        http: None,
                                        guildlog: Some((log, guild)),
                                    }),
                                }
                            ).await {
                                eprintln!("Error sending log access to DB {}", e);
                            }
                            
                            if let Err(e) = guild.set_commands(&ctx.http, vec![
                                ModbotCmd::Punishment.build(), 
                                ModbotCmd::FetchProfile.build()
                            ]).await {
                                eprintln!("Failed to register commands for guild {}: {}", guild, e);
                            };
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
        if let Err(e) = &self.sender.send(
            DBRequest {
                request_type: DBRequestType::Build,
                usercommand: None,
                commandupdate: None,
                init: None,
            }
        ).await {
            eprintln!("Error sending log access to DB {}", e);
        }
    }
}

