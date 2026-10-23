# 迭代记录：2026-10-20（续三）

本文件收录第十至第十三节：任务 #50 与 #48，把会丢输入的表单全部改成失败重渲染。
总目录见 [ITERATION_2026-10.md](ITERATION_2026-10.md)。

---

## 十、任务 #50：signer 策略表单的失败重渲染

### 10.1 问题

`POST /signer/keys/{id}/policy` 走 `manage()` → `respond()`：失败一律
**303 跳回 key 页 + 一条 flash**。而这个边界是**二十九个字段**宽，操作员多半是
从设计文档里整段粘过来的。一次「窗口缺一半」的拒绝，代价是把三十个字段重贴一遍，
而且 flash 里那句话说不清是哪一半。

### 10.2 分层约束决定了改动的形状

`pages/signer/key.rs` 要画这个表单，就需要读到操作员原文（draft）；而
`PolicyForm` 原本定义在 `signer_control/forms.rs`。若 pages 直接依赖
`signer_control`，就与既有方向（`signer_control → pages`，见 `callers.rs` 里
`crate::web::pages::signer`）成环。

所以 `PolicyForm` 迁到中立模块 `src/web/signer_form.rs`，`forms.rs` 只留
`pub use crate::web::signer_form::PolicyForm;` —— **全项目仍只有一处定义**。
同一个 struct 既是 POST 载荷，又是页面回填的值来源（`NodeDraft` 的同一范式）。

### 10.3 三类失败，三种落点

失败不是一种。把三种混成一个 `anyhow::Error` 再统一处理，会把「服务说不行」
和「服务联系不上」压成同一句 `not saved`：

| 失败来源 | 谁的话 | 落点 |
|---|---|---|
| 本地 `build_policy` | 本进程 | **200 重渲染**，错误在该字段旁 |
| 远端 `Outcome::Refused(Refusal)` | 服务自己的 code/message | **200 重渲染**，`general` 桶，渲染在表单上方 |
| 传输失败（custody 不可达） | 传输层 | `general` 桶；页面画不出来时退回 **303 + flash** |

关键实现细节：`save_policy` 返回 `Result<Outcome<SavedBoundary>>`，
**不能**先 `.into_parts()?`——那样远端拒绝会变成 `anyhow::Error`，与传输失败
再也分不开。必须直接在 `ask` 的返回值上 match `Outcome::{Allowed,Refused}`。

成功路径仍然 303。理由写进了代码注释：本进程只知道**服务的答复**，200 会声称
一个它没有观察到的状态；跳转落地的页面会重新从服务读回边界。

### 10.4 错误归因：类型化，不是字符串匹配

`build_policy` 的返回类型从 `anyhow::Result<Policy>` 换成
`std::result::Result<Policy, PolicyRefusal>`：

```rust
pub struct PolicyRefusal {
    pub field: &'static str,   // HTML name，来自 signer_form 的常量
    pub message: String,       // 操作员读的那句话，一字未改
}

fn blame<T>(field: &'static str, result: Result<T>) -> std::result::Result<T, PolicyRefusal>
```

每个解析点写成 `blame(ALLOW_TRANSFER, switch(&form.allow_transfer, "transfers"))`
——「人类可读名」与「控件 name」写在同一行，改一个忘另一个会被 review 看见。
`field` 是 `&'static str` 常量，所以**不存在的 name 无法表达**。

**刻意没有**照抄 `control/settings.rs` 的 `upgrade_field_for`（按消息子串匹配）：
那条路一旦消息改写就静默落到 `general`。归错字段比不归因更糟——操作员会去改一个
本来没错的字段。配对错误（滚动窗口、签名速率窗口）归到**缺的那一半**：
`window_seconds` 空而金额有值 → 错在 `window_seconds`，不是金额。

### 10.5 页面的单一真值

`policy_form` 的每个控件现在都这样画：

```rust
for (label, name) in [
    ("Transfers", signer_form::ALLOW_TRANSFER),
    ...
] {
    html::ChoiceField {
        label,
        name,
        selected: form.value(name),                  // FormDraft
        error: form_state::error_for(draft.errors, name),
        ...
    }
}
```

控件用的常量和取值的常量是**同一个**，所以把 A 字段的文本画在 B 字段的标签下
这件事在结构上不可能发生。`FormDraft::value` 的 29 条 match 臂是唯一一处
name → 槽位的映射；`POLICY_FIELDS` 列全 29 个名字，配一条 sentinel 单测
（每个字段填自己的名字，逐个断言 `value(name) == name`，并断言无重名）。

未保存必须看得出来：`policy_summary` 依次渲染「N fields need attention.」、
`general` 原文、以及一行 muted 说明——**上面的卡片仍然是服务正在执行的边界，
下面的表单是你打的字**。两者在拒绝之后确实不一致，页面必须说出来。

### 10.6 降级路径

重渲染要读 key / audit / callers 三份数据，是**又一次**与服务的对话。若这次
也失败（服务在保存和重绘之间挂了），输入就无法保留。此时退回 303，并在 flash 里
明说：`not saved: … — and the page could not be redrawn (…), so the form was lost`。
丢输入是坏事，把原因一起丢掉更坏。

### 10.7 测试

- `tests/unit/web/signer_control/tests.rs`（+1）：
  `a_refused_boundary_names_the_field_the_operator_has_to_change` —— 6 组配对
  与解析失败，逐一断言落在哪个控件。
- `tests/unit/web/signer/tests.rs`（+1）：`every_policy_field_reads_its_own_slot`
  —— 29 个字段各自读自己的槽位，且 `POLICY_FIELDS` 无重名。
- `tests/web.rs`（+2 改写 1 条）：`the_boundary_reported_back_is_the_one_the_service_stored`
  的「半个窗口」断言由 303 + `not%20saved` 改为 **200 + 原文保留 + 错误落在
  `f-window_max_amount`**；新增
  `a_refused_boundary_that_cannot_be_redrawn_still_says_why` 固定降级路径
  （303 + 同时含拒绝原因与「页面画不出来」），以及
  `a_policy_the_service_refuses_is_quoted_above_the_form`——服务自己措辞的拒绝
  必须被**原文引用**在表单上方，且**不得**编造「One field needs attention.」的计数。

**注入验证**（不是只看到变绿）：

1. `signer_form.rs` 把 `ALLOW_RAW` 的臂指向 `&self.allow_consensus`
   → `every_policy_field_reads_its_own_slot` 失败（`left: "allow_consensus"`）。
2. `forms.rs` 把「缺长度」那支的 field 从 `WINDOW_SECONDS` 改成
   `WINDOW_MAX_AMOUNT` → `a_refused_boundary_names_the_field_...` 失败
   （`left: "window_max_amount"`）。
3. `key.rs` 的 `policy_summary` 去掉 `{general}` →
   `a_policy_the_service_refuses_is_quoted_above_the_form` 失败
   （「the service's own code was reduced to a generic failure」）。

三处均已逐字节还原（`cmp` 校验）。

### 10.8 同一轮的后半：把 `general` 桶修好，再把原语收敛

10.4 提到「刻意没有照抄 `control/settings.rs` 的 `upgrade_field_for`」。
沿着这条线回去看 settings 页，发现它的问题不是归因精度，而是**更基础的一条：
`general` 桶从来没有被渲染过**。`FieldErrors` 里放了整条策略级别的拒绝，
`render_body` 却只把 `errors` 传给各字段控件——字段控件只认自己那个 name，
于是 `general` 键被安静地跳过。页面回来的样子是：**输入完好、说得清的错因不见了**，
操作员看到的是一个「保存好像没发生」的表单。这比 303 + flash 更糟：
flash 至少还说了一句理由。

三处本地实现（`pages/node_editor` 正确、`pages/settings` 丢 `general`、
`pages/signer/key` 新写一份）合并为一个原语：

```rust
// src/web/form_state.rs
pub fn banner(errors: &FieldErrors) -> String   // 计数（不含 general）+ general 原文
```

`summary` 继续只数具名字段；`banner` = `summary` 的 notice + `general` 的 notice。
两者**不能互相替代**：「One field needs attention.」压在一条关于整条策略的
拒绝上，会指认一个没有错的控件；反过来只引原文而不计数，操作员不知道有几处要改。
字段自己的消息**不进** banner——它由控件画在自己旁边，重复一遍会让页面说两遍，
且第二遍离输入框很远。这条边界用单测钉住
（`the_form_banner_counts_fields_and_quotes_what_belongs_to_none`：
具名字段时断言 banner **不**含该消息，`general` 独占时断言 banner **不**含计数）。

有了原语之后，settings 页剩下三个策略表单（watchdog / rpc-health /
federation monitor）从「失败即丢输入」改成 200 重渲染：

- 草稿类型 `WatchdogDraft` / `MonitorDraft`（纯 `String`，原样保留打字内容）；
- `RefusedForm` 枚举承载**哪一个面板**被拒——两个 monitor 表单 post 的
  `enabled` 与 `interval_seconds` **同名**，用一张共享错误表会让两个面板
  同时被指责；枚举让「错在哪个面板」在类型上只有一个答案；
- `finish(outcome, blame, draw)` 统一尾巴：成功 303 + flash；失败把错误
  放进 `blame(&error)` 指认的字段，重画 200；重画本身失败则退回 303 并
  同时说出错因与「页面画不出来」。

注入验证（新增一处，逐字节还原；前三处见 10.7）：

4. `pages/settings.rs` 让 `RefusedForm::Federation` 分支也匹配 `RpcHealth`
   → `a_monitor_refusal_lands_in_its_own_panel` 失败（「the federation panel
   was blamed for the RPC health mistake」）。

另有三条端到端用例为这一半新写或改写：watchdog 的 `seven` 与
`invalid_jitter_values`、settings 的 `max_nodes_per_run=abc`
（`a_settings_refusal_that_belongs_to_no_field_is_still_shown`）——
原先都断言 303，现改为断言 **200 + 原文字段旁 + 草稿保留**。

## 十一、快照登记表单的失败重渲染

### 11.1 问题

`POST /snapshots/save` 是九个字段：id、label、network、node_type、source_path、
source_url、download_file_name、download_max_bytes、expected_sha256。其中一个是
**64 字符的摘要**——最容易打错、也最不该重打的一个。此前失败一律
`303 + flash` 回 `/snapshots`，九个字段全丢。

顺带更正上一轮遗留清单里的一处误判：`control/node.rs`、`control/artifacts.rs`
被列为「失败即丢输入」，实际**没有任何文本输入**（启动/停止/重启/冒烟/导出/
对账全是按钮），不存在可丢的东西。真正剩下的表单族比清单写的少。

### 11.2 分层：字段名与草稿都在中立模块

新增 `src/web/snapshot_form.rs`，与 `signer_form.rs` 同构：

```rust
pub const ID/LABEL/NETWORK/NODE_TYPE/SOURCE_PATH/SOURCE_URL/
        DOWNLOAD_FILE_NAME/DOWNLOAD_MAX_BYTES/EXPECTED_SHA256;
pub const SNAPSHOT_FIELDS: [&str; 9];
pub struct SnapshotDraft { /* 九个 String，既是载荷也是回填源 */ }
impl FormDraft for SnapshotDraft;
pub struct SnapshotRefusal { pub field: &'static str, pub message: String }
pub fn build(&SnapshotDraft) -> Result<NewFastSyncSnapshot, SnapshotRefusal>;
```

`snapshot_ops.rs` 的 `SaveSnapshotForm` 被删掉，改用 `SnapshotDraft`——**同一份
结构既承载提交也承载重画**，name → 槽位的映射全项目仍只有一处。

### 11.3 只归因「能确定」的字段

`build` 里做了类型化的判断：network / node_type 解析失败、`download_max_bytes`
不是整数、id 与 label 为空、source_path 与 source_url 同时为空（这一对归 `general`
——两半都缺，没有「缺的那一半」可指）、摘要不是 64 位十六进制。

摘要这条**没有复制规则**，而是直接调用域自己的 `snapshots::normalize_sha256`，
只是**更早**调用一次，好把答案贴到控件上：

```rust
crate::snapshots::normalize_sha256(&draft.expected_sha256)
    .map_err(|error| SnapshotRefusal::at(EXPECTED_SHA256, error))?;
```

剩下的交给 workspace：URL 不是 HTTPS、文件名会被下载器拒绝，这些**不猜**。
猜错字段比不归因更糟——被指到一个本来就对的控件上的操作员，没有任何线索
找到真正错的那个。所以 `upsert_fast_sync_snapshot` 的错误进 `general`，
原文渲染在表单上方。

### 11.4 页面

`pages/snapshots.rs` 新增 `render_with_draft(workspace, draft, errors)`，返回
**body** 而不是 `Response`（布局由调用方套）。`register_form(refused)` 里每个
控件都用同一个常量画 name、取值、取错误：

```rust
html::TextField {
    name: snapshot_form::EXPECTED_SHA256,
    value: value(snapshot_form::EXPECTED_SHA256),
    error: error(snapshot_form::EXPECTED_SHA256),
    ..
}
```

首次访问时 network / node_type 仍有默认选中（Mainnet / 第一个 runtime），
重画时改为显示操作员提交的值——这是「默认值」与「你打的字」的区别，代码里
两个分支分开写。

降级路径与 signer 页一致：重画要再读一次 inventory，读失败就 `303 + flash`
并说明「页面画不出来、输入已丢」。

### 11.5 测试与门禁

- `tests/unit/web/snapshot_form/tests.rs`（+4）：7 组归因用例表、
  接受时 trim 与默认值、只有 URL 也算给了来源、九个字段各读自己的槽位。
- `tests/web.rs`（+1）：`a_refused_snapshot_manifest_comes_back_with_what_was_typed`
  —— 200、原因在页面上、`id="f-expected_sha256-error"`、
  `value="deadbeef"` 与 `value="Typo Snapshot"` 都在、含「Nothing was saved」、
  且**没有**写进 workspace。
- 注入验证 4 处（全部逐字节还原）：
  1. `value()` 的 `EXPECTED_SHA256` 臂指向 `&self.label`
     → `every_field_reads_its_own_slot` 失败（`left: "label"`）。
  2. 摘要拒绝的 field 从 `EXPECTED_SHA256` 改成 `GENERAL`
     → 归因用例失败（`left: "general"`）。
  3. 页面里摘要控件的 `error:` 改成 `None`
     → 端到端用例失败（「the reason should sit beside the digest field」）。
  4. `redraw` 传 `&SnapshotDraft::default()` 而非操作员的草稿
     → 端到端用例失败（「the field should still hold what was typed」）。

- `cargo fmt --all -- --check` —— clean
- `cargo clippy --all-targets -- -D warnings` —— clean
- `cargo test --lib` — **1023** passed，1 ignored
- `cargo test --test ci_policy` — 48 passed
- `cargo test --test domain` — 147 passed
- `cargo test --test repository` — 45 passed
- `cargo test --test web` — **86** passed
- `cargo test --test integration` — 17 passed
- `cargo test --test ui_density_metrics` — 7 passed
- `cargo test --test ui_operator_walkthrough` — 4 passed

合计 **1377 项**（上一轮 1365）。

- `cargo run -- --source-quality src` —— findings: 0
- `cargo run -- --source-quality tests` —— findings: 0
- `cargo run -- --source-purity .` —— disallowed: 0
- `cargo run -- --ci-policy .github/workflows/ci.yml` —— findings: 0

## 十二、最后两个会丢输入的表单：备份还原与钱包导入

### 12.1 两个都是「只有一个到三个字段，但字段很长」

- `POST /backup/import`：面板只有一个「归档路径」。路径在服务器上，长且每次都不同，
  而**正是这个路径通常就是失败原因**。用 303 回答，等于让操作员为了读一句话
  再粘贴一次——而这是本页唯一能把整个 workspace 覆盖掉的操作。
- `POST /wallets/import`：路径 + 可选 id + 可选 label。三字段里最费时的是路径。

两处都改成写进 11.x 已建立的那套：草稿 + 字段级错误 + 200 重渲染 + 降级到 303。

### 12.2 字段名归属

两页原先都是**手写 HTML**，没有错误槽位。改成 `html::TextField`，并把字段名收敛成
常量：

- `pages/backup.rs`：`pub const ARCHIVE_PATH`；
- `pages/wallets.rs`：`pub const WALLET_PATH / WALLET_ID / WALLET_LABEL`
  + `WalletImportDraft`（`impl FormDraft`）。

`control/maintenance.rs` 与 `wallet_ops.rs` 都只**单向**依赖 `pages::*`
（`control/settings.rs` 早已是同一方向），没有反向依赖。`wallet_ops` 原来的
`WalletImportForm`（三个 `Option<String>`）删除，直接 `Form<WalletImportDraft>`
——同一结构既做载荷也做回填，与 `SnapshotDraft` 一致。

### 12.3 钱包导入的两类失败分得开

`wallet_ops` 里新增一个小类型 `Refusal { field, message }`：

- 路径不存在 / 不在 workspace 下 / 钱包校验失败 → `WALLET_PATH`。
  这三件事都是**关于那个文件的**，指到路径控件是诚实的。
- `upsert_neo_wallet_profile` 失败 → `GENERAL`。文件本身没问题、存不进去是
  workspace 的问题，不属于任何字段。
- `spawn_blocking` 的 JoinError → `GENERAL`。

顺带一个必须处理的细节：`state` 原来整个 move 进 blocking 闭包，而重画页面
还需要它。改成 `state_for_task = state.clone()` 进闭包、`state` 留给重画——
漏掉这一步会得到一个借用错误，而不是一个运行时缺陷。

### 12.4 测试与门禁

- `tests/web.rs`（+3）：
  - `a_restore_that_cannot_read_the_archive_keeps_the_path`
  - `a_refused_wallet_import_comes_back_with_the_form`（含「没有写进 workspace」）
  - `a_wallet_import_with_no_path_keeps_what_was_typed`（空路径也是 200）
- 注入验证 3 处（逐字节还原）：
  1. 备份面板 `value: path` 改成 `value: ""` → 路径断言失败。
  2. `wallet_ops::redraw` 传 `WalletImportDraft::default()`
     → 「the path should still be in the field」失败。
  3. 钱包页路径控件的 `error` 改成 `None`
     → 「the reason should sit beside the path field」失败。

**行尾事故一则**：`src/web/pages/backup.rs` 原是纯 CRLF，改完变成纯 LF
（`bare_LF` 从 0 变成 193）。`git diff` 看不出来（clean filter 会归一化），
只能靠字节统计发现，已按字节改回 CRLF。**CRLF 文件改完必须统计字节**。

- `cargo fmt --all -- --check` —— clean
- `cargo clippy --all-targets -- -D warnings` —— clean
- `cargo test --lib` — **1023** passed，1 ignored
- `cargo test --test ci_policy` — 48 passed
- `cargo test --test domain` — 147 passed
- `cargo test --test repository` — 45 passed
- `cargo test --test web` — **89** passed
- `cargo test --test integration` — 17 passed
- `cargo test --test ui_density_metrics` — 7 passed
- `cargo test --test ui_operator_walkthrough` — 4 passed

合计 **1380 项**（上一轮 1365）。

- `cargo run -- --source-quality src` —— findings: 0
- `cargo run -- --source-quality tests` —— findings: 0
- `cargo run -- --source-purity .` —— disallowed: 0
- `cargo run -- --ci-policy .github/workflows/ci.yml` —— findings: 0

## 十三、运行目录登记表单（#48 的最后一项）

### 13.1 六字段、一个数字、一个在最下面

`POST /runtimes/catalog`：id、label、source、signature_source、ed25519_public_key、
max_bytes。最容易打错的是**最下面那个数字**，而表单在「已装运行时 / 目录清单」
之后——失败 303 回页顶，等于把上面全部重打一遍。

### 13.2 新模块而不是塞进页面

`pages/runtimes.rs` 当时已 633 行，再加草稿与渲染会到 750。新增
`src/web/catalog_form.rs`（与 `signer_form.rs` / `snapshot_form.rs` 同构）：

```
CATALOG_{ID,LABEL,SOURCE,SIGNATURE_SOURCE,PUBLIC_KEY,MAX_BYTES} + CATALOG_FIELDS
CatalogDraft（六 String，既是载荷也是回填源）
CatalogRefusal { field, message }
build(&CatalogDraft) -> Result<RuntimeCatalogProfile, CatalogRefusal>
DEFAULT_CATALOG_MAX_BYTES 也搬到这里（页面占位符与 build 默认值共用一处）
```

`pages::runtimes::CatalogForm` 删除，直接 `Form<CatalogDraft>`。

### 13.3 归因边界：单字段归位，组合归 `general`

能确定落点的四个：`max_bytes` 非整数、id / label / source 为空。
其余全部交给域：source 是不是可读的 HTTPS 或路径、签名源与公钥是否**同时**
给出、公钥是否恰好 32 字节——这些是**关于组合**的一句话，指到三个控件中的哪一个
都是猜。域的原文进 `general`，渲染在表单上方。

对应的端到端用例专门钉住了「组合类拒绝不得被计成字段」：
`a_catalogue_refusal_about_two_fields_is_quoted_above_the_form` 断言页面含
`must be provided together` 且**不含** `One field needs attention.`。

### 13.4 测试与门禁

- `tests/unit/web/catalog_form/tests.rs`（+4）：归因表（4 组）、接受时 trim 与默认值、
  可选字段空串 → `None`、六字段各读自己的槽位。
- `tests/web.rs`（+2）：`a_refused_catalogue_comes_back_with_what_was_typed`
  （200 + `f-max_bytes-error` + 六字段保留 + 未落库）、
  `a_catalogue_refusal_about_two_fields_is_quoted_above_the_form`。
- 注入验证 3 处（逐字节还原）：`max_bytes` 的拒绝字段改成 `CATALOG_ID`
  → 归因用例失败（`left: "id"`）；`redraw_catalog` 传 `CatalogDraft::default()`
  → 「the field should still hold what was typed」失败；页面把 `error` 改成 `None`
  → 「the reason should sit beside the size limit」失败。

### 13.5 一条被证伪的旧结论

上一节把 `pages/backup.rs` 的 CRLF→LF 归因于 `cargo fmt`。本轮用一个混合行尾的
探针文件直接跑 `rustfmt` 验证：**rustfmt 不改行尾**（3 CRLF + 1 bare 进去，
原样出来）。所以行尾变化来自编辑工具本身，不是格式化。修正后的纪律：

- 改完 CRLF 文件就统计 `CRLF` / `bare_LF`，`rustfmt` **不会**替你收拾混合行尾；
- `git ls-files --eol <path>` 能直接看出工作树当前是 `w/crlf` 还是 `w/lf`。

- `cargo fmt --all -- --check` —— clean
- `cargo clippy --all-targets -- -D warnings` —— clean
- `cargo test --lib` — **1027** passed，1 ignored
- `cargo test --test ci_policy` — 48 passed
- `cargo test --test domain` — 147 passed
- `cargo test --test repository` — 45 passed
- `cargo test --test web` — **91** passed
- `cargo test --test integration` — 17 passed
- `cargo test --test ui_density_metrics` — 7 passed
- `cargo test --test ui_operator_walkthrough` — 4 passed

合计 **1386 项**（上一轮 1365）。

- `cargo run -- --source-quality src` —— findings: 0
- `cargo run -- --source-quality tests` —— findings: 0
- `cargo run -- --source-purity .` —— disallowed: 0
- `cargo run -- --ci-policy .github/workflows/ci.yml` —— findings: 0
