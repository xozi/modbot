mod commandtypes;
mod handler;
use serenity::{
    async_trait, 
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage}, 
    futures::{io::empty, *}, model::{
    application::{Command, Interaction}, 
    channel::*, 
    gateway::Ready, 
    guild, 
    id::{ChannelId, CommandId, GuildId, UserId}, 
    user::CurrentUser,
    }, prelude::*
};

struct Handler;
use std::env;

use crate::commandtypes::ModbotCmd;

#[async_trait]
impl EventHandler for Handler {
     async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        for guildref in ready.guilds {
            let guild = guildref.id; 
                 if let Ok(member) = guild.current_user_member(&ctx).await {
                if member.permissions(&ctx.cache).map_or(false, |p| p.administrator()) {
                    println!("Registering commands for guild: {}", guild);
                    let commands = vec![
                        ModbotCmd::Punishment.build(), 
                        ModbotCmd::FetchProfile.build()
                    ];
                    if let Err(e) = guild.set_commands(&ctx.http, commands).await {
                        eprintln!("Failed to register commands for guild {}: {}", guild, e);
                    }
                    } else {
                        println!("Bot is not an administrator in guild: {}", guild);
                    }     
                }
            };
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES 
    | GatewayIntents::GUILD_MEMBERS 
    | GatewayIntents::GUILD_MODERATION
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILDS   
    | GatewayIntents::DIRECT_MESSAGES 
    | GatewayIntents::AUTO_MODERATION_EXECUTION;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
