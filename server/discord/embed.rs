use std::sync::Arc;
use serenity::{
    all::{content_safe,ContentSafeOptions}, builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter}, cache::Cache, model::{ Timestamp, channel::{Embed, GuildChannel}, guild::{self, PartialMember}, id::{GuildId, RoleId, UserId}, user::User}, utils::{FormattedTimestamp, FormattedTimestampStyle}
};

pub async fn profembed(invodata: &User, udata: User, mdata: PartialMember, cache: &Arc<Cache>) -> CreateEmbed {
    return CreateEmbed::default()
        .title(format!("User Profile"))
        .description(format!("<@{}>",udata.id))
        //Color will never be there because it needs a HTTP RestAPI request
        .color(invodata.accent_colour.unwrap_or_default())
        .fields(vec![
            ("Join Date", FormattedTimestamp::new(mdata.joined_at.unwrap_or_default(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true),
            ("Creation Date", FormattedTimestamp::new(udata.created_at(), Some(FormattedTimestampStyle::ShortDateTime)).to_string(), true),
            ("Roles", mdata.roles
                        .iter()
                        .map(|role_id| format!("<@&{}>", role_id))
                        .collect::<Vec<_>>()
                        .join(", "), false),
        ])
        .footer(CreateEmbedFooter::new(format!("Requestor: {}", invodata.name))
            .icon_url(invodata.avatar_url().unwrap_or_default()))
        .thumbnail(udata.avatar_url().unwrap_or_default())            
        .timestamp(Timestamp::now());
}