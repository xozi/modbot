use serenity::{
    model::{
        id::{UserId, ChannelId, CommandId},
        application::{CommandOptionType, ResolvedOption, ResolvedValue},
    },
    builder::{CreateCommand, CreateCommandOption},
};

/*
pub fn punishstr (s: &str) -> Option<PunishmentType> {
    match s {
        "ban" => Some(PunishmentType::Ban),
        "mute" => Some(PunishmentType::Mute),
        "warn" => Some(PunishmentType::Warn),
        "timeout" => Some(PunishmentType::Timeout),
        _ => None,
    }
}
*/


pub fn unixtime (unit: &str, period: u64) -> Option<u64> {
    let ct = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    match unit {
        "s" => Some(ct + period),
        "m" => Some(ct + period * 60),
        "h" => Some(ct + period * 3600),
        "d" => Some(ct + period * 86400),
        "w" => Some(ct + period * 604800),
        _ => None,
    }
}

pub enum ModbotCmd {
    FetchProfile,
    Punishment,
}

impl ModbotCmd {
    pub fn build(&self) -> CreateCommand {
        match self {
            ModbotCmd::FetchProfile => 
                CreateCommand::new("fetchprofile")
                    .description("Fetch a user's profile")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "The user to fetch the profile for") 
                            .required(true)
                            .set_autocomplete(true)
                        ),
            ModbotCmd::Punishment =>
                CreateCommand::new("punishment")
                    .description("Apply/remove punishment to user")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "Username or ID") 
                            .required(true)
                            .set_autocomplete(true)
                        )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommandGroup,
                            "punishment",
                            "Type of punishment"
                        ) 
                        .required(true)
                        .set_sub_options(
                            vec![CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "ban",
                                "Ban a user"),
                            CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "mute", 
                                "Mute a user"),
                            CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "warn", 
                                "Warn a user"),
                            CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "timeout", 
                                "Timeout a user")]
                        )
                    )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::Integer,
                            "duration",
                            "Duration of punishment in specified units"
                        )
                        .min_int_value(1)
                        .max_int_value(999)
                        .required(false) 
                    )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "units",
                            "Units for duration") 
                            .required(false)
                            .set_sub_options(
                                vec![CreateCommandOption::new(
                                    CommandOptionType::SubCommand,
                                    "minutes",
                                    "Minutes"),
                                CreateCommandOption::new(
                                    CommandOptionType::SubCommand,
                                    "hours", 
                                    "Hours"),
                                CreateCommandOption::new(
                                    CommandOptionType::SubCommand,
                                    "days", 
                                    "Days"),
                                ])
                        )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "reason",
                            "Reason for punishment"
                            ) 
                            .required(false)
                            .max_length(1000)
                        )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommandGroup,
                            "remove",
                            "Remove a specified the punishment"
                            ) 
                            .required(false)
                            .add_sub_option(
                                    CreateCommandOption::new(
                                        CommandOptionType::SubCommand,
                                        "id",
                                        "Remove punishment by ID"
                                    )
                                    .add_sub_option(
                                        CreateCommandOption::new(
                                            CommandOptionType::Integer,
                                            "punishment_id",
                                            "The ID of the punishment to remove"
                                        )
                                        .required(true)
                                        .min_int_value(1)
                                    )
                                )
                                .add_sub_option(
                                    CreateCommandOption::new(
                                        CommandOptionType::SubCommand,
                                        "latest",
                                        "Remove latest punishment for this user"
                                    )
                                )
                        )
                    //Complete will be handled in run_command
        }
    }
}
