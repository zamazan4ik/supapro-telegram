pub struct Parameters {
    pub bot_name: String,
    pub pro_chat_id: i64,
    pub pro_chat_username: String,
    pub supapro_chat_id: i64,
    pub supapro_chat_username: String,
    pub is_webhook_mode_enabled: bool,
}

impl Parameters {
    pub fn new() -> Self {
        let bot_name = std::env::var("BOT_NAME").expect("BOT_NAME env var is not specified");

        let pro_chat_id: i64 = std::env::var("PRO_CHAT_ID")
            .expect("SUPAPRO_CHAT_ID env var is not specified")
            .parse()
            .expect("Cannot parse as i64");

        let pro_chat_username: String =
            std::env::var("PRO_CHAT_USERNAME").expect("PRO_CHAT_USERNAME env var is not specified");

        let supapro_chat_id: i64 = std::env::var("SUPAPRO_CHAT_ID")
            .expect("SUPAPRO_CHAT_ID env var is not specified")
            .parse()
            .expect("Cannot parse as i64");

        let supapro_chat_username: String = std::env::var("SUPAPRO_CHAT_USERNAME")
            .expect("SUPAPRO_CHAT_USERNAME env var is not specified");

        let is_webhook_mode_enabled: bool = std::env::var("WEBHOOK_MODE")
            .unwrap_or("false".to_string())
            .parse()
            .expect(
                "Cannot convert WEBHOOK_MODE to bool. Applicable values are only \"true\" or \"false\"",
            );

        Self {
            bot_name,
            pro_chat_id,
            pro_chat_username,
            supapro_chat_id,
            supapro_chat_username,
            is_webhook_mode_enabled,
        }
    }
}
