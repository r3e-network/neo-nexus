# NeoNexus 架构治理报告

日期：2026-08-28
基线：`docs/ARCHITECTURE_AUDIT_2026.md`、`docs/SYSTEM_AUDIT_2026.md`
版本：v4.3.1
治理原则：**行为保持（behavior-preserving）+ 小步增量 + 每一步都过门禁**。

---

## 一、治理目标

审计结论指出四类边界问题：

1. `src/supervision.rs`（约 1059 行）把引擎循环、启动对账、重启调度、健康探测、升级探测、告警路由、外部进程看管混在一个文件里；
2. `src/supervisor/model.rs`（约 1105 行）把进程模型、适配器、metrics、日志观测状态混在一个文件里；
3. `WorkspaceQueries` 同时承担读与写，Web handler 无法从类型上区分「查询」与「变更」；
4. Web 层大量 handler 直接访问 `Repository`，绕过工作区服务层。

此外还发现 **7 个从未进入任何 `mod` 树的孤儿源文件**（共约 2338 行），属于「存在但从未参与编译」的死代码。

本次治理逐项处理以上五项，并在每一步之后跑完整门禁。

---

## 二、治理动作

### 2.1 拆分 `src/supervision.rs` → `src/supervision/`

`src/supervision.rs` 已删除，替换为目录模块。`mod.rs` 仅做门面重导出：

```rust
pub use engine::Engine;
pub use launch::{launch_node, stop_node};
pub use state::EngineState;
```

| 文件 | 行数 | 职责 |
| --- | --- | --- |
| `state.rs` | 111 | `EngineState` / `LoopState`，持有 nodes、journal、supervisor、bootstrap、`tick()` |
| `engine.rs` | 96 | `Engine` 结构体、`start()`、`Drop` |
| `launch.rs` | 147 | `launch_node()` / `stop_node()` |
| `startup.rs` | 72 | `reconcile_startup()` |
| `restarts.rs` | 211 | `sync_policy()`、`reconcile_exits()`、`schedule_restart()`、`run_due_restarts()` |
| `probes.rs` | 132 | `due()`、`probe_rpc_health()`、`probe_federation()` |
| `upgrade.rs` | 288 | `probe_runtime_upgrade()` |
| `alerts.rs` | 53 | `route_alerts()` |
| `external.rs` | 63 | `watch_external_processes()` |
| `mod.rs` | 36 | 门面 |

最大文件由 1059 行降至 288 行，且每个文件只对应一个监督循环生命周期阶段。

配套调整：
- `Engine` 的 `stop` / `worker` / `log_collection_stop` / `log_collection_handle` 改为 `pub(super)`，供测试访问；
- `LoopState` 改为 `pub(super)`；
- `tests/unit/supervision/tests.rs`、`tests/unit/supervision/upgrade/tests.rs` 的导入路径改为 `super::{launch::launch_node, state::{EngineState, LoopState}, Engine}`，并补齐 `Repository`、`ProcessSupervisor`、`SignerRegistry`、`BTreeMap` 等显式导入。

### 2.2 拆分 `src/supervisor/model.rs` → `src/supervisor/model/`

| 文件 | 行数 | 职责 |
| --- | --- | --- |
| `log_parsers.rs` | 558 | `LogEntry` / `FatalError` / `SyncProgress` / `LogParserAdapter` + 5 个节点类型适配器 |
| `process.rs` | 202 | `PluginMetadata`、插件与生命周期 trait、`ProcessStart/Stop/Exit`、`ManagedProcessKind/Spec` |
| `adapters.rs` | 175 | `NodeAdapters` 注册表，把实现绑定到 `NodeType` |
| `metrics.rs` | 160 | `MetricsExporterAdapter` + 5 个 metrics 适配器 + `NoOpMetricsExporterAdapter` |
| `model.rs` | 22 | 门面，含 `DEFAULT_STOP_GRACE_PERIOD` |

最大文件由 1105 行降至 558 行。门面只保留真正被外部使用的导出项，避免「重导出孤儿文件才用到的符号」这类假性依赖。

### 2.3 `WorkspaceQueries` 读写分离

新增 `src/core/workspace_commands.rs`，承载全部 25 个变更方法；`workspace_queries.rs` 保留 25 个只读方法。

- `WebState` 新增 `pub commands: WorkspaceCommands`，与 `workspace` 并列构造；
- 11 个 Web 文件中所有 `state.workspace.<mutation>(` 改写为 `state.commands.<mutation>(`；
- 原本混在查询侧的 `verify_token_secret`、`list_api_tokens`、`load_watchdog_policy` 等读方法归位到 `WorkspaceQueries`。

边界规则：**读走 `state.workspace`，写走 `state.commands`。**

### 2.4 收敛 Web 层对 `Repository` 的直连

为消除 escape hatch，向两个服务共补充 22 个带类型的方法，并用脚本批量改写 12 个 Web 文件的调用点。

结果：Web 层 `Repository` 直连由 **82+ 处降至 2 处**，且这 2 处是签名要求 `&Repository` 的自由函数，属于有意保留的 `pub(crate)` 逃生口：

- `src/web/control.rs:425` — 备份导出器；
- `src/web/pages/nodes.rs:243` — `node_rpc_health_history()`。

### 2.5 孤儿源文件接入编译

扫描发现 7 个从未被任何 `mod` 声明引用的 `.rs` 文件：

| 文件 | 行数 | 处理 |
| --- | --- | --- |
| `src/metrics/prometheus/neo_cli_adapter.rs` | 325 | 接入 `pub mod prometheus` |
| `src/metrics/prometheus/neo_go_adapter.rs` | 347 | 同上 |
| `src/metrics/prometheus/neo_rs_adapter.rs` | 403 | 同上 |
| `src/metrics/prometheus/neox_geth_adapter.rs` | 420 | 同上 |
| `src/metrics/prometheus/neox_reth_adapter.rs` | 358 | 同上 |
| `src/utils/backoff_tests.rs` | 430 | 以 `#[cfg(test)] mod backoff_tests;` 接入 |
| `src/node_lifecycle/context.rs` | 55 | 以 `mod context;` 接入并导出 `generation_context_for_node` |

接入前这些文件是针对旧 API 写的（`GenerationContext.node`、`NodeConfig.working_dir`、`node.rpc_port: Option`、`serde_yaml::json!`、缺失 `metrics_url()`），产生了 33 个编译错误。修复属于机械性 API 漂移对齐：

- `node.rpc_port.unwrap_or(x)` → `node.rpc_port`；
- `serde_yaml::json!` → `serde_yaml::to_string(&config)?.into_bytes()`；
- 为每个 `MetricsExporterAdapter` 实现补 `metrics_url()`；
- `ctx.node.working_dir` → 新增 `node_dir_from_context(ctx)`（为此在 `GenerationContext` 上增加 `node_dir: Option<PathBuf>` 与 `with_node_dir()`）；
- `AdapterMode` 提升为 `pub`；`cfg!(feature="tokio-console")` 收敛为常量 `TOKIO_CONSOLE_BRIDGE`；
- `new()` 签名改为接受 `rpc_port: u16`；`file_stem()` → `path().file_stem()`；修正枚举上的 `#[derive]`。

**接入后暴露并修复了 4 个真实实现缺陷**（这些代码此前从未执行过，所以缺陷一直不可见）：

1. `neox_geth` `parse_geth_sync` 偏移错误 —— `"number="` 是 7 个字符，代码按 9 计算，改为 `num_start + 7`；
2. `neo_cli` `parse_block_number` 未处理裸 `#12345` 形式，补齐该分支；
3. `neo_go` `parse_line` 在第一个空格处截断消息，导致带引号的 `"block processed"` 解析失败，新增 `split_outside_quotes()` 辅助函数；
4. `neo_rs` `bridge_to_prometheus` 只解析第一个 `key=value`，改为 `tokio_metrics` 返回全部键值对。

另有 2 处测试期望与实现不一致（backoff jitter 断言写死精确值、`reboot` 文案断言），改为区间断言并移除过时断言。

接入后孤儿文件数由 7 降为 **0**。

---

## 三、验证结果

全部通过：

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | ok |
| `cargo check --all-targets` | ok |
| `cargo clippy --all-targets -- -D warnings` | ok |
| `cargo test --lib` | 705 passed / 0 failed / 1 ignored |
| `cargo test --test web` | 64 passed |
| `cargo test --test domain` | 146 passed |
| `cargo test --test repository` | 41 passed |
| `cargo test --test integration` | 1 passed |
| `cargo test --test ci_policy` | 5 passed |
| `cargo run -- --source-quality src` | ok（861 文件，0 findings，0 maintenance-files） |
| `cargo run -- --source-purity .` | pure-rust（18907 文件，0 disallowed） |

单元测试由治理前的 680 增至 **705**（孤儿文件接入带来的 25 个用例），web 由 63 增至 **64**，其余套件无回归。

---

## 四、治理后模块规模

`src/` 共 861 个 `.rs` 文件，无文件超过 1000 行维护阈值。当前最大的 10 个文件：

| 文件 | 行数 |
| --- | --- |
| `src/signing/registry.rs` | 893 |
| `src/web/html.rs` | 764 |
| `src/utils/backoff.rs` | 759 |
| `src/web/pages/signer/overview.rs` | 685 |
| `src/web/signer_control.rs` | 631 |
| `src/web/pages/nodes.rs` | 626 |
| `src/web/control.rs` | 573 |
| `src/core/node_signer.rs` | 559 |
| `src/supervisor/model/log_parsers.rs` | 558 |
| `src/secret_file.rs` | 470 |

本次治理针对的 `supervision.rs` 与 `supervisor/model.rs` 已移出大文件榜。

---

## 五、遗留项与下一步建议

1. **Web 大文件仍未拆分**。`web/html.rs`(764)、`web/pages/signer/overview.rs`(685)、`web/signer_control.rs`(631)、`web/pages/nodes.rs`(626) 仍是体积热点，建议按「渲染 / 取数 / 表单处理」再拆一轮。
2. **`signing/registry.rs`(893) 与 `utils/backoff.rs`(759)** 建议评估：前者按密钥类型或后端拆分，后者把策略计算与退避执行分离。
3. **2 处 `Repository` 逃生口**应最终消除：把备份导出器与 `node_rpc_health_history()` 改为接受工作区服务，或正式登记为受控例外并在门禁中白名单化。
4. **孤儿文件根因未除**。本次是事后扫描才发现 2338 行死代码；建议在 `ci_policy` 测试中固化孤儿检测，避免再次累积。
5. **变更收口**：工作树现有 117 项变更，发布前应按主题（supervision 拆分 / supervisor model 拆分 / workspace 读写分离 / web 收敛 / 孤儿接入 / docs）拆分为独立提交。
6. 审计中标记的 P1 安全项（public-metrics 契约、API token 生命周期）不在本次架构治理范围内，仍需单独收口。
