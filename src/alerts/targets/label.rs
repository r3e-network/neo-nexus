use url::Url;

pub fn alert_target_label(raw: &str) -> String {
    Url::parse(raw).map_or_else(
        |_| "webhook".to_string(),
        |url| {
            let host = url.host_str().unwrap_or("webhook");
            match url.port() {
                Some(port) => format!("{}://{}:{port}", url.scheme(), host),
                None => format!("{}://{}", url.scheme(), host),
            }
        },
    )
}
