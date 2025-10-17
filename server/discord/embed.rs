use serenity::{
    builder::{CreateEmbed, CreateEmbedFooter}, 
    model::{ Timestamp, guild::PartialMember, user::User}, 
    utils::{FormattedTimestamp, FormattedTimestampStyle}
};
use crate::db::Profile;

pub async fn profembed(invodata: &User, data: &(User, Option<PartialMember>), profile: Profile) -> CreateEmbed {
    let basicembed = CreateEmbed::default()
        .title(format!("User Profile"))
        .description(format!("<@{}>",data.0.id))
        //Color will never be there because it needs a HTTP RestAPI request
        .field( "Creation Date", FormattedTimestamp::new(data.0.created_at(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true)
        .footer(CreateEmbedFooter::new(format!("Requestor: {}", invodata.name))
            .icon_url(invodata.avatar_url().unwrap_or_default()))
        .thumbnail(data.0.avatar_url().unwrap_or_default())            
        .timestamp(Timestamp::now());

    let memberembed = match &data.1 {
        Some(member) => {
            let embed = basicembed.fields(vec![
                ("Join Date", FormattedTimestamp::new(member.joined_at.unwrap_or_default(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true),
                ("Roles", member.roles
                            .iter()
                            .map(|role_id| format!("<@&{}>", role_id))
                            .collect::<Vec<_>>()
                            .join(", "), false),
            ]);
            embed
        },
        None => basicembed,
    };

    let punishmentembed = if profile.punishments.len() > 0 {
        let mut punishments = String::new();
        for (pid, record) in profile.punishments.iter() {
            punishments.push_str(&format!("{:?} (ID #{}): \nReason: {}\nPeriod: {} - {}\nModerator: <@{}>\n\n",
                record.punishment,
                pid,
                record.reason.clone().unwrap_or("No reason provided".to_string()),
                FormattedTimestamp::new(record.punished_for.0, Some(FormattedTimestampStyle::ShortDateTime)).to_string(),
                if record.punished_for.1 == Timestamp::default() {
                    "Permanent".to_string()
                } else {
                    FormattedTimestamp::new(record.punished_for.1, Some(FormattedTimestampStyle::ShortDateTime)).to_string()
                },
                record.moderator,
            ));
        }
        memberembed.field("History", punishments, false)
    } else {
        memberembed
    };
    punishmentembed
}