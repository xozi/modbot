use polodb_core::{
    Database,
    CollectionT,
    IndexModel,
    IndexOptions,
    bson::doc,
};
use crate::discord::embed::profembed;
use tokio::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use serenity::{
    all::{CommandInteraction, PartialMember, User}, builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage}, model::{ channel::{Embed, GuildChannel}, guild, id::{GuildId, RoleId, UserId}, Timestamp}
};
use std::collections::BTreeMap;

pub struct DBHandler {
    database: BTreeMap<GuildId, GuildDB>,
    guildlog: BTreeMap<GuildId, GuildChannel>,
    roleperms: BTreeMap<GuildId, RolePermission>,
    context: Option<serenity::prelude::Context>,
    receiver: Receiver<DBRequest>,
}

impl DBHandler {
    pub fn new(receiver: Receiver<DBRequest>) -> Self {
        DBHandler {
            database: BTreeMap::new(),
            guildlog: BTreeMap::new(),
            roleperms: BTreeMap::new(),
            context: None,
            receiver,
        }
    }
    pub async fn process_requests(&mut self) {
        while let Some(request) = self.receiver.recv().await {
            match request.request_type {
                DBRequestType::GiveContext => {
                    if let Some(context) = request.context {
                        self.context = Some(context);
                    }
                },
                DBRequestType::Build => {
                    if let Some((channel, guild)) = request.guildlog {
                        self.guildlog.insert(guild, channel);
                        
                        if let Err(e) = std::fs::create_dir_all("server/databases") {
                            eprintln!("Failed to create database folder: {}", e);
                            continue; 
                        }

                        let db_path = format!("server/databases/{}.db", guild);
                        // Hang can occur here if improper drop, application closing needs to be handled eventually.
                        match Database::open_path(&db_path) {
                            Ok(db) => {
                                let profilecol = db.collection::<Profile>("Profile");
                                let tempcol = db.collection::<Temporary>("Temporary");
                                let rolecol = db.collection::<RolePermission>("RolePermission");
                                
                                // Store with Bitwise ! duration to get the most recent punishment at the top
                                // ASC is the only working order (1)
                                if let Err(e) = profilecol.create_index(IndexModel {
                                    keys: doc!{ 
                                        "negdur": 1,
                                    },
                                    options: None,
                                }) {
                                    eprintln!("Failed to create index for Profile collection in guild {}: {}", guild, e);
                                }

                                if let Err(e) = tempcol.create_index(IndexModel {
                                    keys: doc!{ 
                                        "negdur": 1,
                                    },
                                    options: None,
                                }) {
                                    eprintln!("Failed to create index for Temporary collection in guild {}: {}", guild, e);
                                }
                                
                                self.database.insert(guild, 
                                    GuildDB {
                                    db,
                                    profilecol,
                                    tempcol,
                                    rolecol
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to initialize database for guild {}: {}", guild, e);
                            }
                        }
                    }
                },
                DBRequestType::FetchProfile => {
                    if let (Some(cmd), Some(ctx)) = (request.command, request.context) {
                        let (profile, user, member) = match self.profile_query(&cmd).await {
                            Some((p, u, m)) => (p, u, m),
                            None => {
                                eprintln!("Error retrieving profile for user in guild {}", cmd.guild_id.unwrap_or_default());
                                continue;
                            }
                        };
                        cmd.create_response(&ctx.http, CreateInteractionResponse::Message(  
                            CreateInteractionResponseMessage::new()    
                                .embed(profembed(&cmd.user,user, member, &ctx.cache).await)
                                .ephemeral(true)  
                        )).await.expect("Failed to create response");
                    }
                },
                DBRequestType::AddPunishment => {
                    if let Some(command) = request.command {
                        
                    }
                },
                DBRequestType::RemovePunishment => {
                    if let Some(command) = request.command {
                        
                    }
                },
                DBRequestType::EditPunishment => {
                    if let Some(command) = request.command {
                        
                    }
                },
                DBRequestType::TemporaryComplete => {

                },
                DBRequestType::CommandPermissionUpdate => {
                     if let Some(command) = request.command {
                        
                    }
                }
            }
        }
    }

    async fn profile_query(&self, cmd: &CommandInteraction) -> Option<(Profile, User, PartialMember)> {
        if let Some(guildDB) = self.database.get(&cmd.guild_id.unwrap_or_default()) {
            let user = match cmd.data.resolved.users.values().next().cloned() {
                Some(u) => u,
                None => return None,
            };
            let member = match cmd.data.resolved.members.values().next().cloned() {
                Some(m) => m,
                None => return None,
            };
            let userid = i64::from(user.id);
            match guildDB.profilecol.find_one(doc! { "user_id": userid}) {
                Ok(Some(profile)) => return Some((profile, user, member)),
                Ok(None) => {
                    if let Err(e) = guildDB.profilecol.insert_one(Profile::new(userid)) {
                        eprintln!("Error creating new profile in Fetch Profile: {}", e);
                    }
                    return Some((Profile::new(userid), user, member));
                },
                Err(e) => {
                    eprintln!("Error retrieving profile in Fetch Profile: {}", e);
                    return None;
            }
        };
        } else {
            eprintln!("No database found for queried guild in Fetch Profile");
        }
        return None;
    }
}

pub enum DBRequestType {
    GiveContext,
    Build,
    FetchProfile,
    AddPunishment,
    RemovePunishment,
    EditPunishment,
    TemporaryComplete,
    CommandPermissionUpdate,
}

struct GuildDB {
    db: Database,
    profilecol: polodb_core::Collection<Profile>,
    tempcol: polodb_core::Collection<Temporary>,
    rolecol: polodb_core::Collection<RolePermission>,
}

pub struct DBRequest {
    pub request_type: DBRequestType,
    pub command: Option<CommandInteraction>,
    pub context: Option<serenity::prelude::Context>,
    pub guildlog: Option<(GuildChannel, GuildId)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermission {
    pub role_id: RoleId,
    pub allow: bool,
}

/*
The embed will have the details for the profile, 
a member query should be done if possible. Separate information that 
is not able to be retried from the member API request.
Maybe consider keeping struct elements of the embed for easy recreation?
*/ 

#[derive(Debug, Serialize, Deserialize)]
// We will need to convert UserId to i64 for BSON queries
struct Profile {
    user_id: i64,
    punishments: Vec<Punishment>,
    negdur: i64,
}

impl Profile {
    pub fn new(user_id: i64) -> Self {
        Profile {
            user_id,
            punishments: Vec::new(),
            negdur: !Timestamp::now().unix_timestamp(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Temporary {
    user_id: i64,
    punishment: Punishment,
    negdur: i64,
}

impl Temporary {
    pub fn new(user_id: i64, punishment: Punishment, duration: i64) -> Self {
        Temporary {
            user_id,
            punishment,
            negdur: !duration,
        }
    }
}
 
#[derive(Debug, Serialize, Deserialize)]
pub enum PunishmentType {
    Warn,
    Mute,
    Kick,
    Ban,
    Timeout,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Punishment {
    pub ptype: PunishmentType,
    pub reason: String,
    pub moderator: UserId,
    pub length: (Timestamp, Timestamp)
}


