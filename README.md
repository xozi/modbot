# Modbot

The main point of the modbot system is to moderate users, but it also has the necessary capacity of logging information of any kind regarding the user.

### Structure

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
├── /setpermission
    ├── role (Role | REQUIRED)
    ├── allow (Boolean | REQUIRED)
</pre>

### Profile 
Profiles are embed messages with details about the user. The /fetchprofile command will generate a profile with records over the user and is dynamically updated. The profile present in #modbot-log will be more static in nature and only will be updated when a punishment is commited. The log exists for adminstrators to monitor recent punishments and keep track of moderation actions.

### Punishment
The /punish command will be utilized to add, remove or edit a punishment on a user.

How punishments will be handled will depend on whether they're given a time and duration.
For temporary punishments, DB_Handler workers are to be generated and given a clone of mpsc sender to communicate when it's finished based on Unicode time of completion in the database. Permanent punishments can be achieved by omitting a duration.

Note that edit should be used to commute a sentence, while remove should be use to entirely remove it from record.

Commands will be role limited, necessary documents for setting role system including a database to store these roles should be established:
https://docs.rs/serenity/latest/serenity/builder/struct.CreateCommandPermission.html

It can be proactively updated, avoiding the need of restart:
https://docs.rs/serenity/latest/serenity/model/id/struct.GuildId.html#method.edit_command_permissions

Be aware that when the command is first established all Adminstrator users will have access to set commands via default permissions. Once a role is given a permission, there is a override event. Ensure that you give permission to an adminstrative role first as I'm not sure if adminstrators will have access.


**Note**: Ideas for the DB structure are not yet completely figured out, personally I'm not very apt with SQL or Postgres so this will be under major remake once the rest of the bot is created.

### Database Structure

Embedded databases are generated per guild, there should be 2 collections per database.

* "Temporary" Collection for all currently pending punishments.
* "Profile" Collection for all profiles of punished users.
* "RolePermission" Collection for roles that have permission controls for the commands. By default empty, will verify sender of command.

Documents are BSON.

### Update Ideas

* When a /punish command is sent a ghost ping is sent to the exact log page that is updated for the moderator.
* Admin only /reverse command that helps reverse past punishments for trial mods (security), also limit ban outside their range. Integrated rate limit for trial mods. (Only consider this if necessary)
* Optimized checks for roles to avoid unecessary API pings (hopefully the cache does this)
* Figure out efficient means of adding image evidence to profile punishments without storing the data if possible.
* Make a case for quarantined status flag (need to use events for that)
* Add an info post on intialization of the log channel, that will be updated with time in the embed module if necessary.

### Depedencies

Discord API Handler: serenity
> Docs: https://docs.rs/serenity/latest/serenity/

> A very useful wiki: https://deepwiki.com/serenity-rs/serenity/1-overview

Embedded Database: PoloDB
> Github: https://github.com/PoloDB/PoloDB
