# NeoNexus 全系统功能与架构审计报告

- 审计时间：2026-09-12
- 审计范围：当前工作树全部 Rust 源码、Web/CLI 入口、测试、文档、CI、配置和发布相关文件
- 当前版本：`4.3.1`
- 审计方式：源码静态审计 + 实际构建/测试/质量门禁

## 1. 总体结论

当前系统已经具备较完整的 Neo 节点运维工作台能力：节点注册、生命周期控制、RPC/日志健康、watchdog、运行时目录、插件、Fast Sync、私网角色、钱包元数据、signer custody、备份恢复、告警、指标、Web 工作台和 headless CLI 均已存在，并且本轮核心验证全部通过。

系统当前适合进入“架构治理和发布准备”阶段，不适合继续无边界增加功能。主要工作重点应从功能扩张转为：

1. 固化 core application service 边界；
2. 拆分超大型监督与 signer 模块；
3. 统一 Web/CLI 的 command/query 入口；
4. 增加真正的 ready/readiness、结构化日志、请求 ID 和优雅关闭；
5. 收拢未提交变更、版本文档和发布流程。

### 当前评级

| 维度 | 评级 | 结论 |
|---|---|---|
| 功能完整性 | B+ | 运维功能覆盖广，核心链路齐全 |
| 核心架构 | B | 已有 core/domain/repository 分层，但 adapter 仍直接访问 Repository |
| 生命周期可靠性 | B | 有 supervisor/watchdog/readiness，需继续强化恢复与并发边界 |
| Web 安全 | B+ | session、Bearer、Origin、token file、安全头较完整 |
| 数据安全 | B | 有备份/恢复/secret redaction，需加强迁移、并发和损坏数据库策略 |
| 可观测性 | B- | 指标、日志、事件已存在，缺 request ID、结构化服务日志和 ready 探针 |
| 测试质量 | A- | 本轮所有主要门禁通过，但真实节点和长时间运行测试仍不足 |
| UI/UX | B | 页面分区清楚、SSR/无 JS fallback 有优势，信息密度和大型表单仍需优化 |
| 发布准备 | B- | 验证通过，但工作树变更过大、版本/文档和发布治理需收口 |

## 2. 功能全景审计

### 2.1 运行时与节点管理

已具备：

- neo-cli、neo-go、neo-rs、Neo X geth、Neo X rs 节点类型；
- 节点创建、编辑、删除、启动、停止、重启和 runtime rebind；
- 启动前 readiness、端口冲突、配置生成、运行时探测；
- 配置漂移审计（`--check-config-drift`）与无损原子调和（`--reconcile-node-config`），带自动时间戳备份；
- P2P 网络拓扑与孤立探测（`--peer-health`），支持 Healthy / Sparse / Isolated 分级；
- 交易内存池深度与拥堵分析（`--mempool-status`），支持 Normal / Elevated / Congested 分级；
- supervisor 进程托管、PID 复用识别、状态恢复；
- watchdog 重启策略、指数退避、jitter、维护窗口和升级计划；
- 日志增量采集、cursor、rotation/truncation 处理、同步进度观察。

风险与缺口：

- 真实 neo-cli/neo-go/neo-rs 多平台长时间运行测试不足；
- `supervision.rs`（约 1059 行）同时承载引擎、生命周期、探针、告警和恢复；
- `supervisor/model.rs`（约 1105 行）同时承载进程模型、adapter、metrics 和观察状态；
- 进程、日志、数据库和外部 RPC 的失败语义仍分布在多个模块，排障成本高。

### 2.2 Web 工作台

已具备：

- Home、Nodes、Health、Logs、Readiness、Events、Alerts、Federation、Private Network、Runtimes、Snapshots、Plugins、Configuration、Wallets、Signer、Metrics、Settings；
- SSR 页面、内嵌 CSS/JS、无 JavaScript 表单 fallback；
- session cookie、Bearer API token、权限粒度、Origin/Referer 防 CSRF；
- 统一安全响应头、CSP、healthz、公共状态和 Prometheus 指标；
- 节点控制、插件安装、快照处理、signer relay、API token 管理。

风险与缺口：

- Web handler 仍有大量 `state.repository` 直接访问；本轮只对 Fleet/Home/Nodes/API 建立了 `WorkspaceQueries` 边界；
- 页面与 handler 文件偏大：`html.rs` 764 行、signer overview 685 行、nodes 628 行、control 567 行；
- 大型 settings/signer 表单缺少更强的分组、渐进式披露和危险操作确认统一组件；
- API 读接口有权限中间件，但写操作仍主要是浏览器 form，未来需要统一 command API 或明确长期不提供 REST 写接口；
- 缺少 `/ready` 级别探针，`/healthz` 只能证明进程存活，不能证明数据库/监督引擎可用。

### 2.3 Signer 与密钥托管

已具备：

- local-wallet、local-signer、NeoOS service 三类 backend registry；
- backend_id + key_id 显式引用；
- service admin 与 least-privilege credential 分离；
- signer relay body、超时、并发限制；
- transaction/consensus/raw lane 能力约束；
- 钱包验证、secret redaction、无私钥导入页面。

风险与缺口：

- `signing/registry.rs` 约 893 行，profile resolve、route selection、credential 解析和能力判断仍较集中；
- `state.rs` 中 `Custody` 与 WebState 绑定较深，测试构造和生产配置路径并行；
- 需要增加 key rotation、credential reload、失败审计、服务不可用和重复请求的长期测试；
- 不应将“token hash 比对”误当作完整 secret 生命周期管理，部署侧仍需 secret file 权限和轮换流程。

### 2.4 数据、备份和恢复

已具备：

- SQLite Repository；
- 节点、插件、runtime、snapshot、wallet profile、事件、策略和 signer profile 持久化；
- Scoped API Tokens RBAC 持久化与 CLI 管理（`--create-api-token`, `--list-api-tokens`, `--revoke-api-token`）；
- workspace backup export/import，导入节点实行零信任运行隔离（Quarantine）；
- 恢复前校验、活动节点防覆盖、secret material 拒绝；
- workspace integrity、metrics 和支持包。

风险与缺口：

- 当前 schema 主要通过初始化/兼容逻辑维护，缺少清晰、可编号、可回滚的 migration 目录；
- Repository 方法数量较多，row mapping、policy persistence、event persistence 和 domain validation 仍分散；
- SQLite 并发写入、锁等待、损坏 DB、备份期间运行节点等场景需要专门的故障注入测试；
- `WorkspaceQueries` 当前同时包含 read 和 mutation 方法，命名上应拆为 `WorkspaceQueries` 与 `WorkspaceCommands`，避免只读 service 逐渐变成万能 Repository facade。

### 2.5 插件、快照和远程资源

已具备：

- HTTPS、hash、可选 Ed25519 detached signature；
- size limit、expanded size、路径穿越和 symlink 防护；
- 原子发布和 active node 保护；
- Fast Sync snapshot cache/apply/import；
- remote federation URL normalization 和 probe。

风险与缺口：

- 2 GiB 插件 body limit 对 Web 服务是高资源上限，必须配合磁盘配额、并发上传限制和超时；
- 远程下载、解压和导入应统一进入后台 job，并提供取消、进度、磁盘空间预检查；
- 外部 HTTPS 证书、重定向和代理环境的生产测试不足；
- catalog/snapshot/plugin 的信任模型虽有校验，但 UI 需要更明显地区分“已验证”“仅 hash”“未签名”。

## 3. 架构与依赖审计

### 3.1 当前有效架构

```text
main / manager
        ├── Web adapter (Axum, SSR, forms, JSON reads)
        └── CLI adapter (text/JSON commands)
                    ↓
             core application services
                    ↓
     repository / runtime / supervisor / signer domains
                    ↓
       SQLite / local process / external RPC / signer service
```

这个方向是正确的：Web 和 CLI 应共享决策逻辑，而不是各自实现启动、恢复和安全判断。

### 3.2 已完成的架构改善

本轮引入并使用 `core::WorkspaceQueries`：

- 统一节点、插件状态、快照、事件、RPC 健康等查询入口；
- Fleet、Home、Nodes、Fleet API 已切换到该服务；
- 文档补充 adapter/core/repository 边界；
- 未改变 HTTP 路由和数据库 schema。

### 3.3 仍需治理的问题

#### P1：WorkspaceQueries 继续膨胀

当前文件已经包含读写方法。短期可保留，下一轮应拆为：

```text
core/workspace_queries.rs   read models and read-side operations
core/workspace_commands.rs  node/policy/token/plugin mutations
core/lifecycle.rs           start/stop/restart orchestration
```

Web handler 只能依赖对应 service，不应依赖 Repository。

#### P1：Supervision God Module

建议拆分：

```text
supervision/
  engine.rs          worker lifecycle and tick scheduler
  lifecycle.rs       start/stop/restart/reconcile
  probes.rs          RPC/federation/runtime probes
  logs.rs            collector orchestration
  alerts.rs          event routing and delivery
  recovery.rs        startup recovery and PID reconciliation
```

每个模块通过小型 trait 或明确输入输出协作，减少 `LoopState` 对全部领域类型的直接依赖。

#### P1：Supervisor Model God Module

建议拆分：

```text
supervisor/
  process_model.rs
  process_registry.rs
  runtime_adapters.rs
  metrics_adapters.rs
  observations.rs
  termination.rs
```

生命周期状态机应集中定义，避免 Web、supervision、Repository 各自理解 Running/Starting/Error 的转换规则。

#### P2：Feature-first 目录治理

现有目录同时混合顶层 facade 文件和子目录，长期建议按 feature 聚合：

```text
features/
  nodes/
  runtime/
  plugins/
  snapshots/
  signing/
  private_network/
  observability/
platform/
  repository/
  process/
  http/
```

不建议立即搬迁全部源码；应在下一次大版本或模块边界稳定后迁移。

## 4. 数据流、状态和并发审计

### 已有优点

- 节点生命周期走 readiness → managed config → supervise → persist；
- Web 与后台 supervision 共享 supervisor handle；
- 日志 collector lease 防止重复采集；
- 进程状态使用 PID/状态条件转换，避免盲目覆盖；
- 长任务使用 Jobs，signer relay 使用 semaphore；
- 备份恢复对 active runtime 有保护。

### 需要加强

1. 为每个状态机补充状态转换表和非法转换测试；
2. 统一所有后台任务的 cancellation、join、超时和错误上报；
3. 为 Repository 写入增加明确的 busy/locked 重试策略；
4. 为节点启动、停止、重启建立 operation id，并将相关事件串联；
5. 将“数据库状态”“进程实际状态”“最近观测状态”分成显式字段，不靠 handler 组合推断；
6. 对 supervisor lock、workspace lock、collector lease 的获得顺序写成不变量，防止未来死锁。

## 5. Web 布局与交互设计审计

### 当前布局优点

- Overview/Fleet/Operations/Network/Assets/Security/Workspace 分组符合运维工作流；
- Home 适合作为 fleet posture 总览；
- Nodes、Operations、Metrics 各自有明确任务；
- SSR 首屏快、无 JS fallback 可靠；
- 移动端有 mobile nav 和响应式表格；
- 危险操作使用 POST，并有 Origin/Referer 保护。

### 设计问题

- 侧边栏导航项目较多，首次使用者需要更明显的“当前任务/异常数量/最近活动”引导；
- Settings、Signer、Private network 表单复杂，建议采用分区卡片 + 只显示相关字段；
- 节点详情页应把“当前事实”“可执行操作”“风险/阻塞”“事件时间线”分开，不要混在一张长页面；
- 运行时、插件、快照等资源页应统一显示 trust state、版本、hash、来源、最后验证时间；
- 轮询 `/api/fleet` 适合低频状态，但不适合长任务进度；建议对 Jobs 引入 SSE 或明确的短轮询 job endpoint；
- 所有危险按钮应统一使用确认组件、影响对象、不可逆提示和 operation result。

## 6. 安全审计

### 已通过/较强部分

- 非 loopback bind 强制 HTTPS public origin；
- token 不从命令行或普通环境变量接收；
- protected token file 权限检查；
- session HttpOnly/SameSite/Secure；
- login backoff；
- Origin/Referer 检查；
- Bearer token 按权限路由授权；
- signer relay、插件上传有大小/超时/并发边界；
- source purity/quality 门禁通过；
- `/public-metrics` 已关闭匿名访问。

### 发布前仍应做

- 给所有 API token 增加 last-used、创建者、轮换和撤销审计；
- 将 metrics token 文档统一为 `read_fleet` 最小权限 token；
- 对错误响应做信息分级，避免把外部路径、服务内部错误直接暴露给远端；
- 增加反向代理部署下的 Host/Forwarded header 测试；
- 增加跨租户/多 workspace 隔离测试（如果未来支持多 workspace）；
- 明确 signer service unavailable 时页面和 API 的错误语义；
- 做依赖审计、SBOM、签名发布和构建 provenance 验证。

## 7. 测试与工程质量审计

本轮实际结果：

- fmt：通过；
- check all targets：通过；
- clippy `-D warnings`：通过；
- lib tests：此前 680 passed，1 ignored；
- Web tests：此前 64 passed，本轮新增公共指标鉴权后仍通过；
- domain：146 passed；
- repository：41 passed；
- integration：1 passed；
- ci_policy：5 passed；
- source-quality：通过；
- source-purity：通过。

测试缺口：

1. 无真实三类节点二进制的跨平台矩阵；
2. 缺少长时间 watchdog/supervisor soak test；
3. 缺少磁盘满、SQLite locked、进程树泄漏、代理断开、证书轮换故障注入；
4. UI 主要验证 HTML/HTTP 合约，缺少浏览器级可访问性和响应式截图回归；
5. 缺少 release package 在干净机器的安装/升级/回滚测试。

## 8. 发布与变更安排

当前工作树存在大量未提交和未跟踪变更，发布风险高。建议拆成以下提交/阶段：

1. `security/auth`：session、API token、Origin、metrics 访问控制；
2. `workspace/query-service`：WorkspaceQueries 与 Web read adapters；
3. `node-lifecycle/supervision`：supervisor、watchdog、日志 collector；
4. `runtime-assets`：runtime、plugin、snapshot；
5. `signing`：registry、relay、console control；
6. `web-layout`：页面、导航、表单、可访问性；
7. `validation/docs`：CI、source gates、文档、OpenAPI；
8. `release`：版本、CHANGELOG、package、signature、provenance。

每个阶段必须满足：

```text
实现 → 单模块测试 → 全量 check/clippy → 集成测试 → 文档同步 → 独立提交
```

## 9. 优先级路线图

### P0：发布前必须完成

- 保持当前全部门禁绿；
- 完成干净 release build 和 package integrity 验证；
- 清理未提交/未跟踪文件并按主题拆分提交；
- 核对 CHANGELOG、版本号、安装说明和部署示例；
- 补 `/ready` 探针或在部署文档中明确 healthz 的边界。

### P1：下一轮架构重构

- `WorkspaceQueries` / `WorkspaceCommands` 分离；
- 拆分 supervision.rs；
- 拆分 supervisor/model.rs；
- 将 Web 剩余 Repository 访问迁移到 core services；
- 统一后台 Job、operation id、事件关联和错误层级；
- 增加真实节点和长时间运行测试。

### P2：产品与工程提升

- 页面状态卡片和 trust state 统一设计；
- SSE/job progress；
- 浏览器级可访问性和视觉回归；
- migration 框架和数据库升级/回滚；
- SBOM、签名发布、provenance、灾备演练。

## 10. 最终判定

**核心功能和验证链：通过。**

**架构状态：可维护但需要治理，主要问题是大型模块和 Web/Repository 边界未完全收紧。**

**产品状态：功能丰富、运维导向明确，但高级功能页的信息密度和长任务反馈仍需优化。**

**发布状态：可以进入 release preparation，但在完成变更拆分、release package 验证、ready 探针和发布安全复核前，不建议直接对公网大规模部署。**
