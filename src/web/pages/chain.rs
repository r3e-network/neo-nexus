//! Per-node chain state, read live from the node's own RPC endpoint.
//!
//! Who sits on the committee, who produces the next round, and where a
//! candidate's vote stands are chain facts a node's duties depend on that no
//! configuration shows. The signer page set the precedent for live reads
//! during a page render; this one asks the family guard first, because every
//! method here is Neo N3 native and a Neo X endpoint would answer each with
//! "method not found" instead of one clear sentence.

use std::time::Duration;

use axum::{
    extract::{Path, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::{
        node_chain::{governance_snapshot, GovernanceSnapshot},
        operations::node_rpc_endpoint,
    },
    types::ChainFamily,
    web::{html, WebState},
};

/// The same three seconds the CLI chain reads use: an operator waiting on a
/// page should get an answer or a failure quickly.
const CHAIN_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn node_chain(
    State(state): State<WebState>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let node = match state
        .repository
        .list_nodes()
        .ok()
        .and_then(|nodes| nodes.into_iter().find(|node| node.id == id))
    {
        Some(node) => node,
        None => return Redirect::to("/nodes").into_response(),
    };

    let body = if node.node_type.family() == ChainFamily::NeoX {
        html::notice(
            "warn",
            "Chain state reads exist only on Neo N3, and a Neo X endpoint speaks Ethereum JSON-RPC instead. Governance and designation do not apply to this node.",
        )
    } else {
        let endpoint = node_rpc_endpoint(&node);
        let loaded =
            tokio::task::spawn_blocking(move || governance_snapshot(&endpoint, CHAIN_TIMEOUT))
                .await;
        match loaded {
            Ok(Ok(snapshot)) => render_snapshot(&snapshot),
            Ok(Err(error)) => html::notice("warn", error.message()),
            Err(_) => html::notice("danger", "Chain state request did not finish."),
        }
    };

    let encoded = html::urlencoding_lite(&id);
    let back = format!(r#"<p><a href="/nodes/{encoded}">← Node</a></p>"#);
    Html(html::layout(
        "Chain state",
        "nodes",
        &html::flash(query.as_deref()),
        &format!("{back}\n<h1>Chain state</h1>\n{body}"),
    ))
    .into_response()
}

fn render_snapshot(snapshot: &GovernanceSnapshot) -> String {
    let committee = html::table(
        &["#", "Public key"],
        &snapshot
            .committee
            .iter()
            .enumerate()
            .map(|(index, key)| html::row(&[html::cell(&(index + 1).to_string()), html::cell(key)]))
            .collect::<Vec<_>>(),
    );
    let validators = html::table(
        &["#", "Public key"],
        &snapshot
            .next_validators
            .iter()
            .enumerate()
            .map(|(index, key)| html::row(&[html::cell(&(index + 1).to_string()), html::cell(key)]))
            .collect::<Vec<_>>(),
    );
    let candidates = if snapshot.candidates.is_empty() {
        html::note("No registered candidates.")
    } else {
        html::table(
            &["Public key", "Votes (NEO)"],
            &snapshot
                .candidates
                .iter()
                .map(|candidate| {
                    html::row(&[
                        html::cell(&candidate.public_key),
                        html::cell(&candidate.votes.to_string()),
                    ])
                })
                .collect::<Vec<_>>(),
        )
    };
    format!(
        "<h2>Committee</h2>\n{committee}\n<h2>Next validators</h2>\n{validators}\n<h2>Candidates</h2>\n{candidates}"
    )
}
