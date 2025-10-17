use serenity::{
    all::{Integration, PartialMember, Role, User}, 
    builder::{CreateCommand, CreateCommandOption}, 
    model::{application::{CommandOptionType, InstallationContext, InteractionContext},Permissions}
};
use serde::{Serialize, Deserialize};

pub enum ModbotCmd {
    FetchProfile,
    Punishment,
    RoleSet,
}

//Reference of all values known in commands
#[derive(Default, Debug)]
pub struct CommandOptions {
    pub user: Option<User>,
    pub member: Option<PartialMember>,
    pub role: Option<Role>,
    pub reason: Option<String>,
    pub allow: Option<bool>,
    pub duration: Option<i64>,
    pub units: Option<String>,
    pub id: Option<i64>,
    pub latest: Option<bool>,
    pub punishment: Option<PunishmentType>,
    pub action: Option<PunishmentAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PunishmentType {
    Warn,
    Mute,
    Ban,
    Timeout,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PunishmentAction {
    Add,
    Remove,
    Edit,
}

impl ModbotCmd {
    pub fn build(&self) -> CreateCommand {
        match self {
            ModbotCmd::FetchProfile => 
                CreateCommand::new("fetchprofile")
                    .default_member_permissions(Permissions::ADMINISTRATOR)
                    .add_context(InteractionContext::Guild)
                    .add_integration_type(InstallationContext::Guild)
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
                    .default_member_permissions(Permissions::ADMINISTRATOR)
                    .add_context(InteractionContext::Guild)
                    .add_integration_type(InstallationContext::Guild)
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
                    ),
            ModbotCmd::RoleSet => 
                CreateCommand::new("roleset")
                    .default_member_permissions(Permissions::ADMINISTRATOR)
                    .add_context(InteractionContext::Guild)
                    .add_integration_type(InstallationContext::Guild)
                    .description("Set role permission for commands")
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::Role,
                        "role",
                        "The role to set permissions for") 
                        .required(true)
                        .set_autocomplete(true))
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::Boolean,
                        "allow",
                        "Allow or disallow the command for this role") 
                        .required(true)
                )
                    //Complete will be handled in run_command
        }
    }
}