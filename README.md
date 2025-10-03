# Modbot

The main point of the modbot system is to moderate users, but it also has the necessary capacity of logging information of any kind regarding the user. It mainly has the capacity of universal logging tool, with mod tools attached.

### Structure:

The main component (main.rs in the Server folder) is the interface for commands. Serenity utilizes a client system which builds with a Handler struct that is suppose to implement their EventHandler. In this design, we can build upon the Handler to have certain information to facilitate communication with the DB Handler i.e. mpsc senders.

Immediately replies should be given to all command entry events. Most events will be sent to the CommandHandler.

### Command Handler

The command handler is a structure that is generated new with the receiver address from the mspc and the GuildID. On generation it should establish and store a Connection to the the postgres database. The handler should also generate a log channel if there isn't one and synchronize profiles in that log channel. In case of failures a recheck should be done on new for temporary punishments.

The slash commands for punishments will follow a form that allows for near single command control of the bot. The form of the command is as follows:

<pre>
├──  /punish
	├── user (String)
	├── add (SubCommandGroup)
		├── type (String Choice)
			├──"warn"
			├── "timeout"
			├── "mute"
			├── "ban"
		├── time (Subcommand)
			├── duration (Integer)
			├── unit (String Choice)
				├── "minute(s)"
				├── "hour(s)"
				├── "day(s)"
		├── reason (String)
	├── remove (SubCommandGroup)
		├── id (Subcommand)
			├── latest (Boolean)
			├── number (Integer)
	├── edit (SubCommandGroup)
		├── id (Subcommand)
			├── latest (Boolean)
			├── number (Integer)
		├── time (Subcommand)
			├── duration (Integer)
			├── unit (String Choice)
				├── "minute(s)"
				├── "hour(s)"
				├── "day(s)"
		├── reason (String)
</pre>

### Profile 
Profiles are embed messages with details about the user:

Flags: Quick information to check against for active punishment. This includes ban, mute, timeout, in server, or quarantine. (more to be added)

### Punishment

For temporary punishments, workers are to be generated and given a clone of mpsc sender to communicate when it's finished based on Unicode time of completion in the database. Permanent punishments can be achieved by omitting a duration.

Note: Flags should be triggered indicating an active punishment or removed afterwards.

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
> type: TIMESTAMP

### Depedencies

Discord API Handler: serenity
> [Docs | https://docs.rs/serenity/latest/serenity/]