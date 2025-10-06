mod discord;
use discord::client::ClientHandler;
mod db;
use serenity::prelude::*;

#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("token").unwrap_or_else(|e| {
        panic!("Unable to read token: {}", e)
    });
    //Note will limit these once I got an idea what intents I need.
    let intents = GatewayIntents::GUILD_MESSAGES 
    | GatewayIntents::GUILD_MEMBERS 
    | GatewayIntents::GUILD_MODERATION
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILDS   
    | GatewayIntents::DIRECT_MESSAGES 
    | GatewayIntents::AUTO_MODERATION_EXECUTION;

    let dbconnection = db::DB_Handler;
    let chandle = ClientHandler::new(dbconnection);

    let mut client = Client::builder(&token, intents)
        .event_handler(chandle)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
