# 当前项目审计概览

- 完成全系统功能、架构、数据流、Web 布局、安全、测试、发布安排审计，报告见 `docs/SYSTEM_AUDIT_2026.md`。
- 识别出当前最高收益的后续工作：拆分 `WorkspaceQueries/Commands`、拆分 `supervision.rs` 与 `supervisor/model.rs`、增加 `/ready` 和长任务进度能力。
- 完成当前 v4.3.1 工作树审计，并修复审计发现的 P0/P1 阻塞。
- 修复 Web 测试 `String/&str` 编译错误。
- 正确导出备份恢复 API，并补齐 `Repository::load_setting`。
- 修复 watchdog jitter 表单非法值处理和保存后的 UI 回显。
- 修复日志 collector lease 生命周期，避免 supervision 状态污染。
- 修复 Clippy 的 Default、dead code、冗余闭包等问题。
- 修复日志解析、metrics adapter、wallet token、watchdog、magic override 中的 source-quality 阻塞。
- 删除被 pure-Rust 门禁禁止的 Swagger HTML/JS 资产。
- `/public-metrics` 已改为需要 session 或 `read_fleet` Bearer API token，匿名访问已关闭。
- `docs/AGENT_API.md` 和 `.env.example` 已同步真实认证契约；删除了未实现的旧限流/header 承诺。

## 架构治理（已完成）

治理报告见 `docs/ARCHITECTURE_GOVERNANCE_2026.md`。以下五项已按「行为保持 + 小步增量 + 每步过门禁」完成：

- 拆分 `src/supervision.rs`（1059 行）为 `src/supervision/` 的 10 个职责文件，最大文件降至 288 行。
- 拆分 `src/supervisor/model.rs`（1105 行）为 `src/supervisor/model/` 的 5 个职责文件，最大文件降至 558 行。
- 新增 `WorkspaceCommands`，实现读写分离：`WebState` 暴露 `workspace`（读，25 方法）与 `commands`（写，25 方法）。
- Web 层 `Repository` 直连由 82+ 处收敛到 2 处（均为签名要求 `&Repository` 的自由函数逃生口）。
- 接入 7 个从未进入 `mod` 树的孤儿源文件（约 2338 行），并修复其中 4 个真实实现缺陷；孤儿文件数降为 0。

## 最终验证结果

全部通过：

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --lib` — 705 passed，1 ignored
- `cargo test --test web` — 64 passed
- `cargo test --test domain` — 146 passed
- `cargo test --test repository` — 41 passed
- `cargo test --test integration` — 1 passed
- `cargo test --test ci_policy` — 5 passed
- `cargo run -- --source-quality src` — 861 文件，0 findings
- `cargo run -- --source-purity .` — pure-rust，0 disallowed

## 仍需关注

- `/public-metrics` 的认证契约已收敛并固化：代码用 `ReadFleet` 强制关闭匿名（`src/web/router.rs`），`docs/AGENT_API.md` 已同步，`tests/web.rs` 有断言（匿名 401、`read_fleet` token 200）。遗留待办其余几项见下。
- 工作树现有 117 项变更，发布前应按主题拆分为独立提交并进行 release build/package 验证。
- Web 层仍有 4 个 600+ 行文件（`web/html.rs`、`pages/signer/overview.rs`、`signer_control.rs`、`pages/nodes.rs`）待拆。
- 建议把孤儿源文件检测固化进 `ci_policy` 测试，避免死代码再次累积。
- 完整风险分级与证据见 `docs/ARCHITECTURE_AUDIT_2026.md`。
