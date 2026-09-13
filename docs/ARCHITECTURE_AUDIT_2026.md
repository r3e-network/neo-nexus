# NeoNexus 当前状态审计报告

- 审计时间：2026-09-11
- 审计范围：当前工作树 `D:\Git\neo-nexus`
- 当前版本：`4.3.1`
- 审计方式：只读检查；未修改业务代码
- 结论：**不建议按当前状态合并、发布或部署到公网**

## 一、总览结论

当前工作树已经从 4.0 的基础 Web 工作台扩展到 4.3.1，加入了 API token、signer relay、插件、快照、watchdog、metrics adapter、Node Manager 等大量能力。但当前变更处于“功能继续堆叠、验证尚未收口”的状态：

- Web 集成测试无法编译。
- 全目标 `cargo check` 无法通过。
- `cargo clippy --all-targets -- -D warnings` 无法通过。
- `cargo test --lib` 有 1 个失败。
- `--source-quality src` 失败。
- `--source-purity .` 失败。
- 工作树有大量未提交修改和未跟踪文件，无法作为干净 release 基线。
- 文档声称的 public-metrics 限流和 rate-limit headers，在当前路由实现中未发现对应通用限流层。

因此当前状态更接近“开发集成分支”，而不是 release candidate。

## 二、验证结果

| 检查 | 结果 | 证据 |
|---|---:|---|
| `cargo fmt --all -- --check` | 失败 | `/tmp/neo_fmt.log`；至少 `tests/web.rs` 存在 rustfmt 差异 |
| `cargo check --all-targets` | 失败 | Web 测试 `String` 传给 `&str`；repository 测试引用私有模块；缺失 `Repository::load_setting` |
| `cargo clippy --all-targets -- -D warnings` | 失败 | `src/logs/cursor.rs` 两个可 derive 的 Default；`src/logs/observations.rs` dead code |
| `cargo test --lib` | 失败 | 678 passed, 1 failed；`supervision::tests::engine_log_collection_bounds_reads_and_tolerates_missing_logs` 状态污染 |
| `cargo test --test integration` | 通过 | 1 passed |
| `cargo test --test web` | 无法编译 | `tests/web.rs:933-935` 的 `String` 未借用为 `&str` |
| `cargo test --test ci_policy` | 通过 | 5 passed |
| `cargo run -- --source-quality src` | 失败 | 新增 metrics adapter 测试、`private_network/magic_override.rs`、`wallet/api_token.rs`、`watchdog/scheduler.rs`、`web/pages/api_tokens.rs` 含门禁禁止的 unwrap/expect/panic |
| `cargo run -- --source-purity .` | 失败 | `docs/swagger-ui.html`、`docs/validate-openapi.js` 被识别为 frontend-source-file |

### 2.1 编译阻塞项

1. `tests/web.rs:933`
   - `post_form_as(..., body: &str)` 收到 `String`。
   - 修复为 `&("...".to_string() + "...")`，或先绑定 `let body = ...;` 再传 `&body`。

2. `tests/repository/basics_settings/watchdog_rpc.rs`
   - 直接访问 `neo_nexus::backup::{restore, schema}`，但两个模块是 private。
   - 同文件调用 `Repository::load_setting`，当前 Repository 没有该方法。
   - 应统一选择：要么通过公开 facade 导出稳定 API，要么把测试改成当前公开 API；不应通过放宽模块可见性临时绕过。

### 2.2 Clippy 阻塞项

1. `src/logs/cursor.rs:127`
2. `src/logs/cursor.rs:139`
   - 手写 Default 可由 derive 替代。
3. `src/logs/observations.rs:88`
   - `observed` 当前没有生产调用方。
   - 如果是后续功能，删除或增加明确调用；不要用 `allow(dead_code)` 掩盖。

### 2.3 单元测试阻塞项

`supervision::tests::engine_log_collection_bounds_reads_and_tolerates_missing_logs` 报：

> The Engine already owns this workspace log collector

这是典型的进程内全局/注册表状态污染：前一个测试留下 workspace collector，后一个测试无法重新拥有同一 workspace。应在测试 fixture 中使用唯一 workspace 标识，或提供明确的 teardown/reset；不应依靠测试执行顺序。

## 三、安全审计

### 3.1 已做得较好的部分

- 非 loopback bind 要求显式 public origin，并要求 HTTPS。
- 浏览器状态变更检查 Origin/Referer；Origin 存在时不会被 Referer 绕过。
- session cookie 设置 HttpOnly、SameSite=Strict；HTTPS 部署增加 Secure。
- operator token 不允许通过命令行传入，改用受保护 token file / 环境文件路径。
- token file 有 regular-file、大小和 Unix 权限检查。
- 登录失败有按 peer 的 backoff 和有限容量表。
- API Bearer token 经过数据库 hash 验证，再由 route-level permission 授权。
- signer relay 有 body size、超时、并发 semaphore 边界。
- 插件包配置了 2 GiB body/expanded size 上限。
- 浏览器 HTML 大量使用统一 escape helper，未发现明显的直接未转义用户数据模板注入点。

### 3.2 P1 风险：public-metrics 的安全契约不完整

当前 `src/web/router.rs` 将 `/public-metrics` 明确作为完全公开路由，`require_session` 中也直接放行。`src/web/auth.rs` 定义了 `METRICS_TOKEN_ENV`，但在当前路由检查中没有看到该 token 被用于鉴权。

同时 `docs/AGENT_API.md` 声称：

- `/public-metrics` 为 `10/min/IP`；
- 所有响应包含 `X-RateLimit-*` headers；
- 超限返回 429。

当前 `src/web/router.rs` 未发现通用 rate-limit layer，`public_api`/metrics 路径也没有看到针对该 endpoint 的计数器。文档与实现不一致，且公网开放 Prometheus 指标可能泄露节点状态、端口、进程信息和运行时拓扑。

建议：

1. 默认关闭 public-metrics，或默认要求 `NEONEXUS_METRICS_TOKEN_FILE`。
2. 如果确实需要公开 scrape，增加真实的 per-IP 速率限制、响应 headers 和 429 测试。
3. 将文档改成实现真实支持的契约，不能只写设计目标。
4. 对 metrics 做最小披露：避免输出 binary path、敏感配置、内部 endpoint 或高基数用户输入。

### 3.3 P1 风险：API token 生命周期与错误处理需补强

`src/repository/api_tokens.rs` 中存在多处 `parse().unwrap_or_default()`、`try_into().unwrap_or([0; 32])`。数据库损坏或 schema 漂移时，这会把非法 token id/hash 静默变成默认值，掩盖数据完整性错误。建议改为带上下文的错误返回，并为损坏记录增加失败测试。

`src/web/pages/api_tokens.rs:21` 还直接使用 `unwrap()`，已经被 source-quality 门禁抓到，应修为错误渲染或可恢复响应。

## 四、架构与可维护性

### 4.1 高风险：工作树变更过大且边界不清

当前 `git status` 显示：

- 39 个已修改文件，约 `3700 insertions / 171 deletions`；
- 大量未跟踪的 `src/node_manager/`、benchmarks、OpenAPI 文档、插件/metrics adapter 和测试；
- `Cargo.lock` 单次增加约 956 行；
- 根目录存在多份 build/clippy/test 日志和临时报告文件。

建议按主题拆分提交：

1. API token/auth/security；
2. Web plugin/snapshot/wallet/settings；
3. watchdog/supervision；
4. Node Manager adapters；
5. metrics adapters；
6. docs/OpenAPI/benchmarks；
7. 测试与门禁收口。

每组先实现、再验证、再提交，避免当前这种全局失败难以定位的集成状态。

### 4.2 `src/supervisor/model.rs` 变更过大

单文件本次增加约 798 行，`src/supervisor/process.rs` 增加约 347 行。虽然此前已经移除了 Rust 200 行限制，但取消限制不等于取消模块边界。建议按职责拆为：

- process identity / lifecycle；
- adapters；
- metrics collection；
- log observations；
- restart/watchdog state；
- serialization/reporting。

拆分目标是降低锁、状态机和生命周期交叉，而不是满足行数指标。

## 五、source 门禁问题

### 5.1 source-quality

当前 source-quality 会扫描新加入的 metrics adapter 测试和生产模块，发现大量 unwrap/expect/panic。应先区分：

- 测试代码：若项目政策允许，应让测试扫描器忽略 `tests/` 中明确的断言辅助；
- 生产代码：必须修成 Result/Option 分支、锁中毒恢复或明确错误；
- 文档示例：不要把 `expect` 放进会被生产扫描的 Rust doc snippet，或调整规则实现注释代码识别。

不建议为快速通过而整体放宽 source-quality；这样会削弱该门禁的价值。

### 5.2 source-purity

当前 source-purity 失败文件：

- `docs/swagger-ui.html`
- `docs/validate-openapi.js`

如果项目继续坚持“纯 Rust、无前端文件”边界，应删除这些文件，改为：

- OpenAPI YAML 作为文档数据保留；
- 使用外部 Swagger UI 仅作为开发工具，不纳入仓库；
- 或明确修改 purity 规则，把 OpenAPI 文档工具定义为受控文档资产，并补充新的政策测试。

当前状态属于规则和仓库资产不一致。

## 六、文档与实现漂移

已发现以下漂移或需要统一：

- `CHANGELOG.md` 仍以 4.0.0 为主要 Web 转型条目，而 Cargo 当前为 4.3.1；应补齐 4.1/4.2/4.3 变更，或明确 release notes 来源。
- `docs/AGENT_API.md` 描述了未在路由中证实的 rate limiting 和 headers。
- `.env.example` 有 retry 配置，但 Web token、public origin、metrics token 等关键 Web 部署配置没有同等清晰的示例说明。
- 文档同时保留历史 native 命名和当前 Web/Node Manager 能力，建议增加“当前架构”和“历史迁移”边界，避免运维按旧文档部署。

## 七、优先修复顺序

### P0：合并前必须修复

1. 修复 `tests/web.rs` 编译错误。
2. 修复 repository watchdog 测试对 private module / 缺失 `load_setting` 的错误依赖。
3. 修复 Clippy 两个 Default 和一个 dead_code。
4. 修复 supervision 测试全局状态污染。
5. 让 `cargo fmt --check`、`cargo check --all-targets`、`cargo clippy -D warnings`、`cargo test --lib`、`cargo test --test web` 全部通过。

### P1：发布前必须修复

1. 让 source-quality 对新代码全绿，不通过全局放宽规则掩盖问题。
2. 删除或重新定义 `swagger-ui.html` / `validate-openapi.js` 的 purity 策略。
3. 实现 public-metrics 的真实 token/rate-limit 契约，或关闭匿名公开指标。
4. 修复 API token 数据库解析的静默默认值。
5. 核对所有 API token 和 signer 管理路由的权限、CSRF、审计事件。

### P2：后续优化

1. 拆分 `supervisor/model.rs` 与 `supervisor/process.rs`。
2. 拆分大型 Web 页面和 control 模块，降低 handler 与 repository 的耦合。
3. 清理根目录临时日志和未跟踪报告，建立 artifacts 目录或 CI artifact 流程。
4. 按功能拆分提交，建立 v4.3.x release checklist。
5. 增加真实 neo-cli/neo-go/neo-rs 长时间 supervision、watchdog、升级和 signer relay 集成测试。

仍需关注的发布前风险：public-metrics 的匿名暴露、限流和 metrics token 契约，以及当前工作树大量未提交变更。完成这两项后再进行 release build/package 和公网部署复核。

## 八、最终判定

**当前状态：核心验证已绿，发布前安全与变更治理仍待收口。**

功能和测试门禁已经恢复通过，但不应把“测试全绿”直接等同于“公网安全已完成”；public-metrics 契约和未提交变更边界仍需在 release 前明确。

## 九、修复复验结果

本轮修复后已重新运行完整验证，结果全部通过：

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --lib`：680 passed，1 ignored
- `cargo test --test web`：63 passed
- `cargo test --test domain`：146 passed
- `cargo test --test repository`：41 passed
- `cargo test --test integration`：1 passed
- `cargo test --test ci_policy`：5 passed
- `cargo run -- --source-quality src`
- `cargo run -- --source-purity .`

- `/public-metrics` 已收口为 session 或 `read_fleet` Bearer token，匿名访问测试已移除并替换为未授权拒绝/最小权限 token 测试。
- `docs/AGENT_API.md`、`.env.example` 已同步真实契约；未实现的历史 per-IP rate limit、429 和 `X-RateLimit-*` 声明已删除。
- 本轮验证：Web 测试 64 passed，CI policy 5 passed，fmt/check/clippy/source-quality/source-purity 全部通过。

