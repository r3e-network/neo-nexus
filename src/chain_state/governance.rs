//! Committee, validators and the candidate vote.
//!
//! All three are plain RPC reads. Registering a candidate or voting are signed,
//! GAS-bearing transactions that NeoNexus does not perform: what it can do is
//! show an operator exactly where their key stands so they know whether to.

use std::time::Duration;

use serde_json::{json, Value};

use super::{
    model::{CandidateStanding, ChainQueryError, GovernanceSnapshot},
    rpc::{agent, call},
};

/// Reads the committee, the next round's validators, and the candidate vote.
pub fn governance_snapshot(
    endpoint: &str,
    timeout: Duration,
) -> Result<GovernanceSnapshot, ChainQueryError> {
    let agent = agent(timeout);
    let committee = public_key_list(
        &call(&agent, endpoint, "getcommittee", json!([]))?,
        "getcommittee",
    )?;
    let next_validators = public_key_list(
        &call(&agent, endpoint, "getnextblockvalidators", json!([]))?,
        "getnextblockvalidators",
    )?;
    let candidates = parse_candidates(&call(&agent, endpoint, "getcandidates", json!([]))?)?;
    Ok(GovernanceSnapshot {
        committee,
        next_validators,
        candidates,
    })
}

/// `getcommittee` returns bare hex strings; `getnextblockvalidators` returns
/// objects carrying a `publickey`. Both shapes are accepted so one helper
/// serves both, rather than failing on a difference that does not matter here.
pub(super) fn public_key_list(
    result: &Value,
    method: &'static str,
) -> Result<Vec<String>, ChainQueryError> {
    let Some(entries) = result.as_array() else {
        return Err(ChainQueryError::Unexpected(format!(
            "{method} did not return an array"
        )));
    };
    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("publickey").and_then(Value::as_str))
                .map(str::to_string)
                .ok_or_else(|| {
                    ChainQueryError::Unexpected(format!("{method} returned a malformed entry"))
                })
        })
        .collect()
}

/// `getcandidates` returns `{publickey, votes}` with the vote count as a
/// decimal *string*, because NEO totals exceed what JSON numbers carry safely.
pub(super) fn parse_candidates(result: &Value) -> Result<Vec<CandidateStanding>, ChainQueryError> {
    let Some(entries) = result.as_array() else {
        return Err(ChainQueryError::Unexpected(
            "getcandidates did not return an array".to_string(),
        ));
    };
    let mut candidates: Vec<CandidateStanding> = entries
        .iter()
        .map(|entry| {
            let public_key = entry
                .get("publickey")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ChainQueryError::Unexpected("a candidate has no public key".to_string())
                })?
                .to_string();
            let votes = entry
                .get("votes")
                .and_then(|votes| {
                    votes
                        .as_i64()
                        .or_else(|| votes.as_str().and_then(|text| text.parse().ok()))
                })
                .unwrap_or_default();
            Ok(CandidateStanding { public_key, votes })
        })
        .collect::<Result<_, ChainQueryError>>()?;
    candidates.sort_by(|left, right| right.votes.cmp(&left.votes));
    Ok(candidates)
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/governance/tests.rs"]
mod tests;
