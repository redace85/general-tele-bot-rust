use teloxide::{dispatching::UpdateHandler, prelude::*, RequestError};
use tokio::sync::broadcast;
use signal_hook::{
    consts::signal::{SIGINT, SIGTERM},
    iterator::Signals,
};

use std::{sync::{Arc, atomic::AtomicBool}, time::Duration};

mod cmd_handles;
mod msg_handles;
mod schedule_task;
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
    let timeout = std::env::var("TIMEOUT").expect("timeout not set").parse::<u64>().expect("convert timeout failed");
    log::info!("timeout: {timeout}");
    let schedule_chat_id = std::env::var("SCHEDULE_CHAT_ID")
    .expect("schedule chat id not set").parse::<i64>().expect("convert schdule chat id failed");
    log::info!("schedule: {schedule_chat_id}");

    let bot = Bot::from_env();
    let states = Arc::new(states::SqliteState::new(db_path.into(), auth_token, timeout).unwrap());

    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    let shutdown = Arc::new(AtomicBool::new(false));
    
    let schedule_loop_handle = {
        let shutdown = shutdown.clone();
        let schedule_bot = bot.clone();
        tokio::task::spawn(async move {
            while !shutdown.load(std::sync::atomic::Ordering::Acquire) {
                schedule_task::entry(&schedule_bot, schedule_chat_id).await.unwrap_or_else(|err| {
                    panic!("failed to check for new posts: {err}");
                });

                tokio::select! {
                   _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                   _ = shutdown_rx.recv() => {
                       break
                   }
                }
            }
        })
    };

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![states])
        // .enable_ctrlc_handler()
        .build();

    {
        let shutdown = shutdown.clone();
        let bot_shutdown_token = dispatcher.shutdown_token();

        std::thread::spawn(move || {
            let mut forward_signals =
                Signals::new([SIGINT, SIGTERM]).expect("unable to watch for signals");

            for signal in forward_signals.forever() {
                log::info!("got signal {signal}, shutting down...");
                shutdown.swap(true, std::sync::atomic::Ordering::Relaxed);
                let _res = bot_shutdown_token.shutdown();
                let _res = shutdown_tx.send(()).unwrap_or_else(|_| {
                    // Makes the second Ctrl-C exit instantly
                    std::process::exit(0);
                });
            }
        });
    }

    if let Err(err) = tokio::try_join!(tokio::spawn(async move { dispatcher.dispatch().await }),
     schedule_loop_handle) {
        panic!("{err}")
    }

}

fn schema() -> UpdateHandler<RequestError> {
    let command_handler = teloxide::filter_command::<cmd_handles::Command, _>()
        .branch(dptree::endpoint(cmd_handles::entry));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(msg_handles::entry));

    message_handler
}
