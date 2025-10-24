use crate::discord::{commands::PunishmentType, embed::profembed};
use polodb_core::{CollectionT, Database, IndexModel, IndexOptions, bson::doc, options};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{CommandDataOption, CommandInteraction, PartialMember, ResolvedValue, Role, User},
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{channel::GuildChannel, id::{ChannelId, GuildId}, user, Timestamp},
};
use std::{collections::{BTreeMap, BTreeSet}, time::UNIX_EPOCH};
use tokio::sync::mpsc::Receiver;


pub struct DBHandler {
    database: BTreeMap<GuildId, GuildDB>,
    guildlog: BTreeMap<GuildId, ChannelId>,
    context: Option<serenity::prelude::Context>,
    receiver: Receiver<DBRequest>,
}

impl DBHandler {
    pub fn new(receiver: Receiver<DBRequest>) -> Self {
        DBHandler {
            database: BTreeMap::new(),
            guildlog: BTreeMap::new(),
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
                }
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
                                    keys: doc! {
                                        "negdur": 1,
                                    },
                                    options: None,
                                }) {
                                    eprintln!(
                                        "Failed to create index for Profile collection in guild {}: {}",
                                        guild, e
                                    );
                                }

                                if let Err(e) = tempcol.create_index(IndexModel {
                                    keys: doc! {
                                        "negdur": 1,
                                    },
                                    options: None,
                                }) {
                                    eprintln!(
                                        "Failed to create index for Temporary collection in guild {}: {}",
                                        guild, e
                                    );
                                }

                                self.database.insert(
                                    guild,
                                    GuildDB {
                                        db,
                                        profilecol,
                                        tempcol,
                                        rolecol,
                                    },
                                );
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to initialize database for guild {}: {}",
                                    guild, e
                                );
                            }
                        }
                    }
                }
                DBRequestType::FetchProfile => {
                    if let (Some(cmd), Some(ctx)) = (request.command, request.context) {
                        match cmd {
                            Command::FetchProfile {command, 
                                                    target, 
                                                    targetguild,
                                                    invoker, .. } => {  
                                if let Some(userprofile) = self
                                    .get_profile(target.0.id.get() as i64, &targetguild)
                                    .await
                                {
                                    command
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .embed(
                                                        profembed(
                                                            &invoker,
                                                            &target,
                                                            userprofile,
                                                        )
                                                        .await,
                                                    )
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await
                                        .expect("Failed to send response");
                                }
                            }
                            _ => {
                            }
                        }
                     
                    }
                }
                DBRequestType::Punishment => {
                    if let (Some(cmd), Some(ctx)) = (request.command, request.context) {
                        match cmd {
                            Command::PunishAdd {command, 
                                                target, 
                                                targetguild,
                                                invoker,
                                                ptype,
                                                reason,
                                                length, .. } => {  
                                if let Some(mut userprofile) = self
                                    .get_profile(target.0.id.get() as i64, &targetguild)
                                    .await
                                {
                                    let end = Timestamp::now().unix_timestamp() + length.unwrap_or(-Timestamp::now().unix_timestamp());
                                        userprofile.add_punishment(PunishmentRecord {
                                            punishment: ptype.clone(),
                                            reason: reason.clone(),
                                            punished_for: (Timestamp::now().unix_timestamp(), end),
                                            moderator: invoker.id.get() as i64,
                                        });
                                    self.update_profile(&userprofile, &targetguild).await;
                                    command
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content(format!("Added {:?} punishment to <@{}>.", ptype, target.0.id))
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await
                                        .expect("Failed to send response");
                                    }
                                }
                            Command::PunishEdit {command, 
                                                target, 
                                                targetguild,
                                                invoker,
                                                id,
                                                latest,
                                                length,
                                                reason, .. } => {  
                                if let Some(mut userprofile) = self
                                    .get_profile(target.0.id.get() as i64, &targetguild)
                                    .await
                                {
                                    userprofile.edit_punishment(id, latest, length, reason);
                                    self.update_profile(&userprofile, &targetguild).await;
                                    command
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content(format!("Edited punishment for <@{}>.", target.0.id))
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await
                                        .expect("Failed to send response");
                                }
                            }
                            Command::PunishRemove {command, 
                                                    target, 
                                                    targetguild,
                                                    invoker,
                                                    id,
                                                    latest, .. } => {  
                                if let Some(mut userprofile) = self
                                    .get_profile(target.0.id.get() as i64, &targetguild)
                                    .await
                                {
                                    userprofile.remove_punishment(id, latest);
                                    self.update_profile(&userprofile, &targetguild).await;
                                    command.
                                        create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content(format!("Removed punishment for <@{}>.", target.0.id))
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await
                                        .expect("Failed to send response");
                                }
                            }
                            _ => { 
                            }
                        }
                    }
                }
                DBRequestType::TemporaryComplete => {
                    //
                }
                DBRequestType::CommandPermissionUpdate => {
                    if let (Some(cmd), Some(ctx)) = (request.command, request.context) {
                        match cmd {
                            Command::RoleAdjust {command, 
                                                target, 
                                                targetguild,
                                                invoker, .. } => {  
                                if let Some(roleperm) = self
                                    .get_roleperm(target.id.get() as i64, &targetguild)
                                    .await 
                                {

                                }
                            }
                            _ => {}
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
                }
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
    
    async fn update_profile(&self, profile: &Profile, guildid: &GuildId) {
        if let Some(guilddb) = self.database.get(guildid) {
            if let Ok(bson_profile) = polodb_core::bson::to_bson(profile) {
                guilddb.profilecol.update_one(doc! { "user_id": profile.user_id }, doc! { "$set": bson_profile })
                    .expect("Failed to update profile in Profile Update");
            } else {
                eprintln!("Error converting profile to BSON in Profile Update");
            }
        } else {
            eprintln!("No database found for queried guild in Profile Update");
        }
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
                }
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
    Punishment,
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
    pub command: Option<Command>,
    pub context: Option<serenity::prelude::Context>,
    pub guildlog: Option<(ChannelId, GuildId)>,
}

pub enum Command {
    PunishEdit {
        command: CommandInteraction,
        targetguild: GuildId,
        target: (User, Option<PartialMember>),
        invoker: User,
        reason: Option<String>,
        length: Option<i64>,
        latest: Option<bool>,
        id: Option<String>,
    },
    PunishRemove {
        command: CommandInteraction,
        targetguild: GuildId,
        target: (User, Option<PartialMember>),
        invoker: User,
        latest: Option<bool>,
        id: Option<String>,
    },

    PunishAdd {
        command: CommandInteraction,
        targetguild: GuildId,
        target: (User, Option<PartialMember>),
        invoker: User,
        ptype: PunishmentType,
        reason: Option<String>,
        length: Option<i64>,
    },

    RoleAdjust {
        command: CommandInteraction,
        targetguild: GuildId,
        target: Role,
        invoker: User,
        allow: bool,
    },
    
    FetchProfile {
        command: CommandInteraction,
        targetguild: GuildId,
        target: (User, Option<PartialMember>),
        invoker: User,
    }
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
    punishment_thread: Option<ChannelId>,
    pub punishments: BTreeMap<String, PunishmentRecord>, //id, Record
    negdur: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PunishmentRecord {
    pub punishment: PunishmentType,
    pub reason: Option<String>,
    pub punished_for: (i64, i64), //Start, End
    pub moderator: i64,
}

impl Profile {
    pub fn new(user_id: i64) -> Self {
        Profile {
            user_id,
            punishment_thread: None,
            punishments: BTreeMap::new(),
            negdur: !Timestamp::now().unix_timestamp(),
        }
    }

    pub fn add_punishment(&mut self, record: PunishmentRecord) {
        let id = match self.punishments.keys().last() {
            Some(last_id) => last_id.parse::<u16>().unwrap_or(0) + 1,
            None => 1,
        };
        self.punishments.insert(id.to_string(), record);
    }

    pub fn remove_punishment(&mut self, id: Option<String>, latest: Option<bool>) {
        match (id, latest) {
            (Some(pid), _) => {
                self.punishments.remove(&pid);
            }
            (None, Some(true)) => {
                if let Some(last_id) = self.punishments.keys().last().cloned() {
                    self.punishments.remove(&last_id);
                }
            }
            _ => {}
        }
    }

    pub fn edit_punishment(&mut self, id: Option<String>, latest: Option<bool>, length: Option<i64>, reason: Option<String>) {
         match (id, latest) {
            (Some(pid), _) => {
                if let Some(record) = self.punishments.get_mut(&pid) {
                    if let Some(reason) = reason {
                        record.reason = Some(reason);
                    }
                    if let Some(length) = length {
                        let start = record.punished_for.0;
                        record.punished_for.1 = start + length;
                    }
                }
            }
            (None, Some(true)) => {
                if let Some((_, record)) = self.punishments.iter_mut().next_back() {
                    if let Some(reason) = reason {
                        record.reason = Some(reason);
                    }
                    if let Some(length) = length {
                        let start = record.punished_for.0;
                        record.punished_for.1 = start + length;
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Temporary {
    user_id: i64,
    punishment: PunishmentRecord,
    negdur: i64,
}

impl Temporary {
    pub fn new(user_id: i64, punishment: PunishmentRecord, duration: i64) -> Self {
        Temporary {
            user_id,
            punishment,
            negdur: !duration,
        }
    }
}




