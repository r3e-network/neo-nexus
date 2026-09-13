# 架构审计与优化报告

## 本轮完成

### 1. 架构基线

当前项目是 Rust 单体服务，包含：

- `core/`：领域服务和共享查询 facade；
- `web/`：Axum 浏览器适配层；
- `cli/`：无状态命令适配层；
- `repository/`：SQLite 持久化；
- `supervision.rs` / `supervisor/`：节点生命周期和后台调度；
- `runtime/`、`snapshots/`、`plugins/`、`private_network/`、`signing/`：运行时和运维领域能力。

审计发现的主要结构性问题：

1. Web 层有大量 handler 直接访问 `state.repository`，展示层与 SQLite API 耦合。
2. `supervision.rs` 和 `supervisor/model.rs` 体积过大，分别承载多个独立职责。
3. Web、CLI、后台 supervision 共享领域逻辑的方式不够显式，查询和变更边界不一致。
4. 文档此前没有清晰说明 Web/CLI 是 adapter、core 是应用服务边界。

### 2. 已实施重构

新增 `src/core/workspace_queries.rs` 中的 `WorkspaceQueries`：

- 封装节点、插件状态、快照、事件、RPC 健康等只读工作区查询；
- 由 `WebState` 持有，和 Repository 生命周期保持一致；
- `Fleet::load` 改为接收 `WorkspaceQueries`；
- Web Home、Nodes、Fleet API 改用查询服务；
- Repository 不再是这些页面组装逻辑的直接依赖；
- 保留 Repository 作为底层持久化实现，避免一次性大规模行为变更。

这建立了明确的数据流：

```text
HTTP / CLI adapter
        ↓
core application services
        ↓
repository / supervisor / runtime domain
        ↓
SQLite / node process / external RPC
```

### 3. 文档整理

更新 `docs/native-rust.md`：

- 明确 Web 和 CLI 是 adapter；
- 明确 core 是应用服务和查询 facade；
- 增加 `WorkspaceQueries` 的职责说明；
- 补充 lifecycle、operations、runtime、security 的模块边界；
- 明确 handler 不应嵌入 SQLite 查询或重复生命周期决策。

## 验证结果

全部通过：

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test web`：64 passed
- `cargo test --test domain`：146 passed
- `cargo test --test repository`：41 passed
- `cargo test --test ci_policy`：5 passed
- `cargo run -- --source-quality src`
- `cargo run -- --source-purity .`

## 下一轮建议

### P1：继续拆分后台监督职责

将 `supervision.rs` 拆成：

- `supervision/engine.rs`：线程生命周期和 tick 调度；
- `supervision/lifecycle.rs`：节点 start/stop/restart；
- `supervision/probes.rs`：RPC/federation/runtime probes；
- `supervision/alerts.rs`：事件路由和告警投递；
- `supervision/reconciliation.rs`：启动恢复和进程状态对账。

### P1：继续拆分 ProcessSupervisor 模型

将 `supervisor/model.rs` 拆成：

- process identity/spec；
- runtime adapters；
- metrics adapters；
- log observations；
- status serialization。

### P2：扩展 command/query service

将 Web 中剩余的变更 handler 逐步迁移到：

- `core::commands`：节点、策略、插件、快照、备份命令；
- `core::queries`：节点、事件、指标、运行时和 signer 查询。

## 风险说明

本轮重构保持了外部行为，不改变数据库 schema 和 HTTP 路由。架构改善已验证通过，但大量旧功能仍集中在大型模块中，后续拆分应继续采用“小步重构、每轮全量验证”的方式，避免一次性重写造成行为回归。
