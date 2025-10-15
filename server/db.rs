use std::sync::Arc;
use arrayvec::ArrayVec;
use polodb_core::{
    Database,
    CollectionT,
    IndexModel,
    IndexOptions,
    bson::doc,
};
use tokio::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use serenity::{all::Guild, model::{
    channel::{Embed, GuildChannel}, id::{GuildId, RoleId, UserId}
}};
use std::collections::BTreeMap;

pub struct DBHandler {
    database: BTreeMap<GuildId, GuildDB>,
    guildlog: BTreeMap<GuildId, GuildChannel>,
    roleperms: BTreeMap<GuildId, RolePermission>,
    httprequest: Option<Arc<serenity::http::Http>>,
    receiver: Receiver<DBRequest>,
}

impl DBHandler {
    pub fn new(receiver: Receiver<DBRequest>) -> Self {
        DBHandler {
            database: BTreeMap::new(),
            guildlog: BTreeMap::new(),
            roleperms: BTreeMap::new(),
            httprequest: None,
            receiver,
        }
    }
    pub async fn process_requests(&mut self) {
        while let Some(request) = self.receiver.recv().await {
            match request.request_type {
                DBRequestType::IntializeHTTP => {
                    if let Some(init) = request.init {
                        if let Some(http) = init.http {
                            self.httprequest = Some(http);
                        }
                    }
                },
                DBRequestType::IntializeLog => {
                    if let Some(init) = request.init {
                        if let Some((channel, guild_id)) = init.guildlog {
                            self.guildlog.insert(guild_id,channel);
                        }
                    }
                },
                DBRequestType::Build => {
                        if let Err(e) = std::fs::create_dir_all("server/databases") {
                            eprintln!("Failed to create database folder: {}", e);
                            return; 
                        }
                        for (guild, _) in &self.guildlog {
                            // Generate a database path for each guild
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
                                            "duration": 1,
                                        },
                                        options: None,
                                    }) {
                                        eprintln!("Failed to create index for Profile collection in guild {}: {}", guild, e);
                                    }

                                    if let Err(e) = tempcol.create_index(IndexModel {
                                        keys: doc!{ 
                                            "duration": 1,
                                        },
                                        options: None,
                                    }) {
                                        eprintln!("Failed to create index for Temporary collection in guild {}: {}", guild, e);
                                    }
                                    
                                    self.database.insert(*guild, 
                                        GuildDB {
                                        db,
                                        profilecol,
                                        tempcol,
                                        rolecol
                                    });

                                    println!("Initialized database for guild {}", guild);
                                }
                                Err(e) => {
                                    eprintln!("Failed to initialize database for guild {}: {}", guild, e);
                                }
                            }
                        }
                },
                DBRequestType::FetchProfile => {

                },
                DBRequestType::AddPunishment => {

                },
                DBRequestType::RemovePunishment => {

                },
                DBRequestType::EditPunishment => {

                },
                DBRequestType::TemporaryComplete => {

                },
                DBRequestType::CommandPermissionUpdate => {
 
                }
            }
        }
    }
}

pub enum DBRequestType {
    IntializeHTTP,
    IntializeLog,
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
    pub usercommand: Option<UserCommand>,
    pub commandupdate: Option<RolePermission>,
    pub init: Option<DBIntialization>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermission {
    pub role_id: RoleId,
    pub allow: bool,
}

pub struct UserCommand {
    pub user: Option<UserId>,
    pub punishment: Option<String>,
    pub duration: Option<u64>,
}

pub struct DBIntialization {
    pub http: Option<Arc<serenity::http::Http>>,
    pub guildlog: Option<(GuildChannel, GuildId)>,
}

/*
The embed will have the details for the profile, 
a member query should be done if possible. Separate information that 
is not able to be retried from the member API request.
Maybe consider keeping struct elements of the embed for easy recreation?
*/ 

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    user_id: UserId,
    profile: Embed,
    duration: u64,
}


#[derive(Debug, Serialize, Deserialize)]
struct Temporary {
    user_id: UserId,
    punishment: String,
    duration: u64,
}
