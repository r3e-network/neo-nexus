use super::{parse_designated, ROLE_MANAGEMENT_HASH};
use serde_json::json;

/// The two real mainnet committee keys used as fixtures, hex and the base64
/// ByteString form `getDesignatedByRole` actually returns.
const KEY_HEX: &str = "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c";
const KEY_B64: &str = "A7IJ/U9TpxcOpERODLCmu2pTwr0BaSaYnPhfmw+6F6cM";
const OTHER_HEX: &str = "02df48f60e8f3e01c48ff40b9b7f1310d7a8b2a193188befe1c2e3df740e895093";
const OTHER_B64: &str = "At9I9g6PPgHEj/QLm38TENeosqGTGIvv4cLj33QOiVCT";

/// The RoleManagement hash is fixed by the protocol and identical on every Neo
/// N3 network. Getting it wrong would query a contract that does not exist.
#[test]
fn the_role_management_hash_is_the_native_contract() {
    assert_eq!(
        ROLE_MANAGEMENT_HASH,
        "0x49cf4e5378ffcd4dec034fd98a174c5491e395e2"
    );
}

/// The contract returns base64 ByteStrings; operators read hex. A manager that
/// compared the node's hex key against a base64 list would report every node as
/// undesignated.
#[test]
fn base64_byte_strings_are_decoded_to_hex_public_keys() {
    let item = json!({
        "type": "Array",
        "value": [
            { "type": "ByteString", "value": KEY_B64 },
            { "type": "ByteString", "value": OTHER_B64 },
        ]
    });
    assert_eq!(parse_designated(&item).unwrap(), vec![KEY_HEX, OTHER_HEX]);
}

/// A role nobody holds is a legitimate answer, not a failure — P2PNotary is
/// undesignated on mainnet.
#[test]
fn an_empty_designation_is_a_valid_answer() {
    let item = json!({ "type": "Array", "value": [] });
    assert_eq!(parse_designated(&item).unwrap(), Vec::<String>::new());
}

#[test]
fn a_non_array_answer_is_rejected() {
    let item = json!({ "type": "Integer", "value": "7" });
    assert!(parse_designated(&item).is_err());
}

/// A truncated or padded key would silently never match the node's own, so the
/// length is checked rather than trusted.
#[test]
fn a_key_that_is_not_a_compressed_point_is_rejected() {
    let item = json!({
        "type": "Array",
        "value": [{ "type": "ByteString", "value": "AAAA" }]
    });
    let error = parse_designated(&item).expect_err("a 3-byte key is not a public key");
    assert!(error.message().contains("33-byte"));
}

#[test]
fn a_non_base64_key_is_rejected() {
    let item = json!({
        "type": "Array",
        "value": [{ "type": "ByteString", "value": "not base64!!" }]
    });
    assert!(parse_designated(&item).is_err());
}
