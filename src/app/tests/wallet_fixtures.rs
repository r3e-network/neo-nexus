pub(super) const VALID_NEP6_CONTRACT_PUBLIC_KEY: &str =
    "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0";

pub(super) fn valid_nep6_wallet_json() -> String {
    serde_json::json!({
            "name": "NeoNexus validator wallet",
            "version": "3.0",
            "scrypt": {
                "n": 16384,
                "r": 8,
                "p": 8
            },
            "accounts": [
                {
                    "address": "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq",
                    "label": "validator-1",
                    "isDefault": true,
                    "lock": false,
                    "key": "6PYWB8m1bCnu5bQkRUKAwbZp2BHNvQ3BQRLbpLdTuizpyLkQPSZbtZfoxx",
                    "contract": {
                        "script": "21036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0ac",
                        "parameters": [],
                        "deployed": false
                    },
                    "extra": null
                }
            ],
            "extra": null
        })
        .to_string()
}
