use teloxide::prelude::*;

async fn telegram_request(
    input: String,
    tx: axum::extract::Extension<
        tokio::sync::mpsc::UnboundedSender<Result<teloxide::types::Update, String>>,
    >,
) -> impl axum::response::IntoResponse {
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

    axum::http::StatusCode::OK
}

pub async fn webhook(
    bot: AutoSend<Bot>,
) -> impl teloxide::dispatching::update_listeners::UpdateListener<String> {
    let bind_address = Result::unwrap_or(std::env::var("BIND_ADDRESS"), "0.0.0.0".to_string());
    let bind_port: u16 = std::env::var("BIND_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("BIND_PORT value has to be an integer");

    let host = std::env::var("HOST").expect("HOST env variable missing");
    let path = match std::env::var("WEBHOOK_URI") {
        Ok(path) => path,
        Err(_e) => std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing"),
    };
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url.parse().unwrap())
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let app = axum::Router::new()
        .route(
            format!("/{}", path).as_str(),
            axum::routing::post(telegram_request),
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(tower_http::add_extension::AddExtensionLayer::new(tx))
                .into_inner(),
        );

    let server_address: std::net::SocketAddr = format!("{}:{}", bind_address, bind_port)
        .parse()
        .expect("Unable to parse socket address");

    tokio::spawn(async move {
        axum::Server::bind(&server_address)
            .serve(app.into_make_service())
            .await
            .expect("Axum server error")
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    let (stop_token, _) = teloxide::dispatching::stop_token::AsyncStopToken::new_pair();
    teloxide::dispatching::update_listeners::StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, teloxide::dispatching::stop_token::AsyncStopToken)| state.1.clone(),
    )
}
