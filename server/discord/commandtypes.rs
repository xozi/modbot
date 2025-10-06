use serenity::{
    model::application::CommandOptionType,
    builder::{CreateCommand, CreateCommandOption},
};

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
                    // Add
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommandGroup,
                        "add",
                        "Add a punishment to a user") 
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "timeout",
                            "Add a timeout to a user")
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to punish") 
                                .required(true)
                                .set_autocomplete(true))
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
                                .add_string_choice("minute(s)", "M")
                                .add_string_choice("hour(s)", "H")
                                .add_string_choice("day(s)", "D")
                                .required(true))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "reason",
                                "Reason for timeout") 
                                .max_length(1000))
                        )
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "warn",
                            "Add a warning to a user")
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to punish") 
                                .required(true)
                                .set_autocomplete(true)) 
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "reason",
                                "Reason for warn") 
                                .max_length(1000))
                        )
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "mute",
                            "Add a mute to a user")
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to punish") 
                                .required(true)
                                .set_autocomplete(true))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "duration",
                                "Duration of punishment")
                                .min_int_value(1)
                                .max_int_value(999))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "units",
                                "Units for duration") 
                                .add_string_choice("minute(s)", "M")
                                .add_string_choice("hour(s)", "H")
                                .add_string_choice("day(s)", "D"))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "reason",
                                "Reason for punishment") 
                                .max_length(1000))
                        )
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "ban",
                            "Add a ban to a user")
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to punish") 
                                .required(true)
                                .set_autocomplete(true)) 
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "duration",
                                "Duration of punishment")
                                .min_int_value(1)
                                .max_int_value(999))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "units",
                                "Units for duration") 
                                .add_string_choice("minute(s)", "M")
                                .add_string_choice("hour(s)", "H")
                                .add_string_choice("day(s)", "D"))
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::String,
                                "reason",
                                "Reason for punishment") 
                                .max_length(1000))
                            )
                        )
                    // Remove
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "remove",
                        "Remove a specified punishment")
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "The user to punish") 
                                .required(true)
                                .set_autocomplete(true)) 
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Integer,
                                "id",
                                "The ID of the punishment to remove")
                                .min_int_value(1)) 
                            .add_sub_option(CreateCommandOption::new(
                                CommandOptionType::Boolean,
                                "latest",
                                "Remove the latest punishment for this user"))
                    )
                    // Edit
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::SubCommand,
                        "edit",
                        "Adjust a specified punishment")
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::User,
                            "user",
                            "The user to punish") 
                            .required(true)
                            .set_autocomplete(true))   
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::Integer,
                            "id",
                            "The ID of the punishment to edit")
                            .min_int_value(1)) 
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::Boolean,
                            "latest",
                            "Edit the latest punishment for this user"))
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::String,
                            "reason",
                            "Update the reason for a punishment")
                            .max_length(1000))
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::Integer,
                            "duration",
                            "Update the duration of punishment")
                            .min_int_value(1)
                            .max_int_value(999))
                        .add_sub_option(CreateCommandOption::new(
                            CommandOptionType::String,
                            "units",
                            "units for duration") 
                            .add_string_choice("minutes", "M")
                            .add_string_choice("hours", "H")
                            .add_string_choice("days", "D"))
                    )
                    //Complete will be handled in run_command
        }
    }
}

