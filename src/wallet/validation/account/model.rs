#[derive(Default)]
pub(in crate::wallet::validation) struct AccountStats {
    pub(in crate::wallet::validation) encrypted: bool,
    pub(in crate::wallet::validation) default: bool,
    pub(in crate::wallet::validation) watch_only: bool,
    pub(in crate::wallet::validation) contract_public_key: Option<String>,
}
