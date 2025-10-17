use polodb_core::{
    bson::doc, options, CollectionT, Database, IndexModel, IndexOptions
};
use crate::discord::{embed::profembed,commands::PunishmentType};
use tokio::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use serenity::{
    all::{CommandDataOption, CommandInteraction, User, ResolvedValue, Role, PartialMember}, builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage}, model::{ channel::{Embed, GuildChannel}, guild, id::{GuildId, RoleId, UserId}, Timestamp}
};
use std::collections::{BTreeMap, BTreeSet};

pub struct DBHandler {
    database: BTreeMap<GuildId, GuildDB>,
    guildlog: BTreeMap<GuildId, GuildChannel>,
    roleperms: BTreeMap<GuildId, BTreeSet<RolePermission>>,
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
                    if let ((Some(cmd),_), Some(ctx)) = (request.command, request.context) {
                        if let Some(userprofile) = self.get_profile(cmd.target.0.id.get() as i64, &cmd.targetguild).await {
                            cmd.command.create_response(&ctx.http, CreateInteractionResponse::Message(  
                                CreateInteractionResponseMessage::new()    
                                    .embed(profembed(&cmd.invoker,&cmd.target, userprofile).await)
                                    .ephemeral(true)
                            )).await.expect("Failed to send response");
                        }                    
                    }
                },
                DBRequestType::AddPunishment => {
                    if let ((Some(cmd),_), Some(ctx)) = (request.command, request.context) {
                        if let Some(userprofile) = self.get_profile(cmd.target.0.id.get() as i64, &cmd.targetguild).await {

                        }                    
                    }
                },
                DBRequestType::RemovePunishment => {
                    if let ((Some(cmd),_), Some(ctx)) = (request.command, request.context) {
                        if let Some(userprofile) = self.get_profile(cmd.target.0.id.get() as i64, &cmd.targetguild).await {

                        }                    
                    }
                },
                DBRequestType::EditPunishment => {
                    if let ((Some(cmd),_), Some(ctx)) = (request.command, request.context) {
                        if let Some(userprofile) = self.get_profile(cmd.target.0.id.get() as i64, &cmd.targetguild).await {

                        }                    
                    }
                },
                DBRequestType::TemporaryComplete => {
                    
                },
                DBRequestType::CommandPermissionUpdate => {
                    if let ((_,Some(cmd)), Some(ctx)) = (request.command, request.context) {
                        if let Some(roleperm) = self.get_roleperm(cmd.target.id.get() as i64, &cmd.targetguild).await {

                        }
                    }
                }
            }
        }
    }

    async fn get_profile(&self, userid: i64, guildid: &GuildId) -> Option<Profile> {
        if let Some(guilddb) = self.database.get(guildid) {
            match guilddb.profilecol.find_one(doc! { "user_id": userid}) {
                Ok(Some(profile)) => return Some(profile),
                Ok(None) => {
                    if let Err(e) = guilddb.profilecol.insert_one(Profile::new(userid)) {
                        eprintln!("Error creating new profile in Profile Query: {}", e);
                        return None;
                    }
                    return Some(Profile::new(userid));
                },
                Err(e) => {
                    eprintln!("Error retrieving profile in Profile Query: {}", e);
                    return None;
                }
            };
        } else {
            eprintln!("No database found for queried guild in Profile Query");
        }
        return None;
    }

    async fn get_roleperm(&self, roleid: i64, guildid: &GuildId) -> Option<RolePermission> {
        if let Some(guilddb) = self.database.get(guildid) {
            match guilddb.rolecol.find_one(doc! { "role_id": roleid}) {
                Ok(Some(role)) => return Some(role),
                Ok(None) => {
                    if let Err(e) = guilddb.rolecol.insert_one(RolePermission::new(roleid)) {
                        eprintln!("Error creating new role in Role Query: {}", e);
                        return None;
                    }
                    return Some(RolePermission::new(roleid));
                },
                Err(e) => {
                    eprintln!("Error retrieving role in Role Query: {}", e);
                    return None;
                }
            };
        } else {
            eprintln!("No database found for queried guild in Role Query");
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
    pub command: (Option<UserCommand>, Option<RoleCommand>),
    pub context: Option<serenity::prelude::Context>,
    pub guildlog: Option<(GuildChannel, GuildId)>,
}

pub struct UserCommand{
    pub command: CommandInteraction,
    pub targetguild: GuildId,
    pub target: (User, Option<PartialMember>),
    pub invoker: User,
    pub punishment: (Option<Punishment>,Option<Adjust>),
}
pub struct RoleCommand{
    pub command: CommandInteraction,
    pub targetguild: GuildId,
    pub target: Role,
    pub invoker: User,
    pub allow: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermission {
    pub role_id: i64,
    pub allow: bool,
}

impl RolePermission {
    pub fn new(role_id: i64) -> Self {
        RolePermission {
            role_id,
            allow: false,
        }
    }
}

/*
The embed will have the details for the profile, 
a member query should be done if possible. Separate information that 
is not able to be retried from the member API request.
Maybe consider keeping struct elements of the embed for easy recreation?
*/ 

#[derive(Debug, Serialize, Deserialize)]
// We will need to convert UserId to i64 for BSON queries
pub struct Profile {
    user_id: i64,
    pub punishments: BTreeMap<i64,PunishmentRecord>, //id, Record
    negdur: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PunishmentRecord {
    pub punishment: PunishmentType,
    pub reason: Option<String>,
    pub punished_for: (Timestamp,Timestamp), //Start, End
    pub moderator: i64,
}

impl Profile {
    pub fn new(user_id: i64) -> Self {
        Profile {
            user_id,
            punishments: BTreeMap::new(),
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
pub struct Punishment {
    pub ptype: PunishmentType,
    pub reason: Option<String>,
    pub length: Option<u64>,
}

pub struct Adjust {
    pub reason: Option<String>,
    pub length: Option<u64>,
    pub latest: Option<bool>,
    pub id: Option<i64>,
}