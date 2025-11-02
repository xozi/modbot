use crate::discord::{commands::PunishmentType, embed::profembed, punishment::*, thread::*};
use polodb_core::{CollectionT, Database, IndexModel, bson::doc};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{CommandInteraction, PartialMember, Role, User},
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{Timestamp, id::{ChannelId, GuildId}},     prelude::*,
};
use std::collections::BTreeMap;
use tokio::{task::JoinHandle, time::{sleep, Duration}, sync::mpsc::{Sender,Receiver}};


pub struct DBHandler {
    database: BTreeMap<GuildId, GuildDB>,
    threadlog: BTreeMap<GuildId, (ChannelId, ChannelId)>, //Log Channel, Notifier Channel
    context: Option<serenity::prelude::Context>,
    receiver: Receiver<DBRequest>,
    sender: Sender<DBRequest>,
    active_temps: BTreeMap<i64, (GuildId, Temporary, JoinHandle<()>)>, //UserID, (GuildID, Temporary)
}

impl DBHandler {
    pub fn new(receiver: Receiver<DBRequest>, sender: Sender<DBRequest>) -> Self {
        DBHandler {
            database: BTreeMap::new(),
            threadlog: BTreeMap::new(),
            context: None,
            receiver,
            sender,
            active_temps: BTreeMap::new(),
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
                    if let Some((guild, logs)) = request.threadlog {
                        self.threadlog.insert(guild, logs);

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
                                                            &userprofile.punishments,
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
                                let end = Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + length.unwrap_or(-Timestamp::now().unix_timestamp()));
                                let idkey = target.0.id.get() as i64;
                                match end {
                                    Ok(end) => {
                                        if let Some(punishment) = self.process_punishment( idkey,  
                                            &invoker,
                                            &target,
                                            ptype.clone(),
                                            reason,
                                            (Timestamp::now(), end),
                                            &targetguild,
                                            &ctx).await {
                                                 apply_punishment(&ctx, targetguild, &punishment, &target.0)
                                                    .await
                                                    .expect("Failed to apply punishment");

                                                command
                                                    .create_response(
                                                        &ctx.http,
                                                        CreateInteractionResponse::Message(
                                                            CreateInteractionResponseMessage::new()
                                                                .content(format!("Added {:?} punishment to <@{}>.", ptype, idkey))
                                                                .ephemeral(true),
                                                        ),
                                                    )
                                                    .await
                                                    .expect("Failed to send response");

                                                if length.is_some() {
                                                    self.add_temporary(command, idkey, target, targetguild, invoker, Temporary {
                                                        user_id: idkey,
                                                        punishment,
                                                        negdur: !Timestamp::now().unix_timestamp(),
                                                    }).await;
                                                }                       
                                            }
                                            println!("Added punishment {:?} to user {}.", ptype, idkey);    
                                    },
                                    Err(e) => {
                                        command
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(format!("Invalid timestamp conversion: {}", e))
                                                        .ephemeral(true),
                                                ),
                                            )
                                            .await
                                            .expect("Failed to send response");
                                        continue;
                                    }
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
                                let idkey = target.0.id.get() as i64;
                                if let Some(mut userprofile) = self
                                    .get_profile(idkey, &targetguild)
                                    .await
                                {
                                    self.update_profile(&userprofile, &targetguild, &target, &invoker, &ctx).await;

                                    command
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(format!("Edited punishment for <@{}>.", idkey))
                                                        .ephemeral(true),
                                                ),
                                            )
                                            .await
                                            .expect("Failed to send response");

                                    if self.active_temps.contains_key(&idkey) && length.is_some() {
                                        if let Some((_,record,_)) = self.active_temps.get_mut(&idkey) {
                                            userprofile.edit_punishment(id, latest, length, reason, Some(record));
                                            let record_clone = record.clone();
                                            self.remove_temporary(idkey, &targetguild).await;
                                            self.add_temporary(command,idkey, target, targetguild, invoker, record_clone, ).await;
                                        }
                                    } else {
                                        userprofile.edit_punishment(id, latest, length, reason, None);
                                    }
                                    
                                } else {
                                        command
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(format!("<@{}> lacks any punishment history.", idkey))
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
                                                    latest,
                                                    silent,  .. } => {  
                                let idkey = target.0.id.get() as i64;
                                if let Some(mut userprofile) = self
                                    .get_profile(idkey, &targetguild)
                                    .await
                                {
                                    if self.active_temps.contains_key(&idkey) {
                                        self.remove_temporary(idkey, &targetguild).await;
                                    }

                                    if let Some(removed) = userprofile.remove_punishment(id, latest) {
                                        remove_punishment(&ctx, targetguild, &removed, &target.0)
                                            .await
                                            .expect("Failed to remove punishment");
                                    }

                                    self.update_profile(&userprofile, &targetguild, &target, &invoker, &ctx).await;
                                    if !silent {
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
                                } else {
                                        command
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(format!("<@{}> lacks any punishment history.", target.0.id))
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
                   return None;
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

    async fn process_punishment(&self, userid: i64, invoker: &User, target: &(User, Option<PartialMember>), ptype: PunishmentType, reason: Option<String>, punishtime: (Timestamp, Timestamp), guildid: &GuildId, ctx: &Context) -> Option<PunishmentRecord> {
        if let Some(guilddb) = self.database.get(guildid) {
            match guilddb.profilecol.find_one(doc! { "user_id": userid}) {
                Ok(Some(mut profile)) =>  {
                    let (profile, punishment) = profile.add_punishment(ptype,reason,punishtime, invoker.id.get() as i64);
                    self.update_profile(&profile, guildid, target, invoker, ctx).await;
                    return Some(punishment);
                },
                Ok(None) => {
                    if let Some((log,_)) = self.threadlog.get(guildid) {
                        let id = "1".to_string();
                        let punishment = PunishmentRecord {
                            id: id.clone(),
                            punishment: ptype,
                            reason,
                            punished_for: punishtime,
                            moderator: invoker.id.get() as i64,
                        };

                        let mut newpunishment = BTreeMap::new();
                        newpunishment.insert(id, punishment.clone());
                        let embed = profembed(invoker, target, &newpunishment).await;
                        let userthread = match create_user_profile(log, ctx, embed, userid).await {
                            Ok(channelid) => channelid,
                            Err(e) => {
                                eprintln!("Error creating user profile thread in Profile Query: {}", e);
                                return None;
                            }
                        };
                        if let Err(e) = guilddb.profilecol.insert_one(Profile::new(userid, userthread, newpunishment)) {
                            eprintln!("Error creating new profile in Database: {}", e);
                            return None;
                        }
                        return Some(punishment);
                    } else {
                        return None;
                    }
                }
                Err(e) => {
                    eprintln!("Error retrieving profile in Profile Query: {}", e);
                    return None;
                }
            };
        } else {
            eprintln!("No database found for queried guild in Profile Query");
            return None;
        }

    }
    
    async fn update_profile(&self, profile: &Profile, guildid: &GuildId, target: &(User, Option<PartialMember>), invoker: &User, ctx: &Context) {
        update_thread_post(ctx, 
            &profile.user_thread,
                profembed(invoker, target, &profile.punishments).await
            )
        .await
        .expect("Failed to update profile thread embed");
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

    async fn add_temporary(&mut self, command: CommandInteraction, userid: i64, target: (User, Option<PartialMember>), targetguild: GuildId, invoker: User, record: Temporary) {
        //Temporary task thread generation
        let handle_sender = self.sender.clone();
        let handle_ctx = self.context.clone();
        let handle_command = Command::PunishRemove { 
            command,
            targetguild,
            target,
            invoker,
            latest: None,
            id: Some(record.punishment.id.clone()),
            silent: true,
        };

        let handle = tokio::spawn(async move {
            let sleeptime = Duration::from_secs((record.punishment.punished_for.1.unix_timestamp() - record.punishment.punished_for.0.unix_timestamp()) as u64);             
            sleep(sleeptime).await;
            
            if let Err(e) = handle_sender.send(DBRequest {
                request_type: DBRequestType::Punishment,
                command: Some(handle_command),
                context: handle_ctx,
                threadlog: None,
            }).await {
                eprintln!("Failed to send TemporaryComplete request: {}", e);
            }
            println!("Temporary punishment for user {} has completed.", userid);
        });

        self.active_temps.insert(userid, (targetguild, record.clone(), handle));

        if let Some(guilddb) = self.database.get(&targetguild) {
            if let Err(e) = guilddb.tempcol.insert_one(record) {
                eprintln!("Error creating new temporary in Temporary Add: {}", e);
            }
        } else {
            eprintln!("No database found for queried guild in Profile Update");
        }
    }

    async fn remove_temporary(&mut self, userid: i64, guildid: &GuildId) -> Option<String> {
        if let Some((_, temp, handle)) = self.active_temps.remove(&userid) {
            handle.abort();
            if let Some(guilddb) = self.database.get(guildid) {
                if let Err(e) = guilddb.tempcol.delete_one(doc! { "user_id": userid }) {
                    eprintln!("Error removing temporary in Temporary Remove: {}", e);
                }
            } else {
                eprintln!("No database found for queried guild in Profile Update");
            }
            Some(temp.punishment.id)
        } else {
            None
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
    pub threadlog: Option<(GuildId, (ChannelId, ChannelId))>,
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
        silent: bool,
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
    },
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
    user_thread: ChannelId,
    pub punishments: BTreeMap<String, PunishmentRecord>, //id, Record
    negdur: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PunishmentRecord {
    pub id: String,
    pub punishment: PunishmentType,
    pub reason: Option<String>,
    pub punished_for: (Timestamp, Timestamp), //Start, End
    pub moderator: i64,
}

impl Profile {
    pub fn new(user_id: i64, user_thread: ChannelId, punishments: BTreeMap<String, PunishmentRecord>) -> Self {
        Profile {
            user_id,
            user_thread,
            punishments,
            negdur: !Timestamp::now().unix_timestamp(),
        }
    }

    pub fn add_punishment(&mut self, punishment: PunishmentType, reason: Option<String>, punished_for:(Timestamp, Timestamp), moderator: i64) -> (&mut Profile, PunishmentRecord) {
        self.negdur =!Timestamp::now().unix_timestamp();
        let id = match self.punishments.keys().last() {
            Some(last_id) => last_id.parse::<u16>().unwrap_or(0) + 1,
            None => 1,
        }.to_string();
        let record = PunishmentRecord {
            id: id.clone(),
            punishment,
            reason,
            punished_for,
            moderator,
        };
        self.punishments.insert(id, record.clone());
        (self, record)
    }

    pub fn remove_punishment(&mut self, id: Option<String>, latest: Option<bool>) -> Option<PunishmentRecord> {
        self.negdur =!Timestamp::now().unix_timestamp();
        match (id, latest) {
            (Some(pid), _) => {
                if let Some(record) = self.punishments.remove(&pid) {
                    return Some(record);
                } else {
                    return None;
                }
            }
            (None, Some(true)) => {
                if let Some(last_id) = self.punishments.keys().last().cloned() {
                    if let Some(record) = self.punishments.remove(&last_id) {
                        return Some(record);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }    
            }
            _ => { return None;}
        }
    }

    pub fn edit_punishment(&mut self, id: Option<String>, latest: Option<bool>, length: Option<i64>, reason: Option<String>, temp_record: Option<&mut Temporary >) {
         self.negdur =!Timestamp::now().unix_timestamp();
         match (id, latest) {
            (Some(pid), _) => {
                if let Some(record) = self.punishments.get_mut(&pid) {
                    if let Some(reason) = reason {
                        record.reason = Some(reason);
                    }
                    if let Some(length) = length {
                        let start = record.punished_for.0.unix_timestamp();
                        if let Ok(end) = Timestamp::from_unix_timestamp(start + length) {
                            record.punished_for.1 = end;
                        } else {
                            eprintln!("Error converting timestamp in Edit Punishment");
                        }
                    }
                }
            }
            (None, Some(true)) => {
                if let Some((_, record)) = self.punishments.iter_mut().next_back() {
                    if let Some(reason) = reason {
                        record.reason = Some(reason);
                    }
                    if let Some(length) = length {
                        let start = record.punished_for.0.unix_timestamp();
                        if let Ok(end) = Timestamp::from_unix_timestamp(start + length) {
                            record.punished_for.1 = end;
                            if let Some(temp_record) = temp_record {
                                temp_record.punishment = record.clone();
                                temp_record.negdur = !end.unix_timestamp();
                            }
                        } else {
                            eprintln!("Error converting timestamp in Edit Punishment");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Temporary {
    user_id: i64,
    punishment: PunishmentRecord,
    negdur: i64,
}