use serenity::{
    builder::{CreateEmbed, CreateEmbedFooter}, 
    model::{ Timestamp, guild::PartialMember, user::User}, 
    utils::{FormattedTimestamp, FormattedTimestampStyle}
};
use crate::{db::PunishmentRecord, discord::commands::PunishmentType};
use std::collections::BTreeMap;

//Add a active flag to Profile to allow for fetches to go for the last punishment and set active punishment. Use temporary events to disable this flag if timed.
pub async fn profembed(invodata: &User, data: &(User, Option<PartialMember>), punishments: &BTreeMap<String,PunishmentRecord>) -> CreateEmbed {
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
            embed = embed.field("Join Date", FormattedTimestamp::new(member.joined_at.unwrap_or_default(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true);
            if member.roles.len() > 0 {
                embed = embed.field("Roles", member.roles
                        .iter()
                        .map(|r| format!("<@&{}>", r))
                        .collect::<Vec<String>>()
                        .join(", "), 
                        false);
            } 
            embed
        },
        None => {
            footstring.push_str("\nMember: ❎");                   
            embed
        },
    };

    
    embed = if punishments.len() > 0 {
        let mut detailnames = vec![]; 
        let mut punishdetails = vec![];
        for (pid, record) in punishments.iter() {
            detailnames.push(format!("{:?} (ID {})", record.punishment, pid));

            if let Some(reason) = &record.reason {
                punishdetails.push(format!("\n**Reason:** {}", reason));
            }

            punishdetails.push(format!("\n**Period:** {} - {}\n**Moderator:** <@{}>\n\n",
                FormattedTimestamp::new(record.punished_for.0, Some(FormattedTimestampStyle::ShortDateTime)).to_string(),
                if record.punished_for.1 == Timestamp::default() {
                    "Permanent".to_string()
                } else {
                    FormattedTimestamp::new(record.punished_for.1, Some(FormattedTimestampStyle::ShortDateTime)).to_string()
                },
                record.moderator,
            ));

            if record.punished_for.1 == Timestamp::default() || record.punished_for.1 > Timestamp::now() {
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