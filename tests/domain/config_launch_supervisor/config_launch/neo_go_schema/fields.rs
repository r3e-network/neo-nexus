//! The verified neo-go field names, read from `pkg/config/*.go` on
//! nspcc-dev/neo-go and cross-checked against the shipped
//! `config/protocol.mainnet.yml`. Adding a name here without that check
//! defeats the point of the contract that consumes it.

/// `ProtocolConfiguration` fields (pkg/config/protocol_config.go).
pub(super) const PROTOCOL_FIELDS: &[&str] = &[
    "CommitteeHistory",
    "Genesis",
    "Hardforks",
    "InitialGASSupply",
    "Magic",
    "MaxBlockSize",
    "MaxBlockSystemFee",
    "MaxTimePerBlock",
    "MaxTraceableBlocks",
    "MaxTransactionsPerBlock",
    "MaxValidUntilBlockIncrement",
    "MemPoolSize",
    "NeoFSStateSyncExtensions",
    "P2PNotaryRequestPayloadPoolSize",
    "P2PSigExtensions",
    "P2PStateExchangeExtensions",
    "ReservedAttributes",
    "SeedList",
    "StandbyCommittee",
    "StateRootInHeader",
    "StateSyncInterval",
    "TimePerBlock",
    "ValidatorsCount",
    "ValidatorsHistory",
    "VerifyTransactions",
];

/// `ApplicationConfiguration` fields, including the inlined `Ledger` and
/// `Logger` groups (pkg/config/application_config.go, logger.go, ledger_config.go).
pub(super) const APPLICATION_FIELDS: &[&str] = &[
    "ArchivalNodesSync",
    "Consensus",
    "DBConfiguration",
    "GarbageCollectionPeriod",
    "KeepOnlyLatestState",
    "LogEncoding",
    "LogLevel",
    "LogPath",
    "LogTimestamp",
    "NeoFSBlockFetcher",
    "NeoFSStateFetcher",
    "Oracle",
    "P2P",
    "P2PNotary",
    "Pprof",
    "Prometheus",
    "RPC",
    "Relay",
    "RemoveUntraceableBlocks",
    "RemoveUntraceableHeaders",
    "SaveStorageBatch",
    "SkipBlockVerification",
    "StateRoot",
    "TrustedHeader",
];

/// Nested sections, keyed by the section name as it appears in the YAML.
pub(super) const NESTED_FIELDS: &[(&str, &[&str])] = &[
    (
        "DBConfiguration",
        &["Type", "LevelDBOptions", "BoltDBOptions"],
    ),
    (
        "P2P",
        &[
            "Addresses",
            "AttemptConnPeers",
            "BroadcastFactor",
            "BroadcastTxsBatchDelay",
            "DialTimeout",
            "DisableCompression",
            "ExtensiblePoolSize",
            "MaxPeers",
            "MinPeers",
            "PingInterval",
            "PingTimeout",
            "ProtoTickInterval",
        ],
    ),
    (
        "RPC",
        &[
            "Addresses",
            "DirectRelay",
            "Enabled",
            "EnableCORSWorkaround",
            "MaxGasInvoke",
            "MaxFindResultItems",
            "MaxFindStoragePageSize",
            "MaxIteratorResultItems",
            "MaxNEP11Tokens",
            "MaxRequestBodyBytes",
            "MaxRequestHeaderBytes",
            "MaxWebSocketClients",
            "MaxWebSocketFeeds",
            "MempoolSubscriptionsEnabled",
            "SessionBackedByMPT",
            "SessionEnabled",
            "SessionExpansionEnabled",
            "SessionLifetime",
            "SessionPoolSize",
            "StartWhenSynchronized",
            "TLSConfig",
        ],
    ),
    ("Prometheus", &["Addresses", "Enabled"]),
    (
        "Pprof",
        &["Addresses", "Enabled", "EnableBlock", "EnableMutex"],
    ),
    ("LevelDBOptions", &["DataDirectoryPath", "ReadOnly"]),
];
