use super::{parse_candidates, public_key_list};
use serde_json::json;

/// `getcommittee` returns bare hex strings; `getnextblockvalidators` returns
/// objects carrying a `publickey`. Both are real, and both must parse.
#[test]
fn both_public_key_shapes_are_accepted() {
    let bare = json!(["03aa", "02bb"]);
    assert_eq!(
        public_key_list(&bare, "getcommittee").unwrap(),
        vec!["03aa", "02bb"]
    );

    let wrapped = json!([
        { "publickey": "03aa", "votes": "100" },
        { "publickey": "02bb", "votes": "50" },
    ]);
    assert_eq!(
        public_key_list(&wrapped, "getnextblockvalidators").unwrap(),
        vec!["03aa", "02bb"]
    );
}

#[test]
fn a_malformed_entry_is_reported_not_skipped() {
    let entries = json!(["03aa", { "novotes": true }]);
    assert!(public_key_list(&entries, "getcommittee").is_err());
}

/// NEO vote totals exceed what a JSON number carries safely, so the node sends
/// them as decimal strings. Parsing only numbers would read every candidate as
/// having zero votes.
#[test]
fn vote_totals_arrive_as_strings_and_are_parsed() {
    let result = json!([
        { "publickey": "03aa", "votes": "1000000" },
        { "publickey": "02bb", "votes": "2500000" },
    ]);
    let candidates = parse_candidates(&result).unwrap();
    assert_eq!(candidates[0].public_key, "02bb");
    assert_eq!(candidates[0].votes, 2_500_000);
    assert_eq!(candidates[1].votes, 1_000_000);
}

/// Highest first, so the top of the list is the top of the vote.
#[test]
fn candidates_are_ordered_by_votes() {
    let result = json!([
        { "publickey": "03aa", "votes": "10" },
        { "publickey": "02bb", "votes": "900" },
        { "publickey": "02cc", "votes": "40" },
    ]);
    let votes: Vec<i64> = parse_candidates(&result)
        .unwrap()
        .iter()
        .map(|candidate| candidate.votes)
        .collect();
    assert_eq!(votes, [900, 40, 10]);
}

/// A registered candidate with no votes yet is normal; it must not be dropped
/// or turned into an error.
#[test]
fn a_candidate_without_votes_is_kept_at_zero() {
    let result = json!([{ "publickey": "03aa" }]);
    let candidates = parse_candidates(&result).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].votes, 0);
}

#[test]
fn a_candidate_without_a_key_is_rejected() {
    let result = json!([{ "votes": "10" }]);
    assert!(parse_candidates(&result).is_err());
}

#[test]
fn an_empty_candidate_list_is_valid() {
    assert!(parse_candidates(&json!([])).unwrap().is_empty());
}
