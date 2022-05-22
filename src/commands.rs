use crate::parameters;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands, utils::html};

#[derive(Clone, BotCommands)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "display info about bot.")]
    About,
    #[command(rename = "s", description = "forward a message to supapro.")]
    Supapro,
    #[command(rename = "p", description = "forward a message to pro.")]
    Pro,
}

pub async fn command_handler(
    msg: Message,
    bot: AutoSend<Bot>,
    command: Command,
    parameters: std::sync::Arc<parameters::Parameters>,
) -> anyhow::Result<()> {
    static HELP_TEXT: &str = "Я просто помогаю с поддержанием порядка в С++ чатах!";

    static ABOUT_TEXT: &str = "По всем замечаниям или предложениям обращаться по этому адресу:\
        https://github.com/ZaMaZaN4iK/supapro-telegram . Спасибо!";

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, HELP_TEXT)
                .reply_to_message_id(msg.id)
                .await?;
        }
        Command::About => {
            bot.send_message(msg.chat.id, ABOUT_TEXT)
                .reply_to_message_id(msg.id)
                .await?;
        }
        Command::Supapro => {
            process_forward_command(
                msg,
                bot,
                teloxide::types::ChatId(parameters.supapro_chat_id),
                parameters.supapro_chat_username.as_str(),
            )
            .await?;
        }
        Command::Pro => {
            process_forward_command(
                msg,
                bot,
                teloxide::types::ChatId(parameters.pro_chat_id),
                parameters.pro_chat_username.as_str(),
            )
            .await?;
        }
    };

    Ok(())
}

async fn process_forward_command(
    msg: Message,
    bot: AutoSend<Bot>,
    chat_id_to: teloxide::types::ChatId,
    chat_username_to: &str,
) -> anyhow::Result<()> {
    if let Some(user) = msg.from() {
        let status = bot.get_chat_member(msg.chat.id, user.id).await?.status();

        if status == teloxide::types::ChatMemberStatus::Owner
            || status == teloxide::types::ChatMemberStatus::Administrator
        {
            // 1) Forward reply of this command to a target chat
            // 2) Delete it from the current chat
            // 3) Send a message to an original author with a notification
            // 4) Delete message with command from current chat

            if let Some(reply) = msg.reply_to_message() {
                let forwarded_msg = bot
                    .forward_message(chat_id_to, reply.chat.id, reply.id)
                    .await?;

                bot.delete_message(reply.chat.id, reply.id).await?;

                let chat_ref = format!("@{}", chat_username_to);

                let forwarded_msg_ref = match forwarded_msg.url() {
                    Some(url) => html::link(&url.to_string(), &chat_ref),
                    None => chat_ref,
                };

                let response = if let Some(original_author) = reply.from() {
                    let author_ref = original_author
                        .mention()
                        .unwrap_or(original_author.full_name());

                    format!(
                        "{}, Ваш вопрос перемещён в чат {}. Там Вам с радостью помогут решить проблему :)",
                        html::escape(&author_ref),
                        forwarded_msg_ref
                    )
                } else {
                    format!(
                        "Вопрос перемещён в чат {}. Там с радостью помогут решить проблему :)",
                        forwarded_msg_ref
                    )
                };

                bot.send_message(msg.chat.id, response)
                    .parse_mode(ParseMode::Html)
                    .await?;

                bot.delete_message(msg.chat.id, msg.id).await?;
            } else {
                static MISSING_REPLY: &str = "Пожалуйста, ответьте на сообщение. Спасибо!";
                bot.send_message(msg.chat.id, MISSING_REPLY)
                    .reply_to_message_id(msg.id)
                    .await?;
            }
        }
    }

    Ok(())
}
