//! Signer overview, tabs routing, custody status, and inventory rendering.

mod audit;
mod callers;
mod model;
mod notices;
mod panels;

pub use audit::audit_table;
#[cfg(test)]
pub use callers::{new_caller_form, new_workload_caller_form};
pub use model::{inventory, Inventory};
pub use notices::{custody_notice, render_local_wallet};
pub use panels::{key_actions, key_chain_identity};

use crate::web::html;

use super::{tabs, SignerTab};
use model::AUDIT_ROWS;
use panels::{key_table, new_key_form, overview_panel, page_head};

pub fn closed_page(banner: &str, error: &anyhow::Error, tab: SignerTab) -> String {
    format!(
        "{}{}{}{}",
        page_head(),
        tabs(tab),
        banner,
        html::empty_state(
            "Custody inventory unavailable",
            &format!("No inventory is shown because none could be read: {error}"),
            "",
        )
    )
}

pub fn render_body(
    banner: &str,
    inventory: &Inventory,
    secret: Option<&str>,
    tab: SignerTab,
) -> String {
    let Inventory {
        keys,
        callers,
        audit,
    } = inventory;
    let content = match tab {
        SignerTab::Overview => overview_panel(keys, callers, audit),
        SignerTab::Keys => format!(
            r#"<div class="section-head"><h2>Custody keys</h2><span class="muted">Private material never enters NeoNexus.</span></div>{}{}"#,
            key_table(keys),
            new_key_form()
        ),
        SignerTab::Callers => format!(
            r#"<div class="section-head"><h2>Caller access</h2><span class="muted">Least-privilege identities allowed to ask custody.</span></div>{}{}{}{}"#,
            callers::caller_table(callers),
            callers::new_caller_form(keys),
            callers::new_workload_caller_form(keys),
            callers::api_reference()
        ),
        SignerTab::Audit => format!(
            r#"<div class="section-head"><h2>Custody audit</h2><span class="muted">Latest {AUDIT_ROWS} service decisions.</span></div>{}"#,
            audit::audit_table(audit, keys, callers)
        ),
    };
    let secret = secret.map_or_else(String::new, |token| {
        format!(
            r#"<div class="notice warn" role="status"><strong>New caller token — shown once, not recoverable afterwards.</strong><code class="secret-value">{}</code></div>"#,
            html::escape(token)
        )
    });
    format!(
        "{}{}{}{}{}",
        page_head(),
        tabs(tab),
        banner,
        secret,
        content
    )
}
