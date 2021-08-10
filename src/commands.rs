use crate::parameters;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommand, utils::html};

#[derive(BotCommand)]
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

pub async fn command_answer(
    cx: &UpdateWithCx<Bot, Message>,
    command: Command,
    parameters: std::sync::Arc<parameters::Parameters>,
) -> anyhow::Result<()> {
    static HELP_TEXT: &str = "Я просто помогаю с поддержанием порядка в С++ чатах!";

    static ABOUT_TEXT: &str = "По всем замечаниям или предложениям обращаться по этому адресу:\
        https://github.com/ZaMaZaN4iK/supapro-telegram . Спасибо!";

    match command {
        Command::Help => {
            cx.reply_to(HELP_TEXT).send().await?;
        }
        Command::About => {
            cx.reply_to(ABOUT_TEXT).send().await?;
        }
        Command::Supapro => {
            process_forward_command(
                &cx,
                parameters.supapro_chat_id,
                parameters.supapro_chat_username.as_str(),
            )
            .await?;
        }
        Command::Pro => {
            process_forward_command(
                &cx,
                parameters.pro_chat_id,
                parameters.pro_chat_username.as_str(),
            )
            .await?;
        }
    };

    Ok(())
}

async fn process_forward_command(
    cx: &UpdateWithCx<Bot, Message>,
    chat_id_to: i64,
    chat_username_to: &str,
) -> anyhow::Result<()> {
    if let Some(user) = cx.update.from() {
        let status = cx
            .requester
            .get_chat_member(cx.update.chat_id(), user.id)
            .send()
            .await?
            .status();

        if status == teloxide::types::ChatMemberStatus::Owner
            || status == teloxide::types::ChatMemberStatus::Administrator
        {
            // 1) Forward reply of this command to a target chat
            // 2) Delete it from the current chat
            // 3) Send a message to an original author with a notification
            // 4) Delete message with command from current chat

            if let Some(reply) = cx.update.reply_to_message() {
                let forwarded_msg = cx
                    .requester
                    .forward_message(chat_id_to, reply.chat.id, reply.id)
                    .send()
                    .await?;

                cx.requester
                    .delete_message(reply.chat.id, reply.id)
                    .send()
                    .await?;

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

                cx.requester
                    .send_message(cx.update.chat_id(), response)
                    .parse_mode(ParseMode::Html)
                    .send()
                    .await?;

                cx.requester
                    .delete_message(cx.update.chat.id, cx.update.id)
                    .send()
                    .await?;
            } else {
                static MISSING_REPLY: &str = "Пожалуйста, ответьте на сообщение. Спасибо!";
                cx.reply_to(MISSING_REPLY).send().await?;
            }
        }
    }

    Ok(())
}
