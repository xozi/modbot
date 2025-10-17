use std::sync::Arc;
use serenity::{
    all::{content_safe,ContentSafeOptions}, builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter}, cache::Cache, model::{ Timestamp, channel::{Embed, GuildChannel}, guild::{self, PartialMember}, id::{GuildId, RoleId, UserId}, user::User}, utils::{FormattedTimestamp, FormattedTimestampStyle}
};

pub async fn profembed(invodata: &User, data: &(User, Option<PartialMember>), cache: &Arc<Cache>) -> CreateEmbed {
    let basicembed = CreateEmbed::default()
        .title(format!("User Profile"))
        .description(format!("<@{}>",data.0.id))
        //Color will never be there because it needs a HTTP RestAPI request
        .field( "Creation Date", FormattedTimestamp::new(data.0.created_at(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true)
        .footer(CreateEmbedFooter::new(format!("Requestor: {}", invodata.name))
            .icon_url(invodata.avatar_url().unwrap_or_default()))
        .thumbnail(data.0.avatar_url().unwrap_or_default())            
        .timestamp(Timestamp::now());
    if let Some(member) = &data.1 {
        let embed = basicembed
            .fields(vec![
            ("Join Date", FormattedTimestamp::new(member.joined_at.unwrap_or_default(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true),
            ("Roles", member.roles
                        .iter()
                        .map(|role_id| format!("<@&{}>", role_id))
                        .collect::<Vec<_>>()
                        .join(", "), false),
        ]);
        embed
    } else {
        basicembed
    }

}