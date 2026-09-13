use crate::signer_client::{AuditRow, Caller, KeyPublic};
use crate::web::Admin;

pub const AUDIT_ROWS: usize = 50;

pub struct Inventory {
    pub keys: Vec<KeyPublic>,
    pub callers: Vec<Caller>,
    pub audit: Vec<AuditRow>,
}

pub fn inventory(admin: &Admin) -> anyhow::Result<Inventory> {
    let credentials = admin.credentials()?;
    let client = admin.client();
    Ok(Inventory {
        keys: client.list_keys(&credentials)?.into_parts()?,
        callers: client.list_callers(&credentials)?.into_parts()?,
        audit: client
            .list_audit(&credentials, None, Some(AUDIT_ROWS))?
            .into_parts()?,
    })
}
