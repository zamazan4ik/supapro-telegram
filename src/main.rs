mod commands;
mod logging;
mod parameters;
mod webhook;

use teloxide::{prelude::*, utils::command::BotCommand};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    logging::init_logger();
    log::info!("Starting Supapro bot");

    let parameters = std::sync::Arc::new(parameters::Parameters::new());
    let bot_parameters = parameters.clone();

    let bot = Bot::from_env();

    let mut bot_dispatcher = Dispatcher::new(bot.clone()).messages_handler(
        move |rx: DispatcherHandlerRx<Bot, Message>| {
            let rx = UnboundedReceiverStream::new(rx);
            rx.for_each(move |message| {
                let parameters = bot_parameters.clone();
                async move {
                    let message_text = match message.update.text() {
                        Some(x) => x,
                        None => return,
                    };

                    // Handle commands. If command cannot be parsed - continue processing
                    match commands::Command::parse(message_text, &parameters.bot_name) {
                        Ok(command) => {
                            commands::command_answer(&message, command, parameters)
                                .await
                                .log_on_error()
                                .await;
                            ()
                        }
                        Err(_) => (),
                    };
                }
            })
        },
    );

    if parameters.is_webhook_mode_enabled {
        log::info!("Webhook mode activated");
        let rx = webhook::webhook(bot);
        bot_dispatcher
            .dispatch_with_listener(
                rx.await,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
        return;
    }

    log::info!("Long polling mode activated");
    bot.delete_webhook()
        .send()
        .await
        .expect("Cannot delete a webhook");
    bot_dispatcher.dispatch().await;
}
