use teloxide::payloads::SendMessage;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

use std::path::MAIN_SEPARATOR;
use std::{path::Path, sync::Arc};

use crate::states::SqliteState;

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Start cmd")]
    Start,
    #[command(description = "Auth with auth token")]
    Auth(String),
    #[command(description = "Download file")]
    Down(String),
}

pub async fn entry(
    bot: Bot,
    states: Arc<SqliteState>,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            if let Some(chat_id) = states.get_auth_chat_id() {
                // already auth
                if chat_id != msg.chat.id.0 {
                    let username = msg.from().unwrap().username.clone().unwrap();
                    let warning_msg = format!("{username} is starting the bot");

                    send_warning_notification(&bot, chat_id, warning_msg).await?;
                } else {
                    bot.send_message(msg.chat.id, "🚩 already auth").await?;
                }
            } else {
                // not auth yet please auth
                bot.send_message(msg.chat.id, " 🆓 not auth yet, be quick! auth now 🤞 ")
                    .await?;
            }
        }
        Command::Auth(auth_token) => {
            if let Some(chat_id) = states.get_auth_chat_id() {
                // already auth
                if chat_id != msg.chat.id.0 {
                    let username = msg.from().unwrap().username.clone().unwrap();
                    let warning_msg = format!("{username} is trying to auth");

                    send_warning_notification(&bot, chat_id, warning_msg).await?;
                } else {
                    bot.send_message(msg.chat.id, "🚩 already auth").await?;
                }
            } else {
                // auth
                if states.get_auth_token() == auth_token {
                    states.set_auth_chat_id(msg.chat.id.0);
                    bot.send_message(msg.chat.id, "✅ auth suceed").await?;
                }
            }
        }
        Command::Down(file_name) => {
            if let Some(chat_id) = states.get_auth_chat_id() {
                // already auth
                if chat_id != msg.chat.id.0 {
                    let username = msg.from().unwrap().username.clone().unwrap();
                    let warning_msg = format!("{username} is trying to download file");

                    send_warning_notification(&bot, chat_id, warning_msg).await?;
                } else {
                    let current_path = states
                        .query_current_path(msg.chat.id.0)
                        .unwrap_or("/".into());
                    let file = format!(
                        "{}{}{}",
                        current_path.to_str().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );

                    if !Path::new(&file).is_file() {
                        // file not exist
                        bot.send_message(msg.chat.id, "🚫file: {file} no exists")
                            .await?;
                    } else {
                        let inputfile = InputFile::file(file);
                        bot.send_document(msg.chat.id, inputfile).await?;
                    }
                }
            } else {
                // not auth yet
                bot.send_message(msg.chat.id, "❌ cmd not available before auth")
                    .await?;
            }
        }
    };

    Ok(())
}

fn send_warning_notification(
    bot: &Bot,
    chat_id: i64,
    msg: String,
) -> teloxide::requests::JsonRequest<SendMessage> {
    log::warn!("Warning {msg}");
    bot.send_message(ChatId(chat_id), format!("❗️❗️❗️{msg}❗️❗️❗️"))
}
