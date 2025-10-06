# Modbot

The main point of the modbot system is to moderate users, but it also has the necessary capacity of logging information of any kind regarding the user.

### Structure:

The main component (main.rs in the Server folder) is intialization. Serenity utilizes a client system which builds with a EventHandler struct, the ClientHandler in modbot implements it and recieves events from Discord via this way.

### ClientHandler

 On cache ready state, the client handler will search all guilds for Adminstrator priveleges, then in guilds it holds these priveleges it will either generate or grab the current #modbot-logs channel for profile maintaince and creation. It should be a task on generation to update these or make a regular (maybe paced) pattern to update these profiles as long as Flags for not being in the server or banned are not set. 
 
 The client handler will have a intialized DB_Handler, which will be described later but it mainly it maintains the Connecton to the the postgres database, sending and recieving data. The handler should also generate a log channel if there isn't one and synchronize profiles in that log channel. It'll be important in maintaining temporary punishments, and if disconnections occur it should be dropped and immediately re-generated on cache_ready.

### Commands
<pre>
├── /fetchprofile
    ├── user (User | REQUIRED)
├── /punish
    ├── add (SubCommandGroup)
        ├── timeout (SubCommand)
            ├── user (User | REQUIRED)
            ├── duration (Integer | REQUIRED)
            ├── units (String Choice | REQUIRED)
                ├── "minute(s)"
                ├── "hour(s)"
                ├── "day(s)"
            ├── reason (String)
        ├── warn (SubCommand)
            ├── user (User | REQUIRED)
            ├── reason (String)
        ├── mute (SubCommand)
            ├── user (User | REQUIRED)
            ├── duration (Integer)
            ├── units (String Choice)
                ├── "minute(s)"
                ├── "hour(s)"
                ├── "day(s)"
            ├── reason (String)
        ├── ban (SubCommand)
            ├── user (User | REQUIRED)
            ├── duration (Integer)
            ├── units (String Choice)
                ├── "minute(s)"
                ├── "hour(s)"
                ├── "day(s)"
            ├── reason (String)
    ├── remove (SubCommand)
        ├── user (User | REQUIRED)
        ├── id (Integer)
        ├── latest (Boolean)
    ├── edit (SubCommand)
        ├── user (User | REQUIRED)
        ├── id (Integer)
        ├── latest (Boolean)
        ├── reason (String)
        ├── duration (Integer)
        ├── units (String Choice)
            ├── "minutes"
            ├── "hours"
            ├── "days"
</pre>

Profiles
### Profile 
Profiles are embed messages with details about the user. The /fetchprofile command will foward the most recent profile from the #modbot-log channel, which will hold all profiles on punished users. 

Flags: Quick information to check against for active punishment should be added. This includes ban, mute, timeout, in server, or quarantine. (more to be added).

### Punishment
The /punish command will be utilized to add, remove or edit a punishment on a user.

How punishments will be handled will depend on whether they're given a time and duration.
For temporary punishments, DB_Handler workers are to be generated and given a clone of mpsc sender to communicate when it's finished based on Unicode time of completion in the database. Permanent punishments can be achieved by omitting a duration.

Note: Flags should be triggered indicating an active punishment or removed afterwards, to be displayed on the moderation profile of the user.


**Note**: Ideas for the DB structure are not yet completely figured out, personally I'm not very apt with SQL or Postgres so this will be under major remake once the rest of the bot is created.

### SQL DB Structure (Table Profile)

* UID
> type: BIGINT (PRIMARY KEY)
* Info
> type: JSONB

An index should be made regarding all users that are present in the server (i.e. not banned for quicker parsing in the database)

### SQL DB Structure (Table Temporary)

* UID
> type: BIGINT (PRIMARY KEY)
* Type
> type: VARCHAR(10)
* Duration (Unixtime Offset)
> type: BIGINT

### Depedencies

Discord API Handler: serenity
> Docs: https://docs.rs/serenity/latest/serenity/
> A very useful wiki: https://deepwiki.com/serenity-rs/serenity/1-overview