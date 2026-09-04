# RPC observations and network checks

Managed nodes are checked against their configured public network, independently
of whether their HTTP endpoint answers. NeoNexus queries the same protocol for
all clients in a family:

| Clients | Identity | Peers | Synchronization |
| --- | --- | --- | --- |
| neo-cli, neo-go, neo-rs | `getversion.protocol.network` | `getconnectioncount` | Block-count history |
| geth-neox, neox-rs | `eth_chainId` | `net_peerCount` | `eth_syncing` and block-count history |

N3 public identifiers are 860833102 (MainNet) and 894710606 (TestNet).
Neo X identifiers are 47763 (MainNet) and 12227332 (TestNet T4).
An unknown or mismatched identifier prevents a Healthy verdict. Missing or
malformed peer counts are unknown, never a fabricated zero. Public nodes with
zero peers are Degraded because they cannot follow the public network.

Private nodes and bare CLI endpoints have no inferred expected identity. Their
reported identifier is shown as observed, without an expected identity to compare
against. Zero peers can be intentional for an isolated development node and does
not degrade it. A matching identifier is not proof of a common genesis, a fully
synced node, or an honest RPC server; genesis/fork verification is outside this
check.

The complete probe shares one timeout budget, including the extra identity and
peer calls. Each result stores its actual observation time, observed process PID,
network information and optional EVM synchronization flag. Existing databases
migrate older rows with unknown values. They are not retroactively verified.

The fleet and MCP status mark an observation stale after three configured probe
intervals, or when it belongs to a different PID. A stopped node or disabled
monitor does not display a historical Healthy verdict as current. Node details
retain the original historical result and timestamp. Prometheus exports peer
count, network identifier/check status, synchronization, observation age and
checked-at time; unknown optional values have no sample. Alert rules must check
observation age as well as the stored verdict.

Protocol sources checked during implementation:

- [N3 getversion](https://docs.neo.org/docs/n3/reference/rpc/getversion.html)
- [N3 getconnectioncount](https://docs.neo.org/docs/n3/reference/rpc/getconnectioncount.html)
- [Official N3 TestNet configuration](https://github.com/neo-project/neo-node/blob/master-n3/src/Neo.CLI/config.testnet.json)
- [Ethereum JSON-RPC](https://ethereum.org/developers/docs/apis/json-rpc/)
- [Geth net namespace](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-net)
- [Neo X public network identifiers](https://xdocs.ngd.network/development/development-environment-information)
