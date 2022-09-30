mod commands;
mod logging;
mod parameters;
mod webhook;

use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    logging::init_logger();
    log::info!("Starting Supapro bot");

    let parameters = std::sync::Arc::new(parameters::Parameters::new());

    let bot = Bot::from_env().auto_send();

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<commands::Command>()
            .endpoint(commands::command_handler),
    );

    if !parameters.is_webhook_mode_enabled {
        log::info!("Webhook deleted");
        bot.delete_webhook().await.expect("Cannot delete a webhook");
    }

    let mut bot_dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![parameters.clone()])
        .default_handler(|_| async move {})
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build();

    if parameters.is_webhook_mode_enabled {
        log::info!("Webhook mode activated");
        let rx = webhook::webhook(bot);
        bot_dispatcher
            .dispatch_with_listener(
                rx.await,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
    } else {
        log::info!("Long polling mode activated");
        bot_dispatcher.dispatch().await;
    }
}
