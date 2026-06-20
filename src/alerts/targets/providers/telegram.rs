use anyhow::{Context, Result};
use url::Url;

pub(in crate::alerts) struct TelegramTarget {
    pub(in crate::alerts) chat_id: String,
}

pub(in crate::alerts) fn telegram_target(raw: &str) -> Result<TelegramTarget> {
    let url = Url::parse(raw).context("Telegram Bot API URL is invalid")?;
    let path = url.path().to_ascii_lowercase();
    if !path.contains("/bot") || !path.ends_with("/sendmessage") {
        anyhow::bail!("Telegram alert target must use a Bot API sendMessage URL");
    }
    let chat_id = url
        .query_pairs()
        .find(|(key, _)| key == "chat_id")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .context("Telegram alert target must include a chat_id query parameter")?;
    Ok(TelegramTarget { chat_id })
}
