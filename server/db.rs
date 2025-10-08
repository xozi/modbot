use std::{collections::BTreeSet, sync::Arc};
use polodb_core::Database;
use tokio::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use serenity::{all::Role, model::{
    channel::{Embed, GuildChannel}, id::{GuildId, RoleId, UserId}
}};
use std::collections::BTreeMap;

pub struct DBHandler {
    database: BTreeMap<GuildId, Database>,
    guildlog: BTreeMap<GuildId, GuildChannel>,
    roleperms: BTreeMap<GuildId, BTreeMap<RoleId, bool>>,
    httprequest: Option<Arc<serenity::http::Http>>,
    receiver: Receiver<DBRequest>,
    intialized: bool,
}

impl DBHandler {
    pub fn new(receiver: Receiver<DBRequest>) -> Self {
        DBHandler {
            database: BTreeMap::new(),
            guildlog: BTreeMap::new(),
            roleperms: BTreeMap::new(),
            httprequest: None,
            receiver,
            intialized: false,
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
                    if !self.intialized {
                        if let Err(e) = std::fs::create_dir_all("server/databases") {
                            eprintln!("Failed to create database folder: {}", e);
                            return; 
                        }
                        for (guild, _) in &self.guildlog {
                            // Generate a database path for each guild
                            let db_path = format!("server/databases/{}.db", guild);
                            match Database::open_file(&db_path) {
                                Ok(db) => {
                                    println!("Database initialized for guild {}", db_path);
                                    self.database.insert(*guild, db);
                                }
                                Err(e) => {
                                    eprintln!("Failed to initialize database for guild {}: {}", guild, e);
                                }
                            }
                        }
                        self.intialized = true;
                    }
                },
                DBRequestType::FetchProfile => {
                    if self.intialized {

                    }
                },
                DBRequestType::AddPunishment => {
                    if self.intialized {

                    }
                },
                DBRequestType::RemovePunishment => {
                    if self.intialized {

                    }
                },
                DBRequestType::EditPunishment => {
                    if self.intialized {

                    }
                },
                DBRequestType::TemporaryComplete => {
                    if self.intialized {

                    }
                },
                DBRequestType::CommandPermissionUpdate => {
                    if self.intialized {

                    }
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

pub struct DBRequest {
    pub request_type: DBRequestType,
    pub usercommand: Option<UserCommand>,
    pub commandupdate: Option<(String, RoleId)>,
    pub init: Option<DBIntialization>,
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

#[derive(Debug, Serialize, Deserialize)]
struct RolePermission {
    role_id: RoleId,
    allow: bool,
}