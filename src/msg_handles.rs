use std::ffi::OsString;
use std::path::Path;
use std::path::MAIN_SEPARATOR;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::MediaKind;
use teloxide::types::MessageKind;
use teloxide::ApiError;
use teloxide::RequestError;
use tokio::fs;
use wait_timeout::ChildExt;

use crate::states::SqliteState;

pub async fn entry(bot: Bot, states: Arc<SqliteState>, msg: Message) -> ResponseResult<()> {
    if let Some(chat_id) = states.get_auth_chat_id() {
        // already auth
        if chat_id != msg.chat.id.0 {
            let username = match msg.from().unwrap().username.clone() {
                Some(name) => name,
                None => String::new(),
            };

            let warning_msg = format!("{username} is sending msg!");

            log::warn!("{warning_msg}");
            bot.send_message(ChatId(chat_id), "❗️❗️❗️{warning_msg}❗️❗️❗️")
                .await?;
        } else {
            let mut ret_text: String;
            if let MessageKind::Common(msg_common) = msg.kind {
                let current_path = states.query_current_path(chat_id).unwrap_or("/".into());

                if let MediaKind::Text(text) = msg_common.media_kind {
                    // simple media text
                    let input_text = text.text;
                    let cd_str = "cd ";
                    if input_text.starts_with(cd_str) {
                        // change current path
                        ret_text = match input_text.strip_prefix(cd_str) {
                            Some(arg_path) => {
                                let mut new_path = if !arg_path.starts_with(MAIN_SEPARATOR) {
                                    format!(
                                        "{}{}{}",
                                        current_path.to_str().unwrap(),
                                        MAIN_SEPARATOR,
                                        arg_path
                                    )
                                } else {
                                    String::from(arg_path)
                                };

                                if Path::new(new_path.as_str()).exists() {
                                    let pb = std::fs::canonicalize(new_path.as_str()).unwrap();
                                    new_path = String::from(pb.to_str().unwrap());
                                    states.update_current_path(&OsString::from(new_path.as_str()));
                                    format!("current path changed: {new_path}")
                                } else {
                                    format!("path not exists: {new_path}")
                                }
                            }
                            None => {
                                format!("arg path is not valid")
                            }
                        }
                    } else {
                        // execute cmd
                        let mut cmd = Command::new("bash");

                        let mut child = cmd
                            .current_dir(&current_path)
                            .arg("-c")
                            .arg(&input_text)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .unwrap();

                        let time_out = Duration::from_secs(3);
                        ret_text = match child.wait_timeout(time_out).unwrap() {
                            Some(exit_status) => {
                                let output = child.wait_with_output().unwrap();
                                if exit_status.success() {
                                    String::from_utf8(output.stdout).unwrap()
                                } else {
                                    String::from_utf8(output.stderr).unwrap()
                                }
                            }
                            None => {
                                // child hasn't exited yet
                                child.kill().unwrap();
                                child.wait().unwrap();
                                "💢 exe timeout ❗️❓".into()
                            }
                        };

                        if ret_text.is_empty() {
                            ret_text = format!("⭕️ exe succeed cmd: {input_text}")
                        }
                    }
                } else if let MediaKind::Document(doc) = msg_common.media_kind {
                    // receive doc
                    let file_name = doc.document.file_name.unwrap();

                    bot.send_message(msg.chat.id, format!("⬇️ downloading file: {file_name}"))
                        .await?;
                    let res_getfile = bot.get_file(doc.document.file.id).send();
                    let file_info = match res_getfile.await {
                        Ok(f) => f,
                        Err(err) => {
                            if let RequestError::Api(ApiError::Unknown(un_err)) = err {
                                bot.send_message(msg.chat.id, un_err).await?;
                            }
                            return Err(RequestError::Api(ApiError::Unknown(
                                "file is too big".into(),
                            )));
                        }
                    };
                    let new_file_path = format!(
                        "{}{}{}",
                        current_path.into_string().unwrap(),
                        MAIN_SEPARATOR,
                        file_name
                    );
                    let mut dst = fs::File::create(new_file_path.as_str()).await?;

                    bot.download_file(&file_info.path, &mut dst).await?;
                    ret_text = format!("✅️ file saved to: {new_file_path}");
                } else {
                    ret_text = format!("unsupported media kind: {:?}", msg_common.media_kind);
                }
            } else {
                ret_text = format!("unsupported msg type");
            }

            bot.send_message(msg.chat.id, ret_text).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_process() {
        let output = Command::new("bash").arg("-c").arg("pwd").output().unwrap();
        println!("output :{}", String::from_utf8(output.stdout).unwrap());
    }
}
