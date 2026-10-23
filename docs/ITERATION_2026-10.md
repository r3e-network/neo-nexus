# 迭代记录：2026-10-20

基线：拉取 `origin/main` = `63e4731`（fix(ci): pin workflow shells for cross-platform checks）
方式：先建立可复现基线，再逐项定位根因、修复、复验。

---


---

## 目录

本文按轮次拆成三份，每份都能单独读；节号在拆分后保持不变。

| 节 | 内容 | 位置 |
| --- | --- | --- |
| 一至五 | 拉取与基线、根因与修复、最终验证、审计结论、后续建议 | 本文件 |
| 六至九 | CI 契约门禁、全量审计、第二轮、最终门禁 | [ITERATION_2026-10-audit.md](ITERATION_2026-10-audit.md) |
| 十至十三 | #50 与 #48：会丢输入的表单改为失败重渲染 | [ITERATION_2026-10-forms.md](ITERATION_2026-10-forms.md) |

## 一、拉取与基线

本地 `main` 原在 `82faf93`，与远端新历史**已分叉**（远端 465 个提交，本地 431 个独有提交）。
远端被 force-push 重写；新历史已包含本地全部产物（含 `docs/ARCHITECTURE_GOVERNANCE_2026.md`）。

处理：

1. 建备份分支 `backup/local-main-82faf93`（指向原 HEAD），不丢失任何本地提交；
2. `git reset --hard 63e4731`；
3. 校验 HEAD、工作树干净、`origin/main` 一致。

基线门禁（修复前）：

| 门禁 | 结果 |
| --- | --- |
| fmt / check | 通过 |
| clippy `-D warnings` | **失败**：`tests/unit/backup/documented/tests.rs` 字段 `owner_mode` 在 Windows 上未读取 |
| lib | 1000 passed |
| ci_policy | **失败**：workflow 步骤名匹配不上 |
| domain / repository | 通过 |
| web | **失败**：4 个用例 |
| integration / ui_* | 通过 |

---

## 二、根因与修复

### 1. `ci_policy` 在 Windows 上必然失败

`tests/ci_policy/workflow_shells.rs` 用 `"      - name: {name}\n"` 精确匹配 YAML。
本机 `core.autocrlf=true` 且仓库无 `.gitattributes`，工作树中的 workflow 是 CRLF，
`find` 必然返回 `None` → `workflow step is missing`。

修复：读取后统一 `\r\n` → `\n`（`read_workflow`）。门禁行为在所有平台一致。

### 2. Clippy：平台相关死字段

`MeasuredExport::owner_mode` 仅在 `#[cfg(unix)]` 分支读取。加
`#[cfg_attr(not(unix), allow(dead_code))]` 并说明原因。

### 3. 测试夹具依赖 Unix 工具（3 个用例）

| 用例 | 原夹具 | 问题 |
| --- | --- | --- |
| `a_runtime_that_dies_on_startup_...` | `/bin/sleep` | Windows 不存在，启动失败原因变成"文件未找到" |
| `the_watchdog_notices_a_crash_...` | `cmd /c "timeout /t 2 & exit 3"` | `timeout` 在无控制台/重定向输入下立即失败，崩溃落在 600ms 启动窗口内，**watchdog 路径从未被执行** |
| `crashing_command` | `cmd /c exit 3` / `sh -c exit 3` | 额外引入 shell 启动开销与平台差异 |

修复：在 `tests/support/stub_runtime.rs` 中新增三种**编译型**夹具——
拒绝参数退出、立即崩溃、2 秒后崩溃；全部平台同一行为。
`tests/web.rs` 改为使用它们，并按平台无关的消息断言。

### 4. `SQLITE_READONLY`：真实的产品缺陷

现象：`cargo test --test web` 并行执行时随机出现
`attempt to write a readonly database`，轮换命中不同用例（约每 2–3 次一轮）。
单测隔离运行 3/3 通过，排除单点缺陷。

定位过程：

1. 写临时并发复现（16 线程 × 80 轮，同一 workspace）→ **稳定复现**；
2. 文件属性显示 `neonexus.db`、`-wal`、`-shm` 均非只读 → 不是权限问题；
3. 改为"每线程只 `Repository::open` 一次" → 不再复现 → 指向 `open` 路径；
4. 检查 `Repository::open`：每次打开都执行
   `PRAGMA journal_mode = WAL` + 全量建表/索引/迁移（都会取写锁）；
5. 对照实验：把日志模式改为 `TRUNCATE` 后，web 套件 **4/4 全绿**。

根因有两条，互相叠加：

- **每次 `open` 都做写操作**。任何第二个句柄（CLI 巡检、测试夹具）都可能与正在写入的
  Web 服务争锁，并以 `SQLITE_READONLY` 失败。这不只影响测试：对运行中的服务执行
  `--config-drift` 等命令同样可触发。
- **WAL + 每操作新建连接**。WAL 在最后一个连接关闭时删除 `-shm`；
  连接高频开关会让下一次写入者撞上该转换，Windows 上表现为
  `SQLITE_READONLY`（而非 `BUSY`）。

修复（`src/repository/schema.rs`、`schema/connection.rs`）：

1. **按 schema 版本幂等初始化**：`PRAGMA user_version` 记录版本，
   已是最新则只做一次只读查询后返回，不再执行任何 DDL/WAL 语句。
   版本常量旁写明"修改 schema 必须递增"。
2. **日志模式改为 `TRUNCATE`**：去掉 `-shm` 生命周期，写入者看到确定状态。
   本产品是管理控制台（每秒少量小写入），不依赖 WAL 的读写并发。
3. **连接获取重试**：`ReadOnly` / `Busy` / `Locked` 以退避重试；
   打开后先执行 `PRAGMA schema_version` 强制挂载日志，把可失败的步骤
   从"调用方的写语句内部"移到可控的重试点。

修复后：web 套件连续 5 次运行无 `SQLITE_READONLY`，4 次完整通过（第 5 次为命令超时中断）。

### 5. 测试断言不检验真实结果

`restarting_node_over_http_...` 只断言 303，而失败路径同样返回 303，
所以"restart 没发生"被读成"事件没记录"。`lifecycle_controls_are_journaled`
同样只断言 303。修复后两者都断言 flash 中的实际结果
（`launched with PID` / `stopped`），失败时能直接指出原因。
restart 用例另把固定 `sleep(3s)` 换成 `wait_until(20s, Running)`。

---

## 三、最终验证

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo check --all-targets` | 通过 |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| `cargo test --lib` | 1000 passed / 1 ignored |
| `cargo test --test ci_policy` | 15 passed |
| `cargo test --test domain` | 147 passed |
| `cargo test --test repository` | 45 passed |
| `cargo test --test web` | 74 passed（连续 4 次全绿） |
| `cargo test --test integration` | 17 passed |
| `cargo test --test ui_density_metrics` | 7 passed |
| `cargo test --test ui_operator_walkthrough` | 4 passed |
| `--source-quality src` | 0 findings |
| `--source-purity .` | pure-rust，0 disallowed |

---

## 四、其他审计结论（本次未改动）

- 无 TODO/FIXME/XXX/HACK；24 处 `unwrap/expect/panic` 全部位于测试或文档示例。
- 925 个源文件，无文件超过 1000 行；当时最大 `web/assets/styles.rs` 924 行
  （该文件此后已拆成 `assets/style/` 下四个 Rust 片段，见 `overview.md`）。
- 孤儿源文件 0（`snapshots/import/paths/target_parts/*` 由 `#[path]` 挂载，非孤儿）。
- lint 抑制仅 8 处（6 处 `dead_code`，均有注释说明）。
- Web 层已无 `Repository` 直连逃生口。

---

## 五、后续建议

1. ~~`--source-quality` 建议增加"测试文件不得依赖平台专有可执行文件"的规则，
   这类问题在 Linux CI 上不可见。~~ → **已在本次以 CI 契约门禁的形式落地，
   见第六节。**
2. Web 套件的并行度较高，建议在 CI 中保留一次 `--test-threads=4` 的运行，
   以暴露只在竞争下出现的问题。
3. 若未来确实需要 WAL 的读写并发，应同时引入连接池，
   而不是恢复"每操作一连接 + WAL"的组合。
