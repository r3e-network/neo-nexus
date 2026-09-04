# Configuration and version changes

The Config page provides a **Review config** action for every registered node.
It stages a candidate without changing the running configuration. Start and
managed restart run the same comparison before writing any primary or plugin
configuration file.

| Client | Managed configuration | Version selection | Plugin packages |
| --- | --- | --- | --- |
| neo-cli | JSON plus enabled plugin JSON files | Verified installed binary | Versioned ZIP packages for catalogued plugins |
| neo-go | YAML | Verified installed binary | Services are built in |
| neo-rs | TOML | Verified installed binary | Services are built in |
| neox-rs | TOML plus launch flags | Verified installed binary | No Neo N3 plugin packages |
| geth-neox (`neox-geth`) | TOML plus launch flags | Verified installed binary | No Neo N3 plugin packages |

## Resolve a configuration conflict

1. Open **Config** after Review, Start, a runtime selection or a plugin install
   reports a conflict. Each affected file shows its local path, candidate path
   and previous/proposed version.
2. Compare or edit these files locally. Secret values are not displayed in the
   web page or CLI drift samples.
3. Stop the node, then choose **Keep local** or **Back up local and use generated**.
   The latter writes a uniquely named backup beside the file before replacing it.
   A stale browser review cannot overwrite subsequent file changes.
4. Retry the original operation. All primary and plugin files are checked before
   the exporter begins publishing the new configuration.

Keeping local records the accepted file hash, generated candidate hash and target
version. It lasts while those inputs remain unchanged. A runtime version change
requires another review of local customizations even if the built-in renderer
produces identical text. A changed managed port, wallet injection or other
generated setting also requires review. Unmodified generated files update
automatically. Legacy files with no baseline require review when they differ.

Metadata lives in `.<filename>.neonexus-*` files beside the configuration. Keep
these files with the configuration when backing up or moving a workspace.
Removing metadata never grants permission to overwrite a different local file.
Candidate and backup files receive owner-only permissions on Unix; on Windows,
their access follows the workspace directory ACL.

## Select a node or plugin version

In **Runtimes → Node versions**, choose an installed version for a stopped node.
The operation checks its node family, host platform, installed SHA-256 and byte
count, enabled plugin compatibility, and configuration conflicts before changing
the node's recorded version or executable. Selecting a previous installed binary
is an explicit rollback. Automatic upgrade planning does not propose a downgrade
and orders stable releases above their prereleases.

In **Plugins**, a neo-cli node can install a local ZIP using its expected SHA-256,
explicit plugin version and the release's declared compatible neo-cli version.
These compatibility declarations are supplied by the operator; they are not
inferred by executing or inspecting the plugin assembly. The node runtime must
be pinned to an explicit version. Existing unversioned inventories are labelled
as such rather than assigned an invented release version.

Changing plugin packages or enabling/disabling neo-cli plugins requires a stopped node. Disabled packages are moved outside the Plugins loader tree to `.neonexus-disabled-plugins`, preserving their files and inventory; upgrading a disabled package leaves it disabled. Managed launches reconcile the recorded enabled state before writing config. A live restart whose plugin activation changed requires Stop first. Different packaged configuration
defaults create candidates and leave the existing plugin untouched until resolved.
Local configuration and key files survive successful replacements. Install an
older verified compatible ZIP to restore a previous plugin version. A versioned
plugin whose declared runtime compatibility excludes the selected node version
blocks managed launch and runtime selection.

## Scope of validation

The built-in generators describe the schemas supported by this codebase. The
release catalogue currently carries binary versions and digests, not an upstream
configuration schema/default bundle for every release. Therefore a version bump
can be detected and local edits protected, but compatibility with unknown future
upstream fields still needs the client's release notes and a runtime smoke test.
The runtime installer currently accepts an executable file; multi-file upstream
distributions must first be unpacked/provisioned with their dependencies. The
workbench does not infer archive layouts or a .NET distribution's dependencies.

Explicit external configuration arguments bypass managed generation; automatic
version selection directs these nodes to the node editor for a manual external
config review. The plugin catalogue includes SignClient for neo-cli. Its typed configuration
writes only the signer name and a loopback HTTP gRPC endpoint; custody key IDs
and caller tokens remain in the separately provisioned bridge process.

## Neo SignClient connection

In **Plugins**, install a compatible SignClient ZIP, enable it while the node is
stopped, and set **SignClient bridge** to the bridge's actual loopback port.
`Plugins/SignClient/SignClient.json` receives `PluginConfiguration.Name` and
`Endpoint`, matching the [official v3.10.1 configuration](https://github.com/neo-project/neo-node/blob/v3.10.1/plugins/SignClient/SignClient.json).
The separately provisioned NeoOS bridge binds one caller/key; the client sends
public keys obtained from the validator set rather than a custody key ID.

Connection configuration does not automatically start a signing duty. The
[upstream DBFT plugin](https://github.com/neo-project/neo-node/blob/v3.10.1/plugins/DBFTPlugin/DBFTPlugin.cs)
selects the remote signer via the explicit interactive command
`start consensus SignClient` (substitute the configured signer name).
Its `AutoStart` path still starts the wallet when the wallet opens. NeoNexus's
managed process currently provides no unattended interactive consensus-start
channel; configuring or enabling SignClient must not be interpreted as proof
that consensus is running. Check the node's actual chain status and logs.
