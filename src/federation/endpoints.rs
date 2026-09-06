pub const PUBLIC_STATUS_PATH: &str = "/api/public/status";
pub fn public_endpoint_url(base_url: &str, public_path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        public_path.trim_start_matches('/')
    )
}
