use crate::{
    db::*,
    discord::commands::ModbotCmd,
    discord::commands::{CommandOptions, PunishmentAction, PunishmentType},
};


use serenity::{
    all::{
        CreateEmbed, CreateForumPost,CreateMessage, EditThread, ResolvedValue
    },
    async_trait,
    builder::{CreateChannel, CreateInteractionResponse, CreateInteractionResponseMessage, EditRole},
    model::{application::Interaction, channel::*, id::GuildId, permissions::Permissions},
    prelude::*,
};
use tokio::sync::mpsc::Sender;

pub struct ClientHandler {
    sender: Sender<DBRequest>,
}

use regex::Regex;


impl ClientHandler {
    pub fn new(sender: Sender<DBRequest>) -> Self {
        ClientHandler { sender }
    }

    async fn create_log(ctx: &Context, guild: GuildId) -> Result<GuildChannel, SerenityError> {
        let gch = guild.channels(&ctx.http).await?;
        if let Some(channel) = gch.values().find(|channel| channel.name == "modbot-log") {
            Ok(channel.clone())
        } else {
            let logbuilder = CreateChannel::new("modbot-log")
                .kind(ChannelType::Forum)
                .topic("Moderation Profiles of Users")
                .permissions(vec![PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY
                        | Permissions::SEND_MESSAGES_IN_THREADS,
                    kind: PermissionOverwriteType::Role(guild.everyone_role()),
                }]);
            let channel = guild.create_channel(&ctx.http, logbuilder).await?;
            println!("Created mod-log channel: {}", channel.id);
            let thread = channel.create_forum_post(&ctx.http,CreateForumPost::new("Modbot Information",
                CreateMessage::new()
                                .embed(
                                CreateEmbed::new()
                                    .description(
                                        "This channel is used by Modbot to store moderation profiles of users.
                                        Each user who has received a punishment will have a dedicated thread in this forum channel.
                                        Please do not delete or modify these threads, as they are essential for Modbot's functionality.",
                                    )
                            )
                )
            ).await?;
            thread.id.edit_thread(&ctx, EditThread::new().archived(true)).await?;
            Ok(channel.clone())
        }
    }

    async fn create_punishment_notifer(ctx: &Context, guild: GuildId) -> Result<GuildChannel, SerenityError> {
        let gch = guild.channels(&ctx.http).await?;
        if let Some(channel) = gch.values().find(|channel| channel.name == "punishment-notifications") {
            Ok(channel.clone())
        } else {
            let notifybuilder = CreateChannel::new("punishment-notifications")
                .kind(ChannelType::Forum)
                .topic("Active punishment notifications for users")
                .permissions(vec![PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,
                    kind: PermissionOverwriteType::Role(guild.everyone_role()),
                }]);
            let channel = guild.create_channel(&ctx.http, notifybuilder).await?;
            println!("Created mod-notifications channel: {}", channel.id);
            Ok(channel.clone())
        }
    }

    async fn role_add(ctx: &Context, guild: GuildId, channels: &Vec<&GuildChannel>, deny: Permissions, rolename: &str) -> Result<(), SerenityError> {
        if let Some(role) = guild.roles(&ctx.http).await?.values().find(|role| role.name == rolename) {
            for channel in channels {
                let _ = channel.create_permission(&ctx.http, PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny,
                    kind: PermissionOverwriteType::Role(role.id),
                }).await;
            }
        } else {
            let newrole = guild.create_role(&ctx.http, EditRole::new()
                .name(rolename)
                .mentionable(false)
            ).await?;
            for channel in channels {
                channel.create_permission(&ctx.http, PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny,
                    kind: PermissionOverwriteType::Role(newrole.id),
                }).await?;
            }
        }
        Ok(())
    } 

    async fn permission_check(ctx: &Context, guild: GuildId) -> Result<bool, SerenityError> {
        let bot_id = ctx.cache.current_user().id;
        let member = guild.member(&ctx.http, bot_id).await?;
        if member
            .permissions(&ctx.cache)
            .map_or(false, |p| p.administrator())
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn millis(duration: String) -> Option<i64> {
        match Regex::new(r"(?i)^(\d+)([MHD])$") {
            Ok(re) => {
                if let Some(caps) = re.captures(&duration) {
                    let number = caps.get(1)?.as_str().parse::<i64>();
                    let unit = caps.get(2)?.as_str();
                    match number {
                        Ok(num) => {
                            match unit {
                                "M" => Some(num * 60),
                                "H" => Some(num * 60 * 60),
                                "D" => Some(num * 60 * 60 * 24),
                                "m" => Some(num * 60),
                                "h" => Some(num * 60 * 60),
                                "d" => Some(num * 60 * 60 * 24),
                                _ => None,
                            }
                        }
                        Err(_) => None,
                    }
                  
                    } else {
                        None
                    }
                }
            Err(_) => None,
        }
    }
}

#[async_trait]
impl EventHandler for ClientHandler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        if let Err(e) = &self
            .sender
            .send(DBRequest {
                request_type: DBRequestType::GiveContext,
                command: None,
                context: Some(ctx.clone()),
                threadlog: None,
            })
            .await
        {
            eprintln!("Error sending Give Context event {}", e);
        }
        for guild in guilds {
            match ClientHandler::permission_check(&ctx, guild).await {
                Ok(true) => {
                    match guild.channels(&ctx.http).await {
                        Ok(channels) => {
                            let guildchs = channels.values().collect::<Vec<_>>();
                            match ClientHandler::role_add(&ctx, guild, &guildchs, Permissions::all(), "Muted").await {
                                Ok(_) => (),
                                Err(e) => {
                                    eprintln!("Error applying Muted role to Guild {}: {}", guild, e);
                                    continue;
                                }
                            }        
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch channels for guild {}: {}", guild, e);
                            continue;
                        }
                    };
                    println!("Bot has permissions in connected guild {}", guild);

                    let logchannel = match ClientHandler::create_log(&ctx, guild).await {
                        Ok(log) => {
                           log
                        }
                        Err(e) => {
                            eprintln!("Failed to create or get log channel for guild {}: {}", guild, e);
                            continue;
                        }
                    };

                    match ClientHandler::create_punishment_notifer(&ctx, guild).await {
                        Ok(notifier) => {
                            if let Err(e) = &self
                                .sender
                                .send(DBRequest {
                                    request_type: DBRequestType::Build,
                                    command: None,
                                    context: None,
                                    threadlog: Some((guild, (logchannel.id, notifier.id))),
                                })
                                .await
                            {
                                eprintln!("Error sending DB Build event {}", e);
                            }

                            if let Err(e) = guild
                                .set_commands(
                                    &ctx.http,
                                    vec![
                                        ModbotCmd::Punishment.build(),
                                        ModbotCmd::FetchProfile.build(),
                                        ModbotCmd::RoleSet.build(),
                                    ],
                                )
                                .await
                            {
                                eprintln!("Failed to register commands for guild {}: {}", guild, e);
                            };
                            println!("Initialized modbot for guild {}", guild);
                        }
                        Err(e) => {
                            eprintln!("Failed to create or get punishment notification channel for guild {}: {}", guild, e);
                            continue;
                        }
                    }
                }
                Ok(false) => {
                    eprintln!("Do not have adminstrative privelges in {}", guild);
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "Error during permission check for guild, verify that right intents are given {}: {}",
                        guild, e
                    );
                    continue;
                }
            }
        }
    }

    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        let mut channels = Vec::new();
        channels.push(&channel);
        match ClientHandler::role_add(&ctx, channel.guild_id, &channels, Permissions::all(), "Muted").await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error applying Muted role to Guild {}: {}", channel.guild_id, e);
            }
        }
        
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let targetguild = match command.guild_id {
                Some(gid) => gid,
                None => {
                    command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("These command can only be used in a server/guild.")
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .expect("Failed to send response");
                    return;
                }
            };
            let invoker = (command.user).clone();
            let mut opts = CommandOptions::default();
            // Parse every current option
            for opt in command.data.options() {
                match (opt.name, &opt.value) {
                    ("user", ResolvedValue::User(u, m)) => {
                        if let Some(m) = m {
                            opts.member = Some((**m).clone());
                        }
                        opts.user = Some((**u).clone());
                    }
                    ("role", ResolvedValue::Role(r)) => {
                        opts.role = Some((**r).clone());
                    }
                    ("allow", ResolvedValue::Boolean(a)) => {
                        opts.allow = Some(*a);
                    }
                    ("add", ResolvedValue::SubCommandGroup { .. }) => {
                        opts.action = Some(PunishmentAction::Add);
                        if let ResolvedValue::SubCommandGroup(options) = &opt.value {
                            //Should only be one subcommand here
                            for subopt in options {
                                match (subopt.name, &subopt.value) {
                                    ("warn", ResolvedValue::SubCommand { .. }) => {
                                        opts.punishment = Some(PunishmentType::Warn);
                                        if let ResolvedValue::SubCommand(options) = &subopt.value {
                                            for subopt2 in options {
                                                match (subopt2.name, &subopt2.value) {
                                                    ("user", ResolvedValue::User(u, m)) => {
                                                        if let Some(m) = m {
                                                            opts.member = Some((**m).clone());
                                                        }
                                                        opts.user = Some((**u).clone());
                                                    }
                                                    ("duration", ResolvedValue::String(d)) => {
                                                        opts.duration = Some((*d).to_string());
                                                    }
                                                    ("reason", ResolvedValue::String(r)) => {
                                                        opts.reason = Some((*r).to_string());
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    ("mute", ResolvedValue::SubCommand { .. }) => {
                                        opts.punishment = Some(PunishmentType::Mute);
                                        if let ResolvedValue::SubCommand(options) = &subopt.value {
                                            for subopt2 in options {
                                                match (subopt2.name, &subopt2.value) {
                                                    ("user", ResolvedValue::User(u, m)) => {
                                                        if let Some(m) = m {
                                                            opts.member = Some((**m).clone());
                                                        }
                                                        opts.user = Some((**u).clone());
                                                    }
                                                        ("duration", ResolvedValue::String(d)) => {
                                                        opts.duration = Some((*d).to_string());
                                                    }
                                                    ("reason", ResolvedValue::String(r)) => {
                                                        opts.reason = Some((*r).to_string());
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    ("timeout", ResolvedValue::SubCommand { .. }) => {
                                        opts.punishment = Some(PunishmentType::Timeout);
                                        if let ResolvedValue::SubCommand(options) = &subopt.value {
                                            for subopt2 in options {
                                                match (subopt2.name, &subopt2.value) {
                                                    ("user", ResolvedValue::User(u, m)) => {
                                                        if let Some(m) = m {
                                                            opts.member = Some((**m).clone());
                                                        }
                                                        opts.user = Some((**u).clone());
                                                    }
                                                    ("duration", ResolvedValue::String(d)) => {
                                                        opts.duration = Some((*d).to_string());
                                                    }
                                                    ("reason", ResolvedValue::String(r)) => {
                                                        opts.reason = Some((*r).to_string());
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    ("ban", ResolvedValue::SubCommand { .. }) => {
                                        opts.punishment = Some(PunishmentType::Ban);
                                        if let ResolvedValue::SubCommand(options) = &subopt.value {
                                            for subopt2 in options {
                                                match (subopt2.name, &subopt2.value) {
                                                    ("user", ResolvedValue::User(u, m)) => {
                                                        if let Some(m) = m {
                                                            opts.member = Some((**m).clone());
                                                        }
                                                        opts.user = Some((**u).clone());
                                                    }
                                                    ("duration", ResolvedValue::String(d)) => {
                                                        opts.duration = Some((*d).to_string());
                                                    }
                                                    ("reason", ResolvedValue::String(r)) => {
                                                        opts.reason = Some((*r).to_string());
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    ("remove", ResolvedValue::SubCommand { .. }) => {
                        opts.action = Some(PunishmentAction::Remove);
                        if let ResolvedValue::SubCommand(options) = &opt.value {
                            for subopt in options {
                                match (subopt.name, &subopt.value) {
                                    ("user", ResolvedValue::User(u, m)) => {
                                        if let Some(m) = m {
                                            opts.member = Some((**m).clone());
                                        }
                                        opts.user = Some((**u).clone());
                                    }
                                    ("latest", ResolvedValue::Boolean(l)) => {
                                        opts.latest = Some(*l);
                                    }
                                    ("id", ResolvedValue::String(i)) => {
                                        opts.id = Some((*i).to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    ("edit", ResolvedValue::SubCommand { .. }) => {
                        opts.action = Some(PunishmentAction::Edit);
                        if let ResolvedValue::SubCommand(options) = &opt.value {
                            for subopt in options {
                                match (subopt.name, &subopt.value) {
                                    ("user", ResolvedValue::User(u, m)) => {
                                        if let Some(m) = m {
                                            opts.member = Some((**m).clone());
                                        }
                                        opts.user = Some((**u).clone());
                                    }
                                    ("latest", ResolvedValue::Boolean(l)) => {
                                        opts.latest = Some(*l);
                                    }
                                    ("id", ResolvedValue::String(i)) => {
                                        opts.id = Some((*i).to_string());
                                    }
                                    ("duration", ResolvedValue::String(d)) => {
                                        opts.duration = Some((*d).to_string());
                                    }
                                    ("reason", ResolvedValue::String(r)) => {
                                        opts.reason = Some((*r).to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            match command.data.name.as_str() {
                "punish" => {
                    let (user, member) = match (opts.user, opts.member) {
                        (Some(u), Some(m)) => (u, Some(m)),
                        (Some(u), None) => (u, None),
                        _ => {
                            command
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Missing user and member information.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .expect("Failed to send response");
                            return;
                        }
                    };

                    let length = match (opts.duration) {
                        (Some(duration)) => {
                            ClientHandler::millis(duration)
                        }
                        _ => None,
                    };

                    match opts.action {
                        Some(PunishmentAction::Add) => {
                            if let Some(punishment) = opts.punishment {
                                self.sender
                                    .send(DBRequest {
                                        request_type: DBRequestType::Punishment,
                                        command: Some(Command::PunishAdd {
                                            command,
                                            targetguild,
                                            target: (user, member),
                                            invoker,
                                            ptype: punishment,
                                            reason: opts.reason,
                                            length,
                                        }),
                                        context: Some(ctx),
                                        threadlog: None,
                                    })
                                    .await
                                    .unwrap_or_else(|e| {
                                        eprintln!("Error sending Punishment event {}", e);
                                    });
                            }
                        }
                        Some(PunishmentAction::Remove) => {
                            self.sender
                                .send(DBRequest {
                                    request_type: DBRequestType::Punishment,
                                    command: Some(Command::PunishRemove {
                                        command,
                                        targetguild,
                                        target: (user, member),
                                        invoker,
                                        latest: opts.latest,
                                        id: opts.id,
                                        silent: false,
                                    }),
                                    context: Some(ctx),
                                    threadlog: None,
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    eprintln!("Error sending Punishment event {}", e);
                                });
                        }
                        Some(PunishmentAction::Edit) => {
                            self.sender
                                .send(DBRequest {
                                    request_type: DBRequestType::Punishment,
                                    command: (
                                        Some(Command::PunishEdit {
                                            command,
                                            targetguild,
                                            target: (user, member),
                                            invoker,
                                            reason: opts.reason,
                                            length,
                                            latest: opts.latest,
                                            id: opts.id,
                                        })
                                    ),
                                    context: Some(ctx),
                                    threadlog: None,
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    eprintln!("Error sending Punishment event {}", e);
                                });
                        }
                        None => {
                            command
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Missing action option.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .expect("Failed to send response");
                            return;
                        }
                    }
                }
                "fetchprofile" => {
                    let (user, member) = match (opts.user, opts.member) {
                        (Some(u), Some(m)) => (u, Some(m)),
                        (Some(u), None) => (u, None),
                        _ => {
                            command
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Missing user and member information.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .expect("Failed to send response");
                            return;
                        }
                    };
                    self.sender
                        .send(DBRequest {
                            request_type: DBRequestType::FetchProfile,
                            command: (
                                Some(Command::FetchProfile { 
                                    command, 
                                    targetguild, 
                                    target: (user,member), 
                                    invoker
                                })
                            ),
                            context: Some(ctx),
                            threadlog: None,
                        })
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Error sending Fetch Profile event {}", e);
                        });
                    return;
                }
                "roleset" => {}
                _ => {
                    return;
                }
            };
        }
    }
}