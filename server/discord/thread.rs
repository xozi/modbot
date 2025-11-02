use serenity::{
    all::{CreateEmbed, CreateForumPost, CreateMessage, EditMessage},
    builder::GetMessages,
    model::id::ChannelId,
    prelude::*,
};

/*
use async_recursion::async_recursion;
//Rate limit the async recursion with a value.
#[async_recursion]
pub async fn recarch_thread_search<'a>(log_channel: &'a GuildChannel, ctx: &'a Context, threadname: String, mut timestamp: Option<u64>) -> Result<Option<GuildChannel>, SerenityError> {
    let thread_data = log_channel.id
                    .get_archived_public_threads(&ctx.http, timestamp, None)
                    .await?;
    if thread_data.has_more {
        timestamp = thread_data.threads.last()
            .and_then(|last_thread| {
                last_thread.thread_metadata.as_ref()
                    .and_then(|m| m.archive_timestamp)
                    .map(|ts| ts.unix_timestamp() as u64)
            });
        if timestamp.is_some() {
            return recarch_thread_search(log_channel, ctx, threadname, timestamp).await;
        }
    } else {
        return Ok(thread_data.threads
                        .iter()
                        .find(|t| t.name == threadname)
                        .cloned());
    } 
    Ok(None)
}

pub async fn active_thread_search(guild: &GuildId,ctx: &Context,threadname: String,) -> Result<Option<GuildChannel>, SerenityError> {
    Ok(guild
        .get_active_threads(&ctx.http)
        .await?.threads
        .into_iter()
        .find(|t| t.name == threadname))
}
*/

pub async fn update_thread_post(ctx: &Context,thread: &ChannelId,edit: CreateEmbed) -> Result<(), SerenityError> {
    //Finds the first message in the thread and edit it with new embed. Include additional adding if it exceeds 5 punishments.
    let messages = thread.messages(&ctx.http, GetMessages::new().limit(1)).await?;
    if let Some(mut message) = messages.into_iter().next() {
        message
            .edit(&ctx.http, EditMessage::new()
            .embed(edit)).await?;
    }
    Ok(())
}

pub async fn create_user_profile(log: &ChannelId,ctx: &Context, embed: CreateEmbed, userid: i64) -> Result<ChannelId, SerenityError> {
    Ok(log.create_forum_post(&ctx.http,CreateForumPost::new(userid.to_string(),
    CreateMessage::new()
                .embed(embed)
        )
    ).await?.id)
}