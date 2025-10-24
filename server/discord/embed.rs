use serenity::{
    builder::{CreateEmbed, CreateEmbedFooter}, 
    model::{ Timestamp, guild::PartialMember, user::User}, 
    utils::{FormattedTimestamp, FormattedTimestampStyle}
};
use crate::{db::Profile, discord::commands::PunishmentType};
//Add a active flag to Profile to allow for fetches to go for the last punishment and set active punishment. Use temporary events to disable this flag if timed.
pub async fn profembed(invodata: &User, data: &(User, Option<PartialMember>), profile: Profile) -> CreateEmbed {
    let mut footstring = format!("Moderator: {}", invodata.name);   
    let mut embed = CreateEmbed::default()
        .title(format!("User Profile"))
        .description(format!("<@{}>",data.0.id))
        //Color will never be there because it needs a HTTP RestAPI request
        .field( "Creation Date", FormattedTimestamp::new(data.0.created_at(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true)
        .thumbnail(data.0.avatar_url().unwrap_or_default())            
        .timestamp(Timestamp::now());

    embed = match &data.1 {
        Some(member) => {
            footstring.push_str("\nMember: ✅");     
            let embed = embed.fields(vec![
                ("Join Date", FormattedTimestamp::new(member.joined_at.unwrap_or_default(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true),
                ("Roles", if member.roles.len() > 0 {
                    member.roles
                        .iter()
                        .map(|r| format!("<@&{}>", r))
                        .collect::<Vec<String>>()
                        .join(", ")
                } else {
                    "No Roles".to_string()
                }, false),
            ]);
            embed
        },
        None => {
            footstring.push_str("\nMember: ❎");                   
            embed
        },
    };

    
    embed = if profile.punishments.len() > 0 {
        let mut detailnames = vec![]; 
        let mut punishdetails = vec![];
        for (pid, record) in profile.punishments.iter() {
            if let (Ok(start), Ok(end)) = (Timestamp::from_unix_timestamp(record.punished_for.0),Timestamp::from_unix_timestamp(record.punished_for.1)) {
                
                if end == Timestamp::default() || end > Timestamp::now() {
                    embed = match record.punishment {
                        PunishmentType::Ban => {
                            footstring.push_str("  -  Banned: ✅");
                            embed.color(0xFF0000) //Red
                        }
                        PunishmentType::Mute => {
                            footstring.push_str("  -  Muted: ✅");
                            embed.color(0xFF9900) //Orange
                        }
                        PunishmentType::Timeout => {
                            footstring.push_str("  -  Timeout: ✅");
                            embed.color(0xFFE600) //Yellow
                        },
                        _ => {embed}
                    };
                detailnames.push(format!("{:?} (ID {})", record.punishment, pid));
                punishdetails.push(format!("\n **Reason:** {}\n**Period:** {} - {}\n**Moderator:** <@{}>\n\n",
                    record.reason.clone().unwrap_or("No reason provided.".to_string()),
                    FormattedTimestamp::new(start, Some(FormattedTimestampStyle::ShortDateTime)).to_string(),
                    if end == Timestamp::default() {
                        "Permanent".to_string()
                    } else {
                        FormattedTimestamp::new(end, Some(FormattedTimestampStyle::ShortDateTime)).to_string()
                    },
                        record.moderator,
                ));
            }
            } else {
                eprintln!("Failed to parse punishment timestamps");
            }
        }
        for (name, detail) in detailnames.iter().rev().zip(punishdetails.iter().rev()) {
            embed = embed.field(name, detail, false);
        }
        embed
    } else {
        embed
    };

    embed.footer(CreateEmbedFooter::new(footstring)
        .icon_url(invodata.avatar_url().unwrap_or_default()))
}