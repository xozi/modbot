use core::arch;

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
    builder::{CreateChannel, CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{application::Interaction, channel::*, id::GuildId, permissions::Permissions},
    prelude::*,
};
use tokio::sync::mpsc::Sender;

pub struct ClientHandler {
    sender: Sender<DBRequest>,
}



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

    fn millis(unit: &str, period: u64) -> Option<u64> {
        match unit {
            "M" => Some(period * 1000 * 60),
            "H" => Some(period * 1000 * 60 * 60),
            "D" => Some(period * 1000 * 60 * 60 * 24),
            _ => None,
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
                command: (None, None),
                context: Some(ctx.clone()),
                guildlog: None,
            })
            .await
        {
            eprintln!("Error sending Give Context event {}", e);
        }
        for guild in guilds {
            match ClientHandler::permission_check(&ctx, guild).await {
                Ok(true) => {
                    println!("Bot has permissions in connected guild {}", guild);
                    match ClientHandler::create_log(&ctx, guild).await {
                        Ok(log) => {
                            if let Err(e) = &self
                                .sender
                                .send(DBRequest {
                                    request_type: DBRequestType::Build,
                                    command: (None, None),
                                    context: None,
                                    guildlog: Some((log, guild)),
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
                            eprintln!("Failed to create or get log channel for guild {}: {}", guild, e);
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
                                                    ("duration", ResolvedValue::Integer(d)) => {
                                                        opts.duration = Some(*d);
                                                    }
                                                    ("units", ResolvedValue::String(u)) => {
                                                        opts.units = Some((*u).to_string());
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
                                                    ("duration", ResolvedValue::Integer(d)) => {
                                                        opts.duration = Some(*d);
                                                    }
                                                    ("units", ResolvedValue::String(u)) => {
                                                        opts.units = Some((*u).to_string());
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
                                                    ("duration", ResolvedValue::Integer(d)) => {
                                                        opts.duration = Some(*d);
                                                    }
                                                    ("units", ResolvedValue::String(u)) => {
                                                        opts.units = Some((*u).to_string());
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
                                                    ("duration", ResolvedValue::Integer(d)) => {
                                                        opts.duration = Some(*d);
                                                    }
                                                    ("units", ResolvedValue::String(u)) => {
                                                        opts.units = Some((*u).to_string());
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
                    }
                    ("edit", ResolvedValue::SubCommand { .. }) => {
                        opts.action = Some(PunishmentAction::Edit);
                    }
                    ("duration", ResolvedValue::Integer(d)) => {
                        opts.duration = Some(*d);
                    }
                    ("units", ResolvedValue::String(u)) => {
                        opts.units = Some((*u).to_string());
                    }
                    ("reason", ResolvedValue::String(r)) => {
                        opts.reason = Some((*r).to_string());
                    }
                    ("id", ResolvedValue::Integer(i)) => {
                        opts.id = Some(*i);
                    }
                    ("latest", ResolvedValue::Boolean(l)) => {
                        opts.latest = Some(*l);
                    }
                    ("allow", ResolvedValue::Boolean(a)) => {
                        opts.allow = Some(*a);
                    }
                    _ => {}
                }
            }
            match command.data.name.as_str() {
                "punishment" => {
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

                    let length = match (opts.duration, opts.units) {
                        (Some(duration), Some(units)) => {
                            ClientHandler::millis(&units, duration as u64)
                        }
                        (Some(_), None) => {
                            command
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Units provided without duration")
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .expect("Failed to send response");
                            return;
                        }
                        (None, Some(_)) => {
                            command
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Duration provided without units")
                                            .ephemeral(true),
                                    ),
                                )
                                .await
                                .expect("Failed to send response");
                            return;
                        }
                        _ => None,
                    };

                    match opts.action {
                        Some(PunishmentAction::Add) => {
                            if let Some(punishment) = opts.punishment {
                                self.sender
                                    .send(DBRequest {
                                        request_type: DBRequestType::AddPunishment,
                                        command: (
                                            Some(UserCommand {
                                                command,
                                                targetguild,
                                                target: (user, member),
                                                invoker,
                                                punishment: (
                                                    Some(Punishment {
                                                        ptype: punishment,
                                                        reason: opts.reason,
                                                        length,
                                                    }),
                                                    None,
                                                ),
                                            }),
                                            None,
                                        ),
                                        context: Some(ctx),
                                        guildlog: None,
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
                                    request_type: DBRequestType::RemovePunishment,
                                    command: (
                                        Some(UserCommand {
                                            command,
                                            targetguild,
                                            target: (user, member),
                                            invoker,
                                            punishment: (
                                                None,
                                                Some(Adjust {
                                                    reason: opts.reason,
                                                    length,
                                                    latest: opts.latest,
                                                    id: opts.id,
                                                }),
                                            ),
                                        }),
                                        None,
                                    ),
                                    context: Some(ctx),
                                    guildlog: None,
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    eprintln!("Error sending Punishment event {}", e);
                                });
                        }
                        Some(PunishmentAction::Edit) => {
                            self.sender
                                .send(DBRequest {
                                    request_type: DBRequestType::EditPunishment,
                                    command: (
                                        Some(UserCommand {
                                            command,
                                            targetguild,
                                            target: (user, member),
                                            invoker,
                                            punishment: (
                                                None,
                                                Some(Adjust {
                                                    reason: opts.reason,
                                                    length,
                                                    latest: opts.latest,
                                                    id: opts.id,
                                                }),
                                            ),
                                        }),
                                        None,
                                    ),
                                    context: Some(ctx),
                                    guildlog: None,
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
                                Some(UserCommand {
                                    command,
                                    targetguild,
                                    target: (user, member),
                                    invoker,
                                    punishment: (None, None),
                                }),
                                None,
                            ),
                            context: Some(ctx),
                            guildlog: None,
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
