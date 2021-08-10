use tokio::sync::mpsc;

use actix_web::middleware;
use actix_web::web;
use actix_web::{App, HttpResponse, HttpServer, Responder};
use std::env;
use teloxide::dispatching::{stop_token::AsyncStopToken, update_listeners::StatefulListener};
use teloxide::prelude::*;
use teloxide::types::Update;
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn telegram_request(
    tx: web::Data<mpsc::UnboundedSender<Result<Update, String>>>,
    input: String,
) -> impl Responder {
    let try_parse = match serde_json::from_str(&input) {
        Ok(update) => Ok(update),
        Err(error) => {
            log::error!(
                "Cannot parse an update.\nError: {:?}\nValue: {}\n\
                       This is a bug in teloxide, please open an issue here: \
                       https://github.com/teloxide/teloxide/issues.",
                error,
                input
            );
            Err(error)
        }
    };
    if let Ok(update) = try_parse {
        tx.send(Ok(update))
            .expect("Cannot send an incoming update from the webhook")
    }

    HttpResponse::Ok()
}

pub async fn webhook(
    bot: Bot,
) -> impl teloxide::dispatching::update_listeners::UpdateListener<String> {
    let bind_address = Result::unwrap_or(env::var("BIND_ADDRESS"), "0.0.0.0".to_string());
    let bind_port: u16 = env::var("BIND_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("BIND_PORT value has to be an integer");

    let host = env::var("HOST").expect("HOST env variable missing");
    let path = match env::var("WEBHOOK_URI") {
        Ok(path) => path,
        Err(_e) => env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing"),
    };
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url.parse().unwrap())
        .send()
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let sender_channel_data: web::Data<mpsc::UnboundedSender<Result<Update, String>>> =
        web::Data::new(tx);

    tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(sender_channel_data.clone())
                .route(path.as_str(), web::post().to(telegram_request))
        })
        .bind(format!("{}:{}", bind_address, bind_port))
        .unwrap()
        .run()
    });

    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    let (stop_token, _) = AsyncStopToken::new_pair();
    StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, AsyncStopToken)| state.1.clone(),
    )
}
