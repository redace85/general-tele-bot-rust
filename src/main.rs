use teloxide::{dispatching::UpdateHandler, prelude::*, RequestError};

use std::sync::Arc;

mod cmd_handles;
mod msg_handles;
mod ollama_ops;
mod states;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let auth_token = std::env::var("AUTH_TOKEN").expect("auth_token not set");
    log::info!("auth_token: {auth_token}");
    let db_path = std::env::var("DB_PATH").expect("db_path not set");
    log::info!("db_path: {db_path}");

    let bot = Bot::from_env();
    let states = Arc::new(states::SqliteState::new(db_path.into(), auth_token).unwrap());

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![states])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<RequestError> {
    let command_handler = teloxide::filter_command::<cmd_handles::Command, _>()
        .branch(dptree::endpoint(cmd_handles::entry));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(msg_handles::entry));

    message_handler
}
