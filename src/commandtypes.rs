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
                    .add_option(CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "The user to fetch the profile for") 
                            .required(true)
                            .set_autocomplete(true)
                        ),
            ModbotCmd::Punishment =>
                CreateCommand::new("punish")
                    .description("Add/remove/edit punishment to a user")
                    .add_option(CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "Username or ID") 
                            .set_autocomplete(true)
                            .required(true))
                    .add_option(CreateCommandOption::new(
                            CommandOptionType::SubCommandGroup,
                            "add",
                            "Add a punishment to a user") 
                        .required(false)
                        .add_sub_option(
                            CreateCommandOption::new(
                            CommandOptionType::String,
                            "type",
                            "Type of punishment"))   
                            .add_string_choice("timeout", "T")
                            .add_string_choice("warn", "W")
                            .add_string_choice("mute", "M")
                            .add_string_choice("ban", "B")
                            .required(true)
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "time",
                            "Type of punishment"))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "duration",
                                "Duration of punishment")
                                .min_int_value(1)
                                .max_int_value(999)
                                .required(true))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "units",
                                "Units for duration") 
                                .required(true)
                                .add_string_choice("minute(s)", "M")
                                .add_string_choice("hour(s)", "H")
                                .add_string_choice("day(s)", "D")
                            .required(true))
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::String,
                            "reason",
                            "Reason for punishment"
                            ) 
                            .max_length(1000)
                            .required(true))
                    )
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommandGroup,
                        "remove",
                        "Remove a specified punishment"
                        ) 
                        .required(false)
                        .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "id",
                                "Remove punishment by ID"
                            )
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Boolean,
                                "latest",
                                "Remove latest punishment for this user")
                            )
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "number",
                                "The ID of the punishment to remove"
                                )   
                                .min_int_value(1))
                            )
                            .required(true)
                    )
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommandGroup,
                        "edit",
                        "Adjust a specified punishment"
                        ) 
                        .required(false)
                        .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::SubCommand,
                                "id",
                                "Remove punishment by ID"
                            )
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Boolean,
                                "latest",
                                "Remove latest punishment for this user")
                            )
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "number",
                                "The ID of the punishment to remove"
                                )   
                                .min_int_value(1))
                            )
                            .required(true)
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::String,
                            "reason",
                            "Update the reason for a punishment")
                            .max_length(1000)
                            .required(false)
                        )
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "time",
                            "Update the length of punishment"))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "duration",
                                "Duration of punishment")
                                .min_int_value(1)
                                .max_int_value(999)
                                .required(true))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "units",
                                "Units for duration") 
                                .required(true)
                                .add_string_choice("minutes", "M")
                                .add_string_choice("hours", "H")
                                .add_string_choice("days", "D")
                            .required(false))
                    )
                    //Complete will be handled in run_command
        }
    }
}
