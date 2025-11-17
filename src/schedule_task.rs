use teloxide::prelude::*;

pub async fn entry(bot: &Bot, chat_id: i64) -> Result<(), String> {

    let bot_result = bot.send_message(ChatId(chat_id), "schedule msg").await;
    if let Err(e) = bot_result {
        return Err(e.to_string());
    }

    Ok(())
}