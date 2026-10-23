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

## 最新迭代（2026-10-20）

对齐 `origin/main` = `63e4731`，详细记录见 `docs/ITERATION_2026-10.md`。

远端历史被 force-push 重写（本地 `82faf93` 与远端分叉），已建备份分支
`backup/local-main-82faf93` 后对齐，未丢失本地提交。

修复的真实问题：

- **`SQLITE_READONLY`（"attempt to write a readonly database"）**：`Repository::open`
  每次都执行 journal-mode 与全量 DDL，会与运行中的服务争写锁；叠加 WAL 在连接
  全部关闭时删除 `-shm`，Windows 上随机导致写失败。现改为按 `PRAGMA user_version`
  幂等初始化、日志模式使用 `TRUNCATE`、连接获取对 `ReadOnly/Busy/Locked` 退避重试。
  这同时影响生产路径：对运行中的服务执行 CLI 巡检命令也会触发。
- **watchdog 崩溃路径在 Windows 上从未被测试**：原夹具用 `cmd /c timeout /t 2`，
  而 `timeout` 在无控制台时立即失败，崩溃落在 600ms 启动窗口内。改用编译型
  stand-in 二进制，watchdog 路径在两个平台都被真实执行。
- **`ci_policy` 在 Windows 必然失败**：workflow 步骤匹配硬编码 LF，而
  `core.autocrlf=true` 检出为 CRLF。现统一行尾后匹配。
- **测试断言不检验结果**：restart / lifecycle 用例只断言 303（失败路径也是 303），
  已改为断言 flash 中的实际结果，并把固定 `sleep(3s)` 换成状态轮询。
- **Clippy**：修复仅在 Unix 分支使用的字段在 Windows 上的 `dead_code`。
- **两条 CI 契约规则此前是空的**：平台规则只认 raw string，普通字符串
  `Command::new("/bin/sleep")` 直接放行；端点规则抓所有以 `/` 开头的引号字符串，
  注释遮蔽的两条路由取不到、非路径字符串反而被误捕，且不比较 HTTP 方法。
  现改为用 `syn` 读语法树：cfg 谓词按 CI 矩阵四样本做三值求值，平台专有字面量
  必须被编译期 `#[cfg]` 门控保护；路由与文档在 method + path 双维度上比对。
- **`--source-quality tests` 此前为红**：`workflow_shells.rs` 用 `panic!` 报告
  缺失的 workflow step，既违反全局禁令也会杀掉整个测试二进制。改为可恢复错误。
- **两条红灯来自并发与端口，不是产品缺陷**：`cargo test --lib` 有 flaky
  （日志采集器「先采集再 park 30s」，而用例在启动后才写日志文件且从不唤醒它）；
  `cargo test --test web` 则 6/6 必败——测试把端口硬编码在操作系统临时端口区
  （Windows 49152+、Linux 32768+），被本机代理的一条出站连接占用后，
  readiness 预检正确地拒绝启动节点。两处均已修正：显式唤醒采集器，
  并把 22 组测试端口迁到 26032–27233。

- **内容安全策略让三个控件静默失效**：控制台发的是
  `script-src 'sha256-<hash>'`，**没有 `'unsafe-inline'`**，浏览器因此拒绝编译
  任何内联事件处理器。全项目 3 处内联处理器全部失效：日志页「清除日志」的
  `onclick="return confirm(...)"` **确认框从不出现**，表单直接提交，删掉工作区
  全部日志文件；节点编辑页的 `onchange="toggleNodeRpcService(...)"` 让 JSON-RPC
  开关点击后不展开端口面板；`onclick="selectNodeRolePreset(this)"` 让职责预设卡
  点击后不填表。这三处**没有任何测试失败**——页面渲染的 markup 与从前一模一样，
  差别只在浏览器执行与否。现改走 `data-*` 属性 + 脚本事件委托，「清除日志」
  改为服务器端两段式确认页（`GET /logs/clear`），与 JavaScript 无关。
- **新增第三条 CI 契约规则：禁止内联事件处理器**。用 `syn` 读语法树判定，
  注释提及属性、散文提到属性、属性值里的字样都不算违规，只有 markup 里真正的
  事件属性才算。第一版因为 `syn::visit` **不进入宏调用的实参**而拦不住任何
  东西（本项目几乎全部用 `format!` 渲染 markup），覆写 `visit_macro` 后修复。
- **失败的反馈和成功的反馈长得一模一样**：所有控制用同一套 `.flash` 样式
  报告结果，运维必须逐字读完横幅才能分辨「启动失败」与「启动成功」。现引入
  `html::Flash { message, kind }`，严重度随消息一起在 query 里传递，失败渲染为
  红色 + `role="alert" aria-live="assertive"`，成功为绿色 + `polite`。
  **56 处失败路径**现在显式标注。
- **长耗时操作钉住工作线程**：导出支持包、写就绪报告、完整性检查、备份导出与
  导入都在 `async fn` 里同步调用，占住一个 Tokio worker 线程——服务器用固定大小
  的线程池服务所有请求，一次「导出支持包」可以让整个控制台为之一滞。现统一
  迁入 `spawn_blocking`（`control/node.rs` 早有正确范式，此处未遵守）。
- **创建 API 密钥可能落到空白页**：未勾选权限时返回裸 `400` 纯文本
  （浏览器端完全可达，权限是三个无 `required` 的 checkbox），导航与已填内容
  全部丢失。现改为重渲染表单 + 页面级提示 + **保留已输入的 name 与勾选状态**。
  同时修掉一个功能性缺陷：新建密钥页返回的是裸 HTML 片段，没有 `html::layout`，
  因此 **「Copy Secret Key」按钮永远不工作**（复制脚本只由 layout 注入）。
- **危险操作没有视觉语义**：`danger_control_form` 全项目只用了 2 处，插件/
  联邦服务器/签名密钥/Caller 的 Disable 用的是普通按钮，钱包 Delete 竟是纯文字
  链接。现统一为**关闭用 danger、开启用普通按钮**（把能力放回去不是会出事的那步）。
- **裸表格**：`api_tokens.rs` 手写的表 `<th>` 缺 `scope="col"`、`<td>` 缺
  `data-label`，外层 class 在 CSS 里根本不存在（移动端因此渲染出空白）。
  迁到 `html::table` 原语后一次性补齐六项无障碍属性。

新增的产品能力：

- **`GET /ready`**：面向编排器的就绪探针，工作区被阻塞或无法判定时返回 `503`，
  供负载均衡停止导流；公开、无需会话，与 `/api/readiness` 的分工已写入文档。
- **`GET /api/jobs`**：后台任务列表的 JSON 形态，供流水线轮询长任务进度。


## 最终验证结果

全部通过：

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --lib` — 1016 passed，1 ignored
- `cargo test --test ci_policy` — 48 passed
- `cargo test --test domain` — 147 passed
- `cargo test --test repository` — 45 passed
- `cargo test --test web` — 81 passed
- `cargo test --test integration` — 17 passed
- `cargo test --test ui_density_metrics` — 7 passed
- `cargo test --test ui_operator_walkthrough` — 4 passed
- `cargo run -- --source-quality src` — 929 文件，0 findings
- `cargo run -- --source-quality tests` — 319 文件，0 findings
- `cargo run -- --source-purity .` — pure-rust，0 disallowed
- `cargo run -- --ci-policy .github/workflows/ci.yml` — 0 findings，三平台齐备

三条 CI 契约规则另做了逐条注入验证：12 个注入用例（普通字符串、raw string、
门控方向相反、feature 门控、OpenAPI 虚假端点、手册虚假端点、内联事件处理器、
`format!` 内的内联事件、注释提及属性、`data-*` 委托标记等）中，
9 个必须被拦的都被拦下，3 个必须放行的都被放行，12/12 与预期一致；
被注入的文件均以逐字节比对确认还原。

收尾复验时注入验证一度变成 11/12：某用例的 payload 在 Python 源码里写成
`\"go()\"`，落到 Rust 源码引号不配对而**编译失败**——脚本正确地把它判成
「没有测到规则」。payload 改用 raw string，并补两条常驻单测覆盖
**转义引号形态**的 `format!` 实参（规则读的是解转义后的 `literal.value()`，
此前无任何测试证明这条路径）。修正后 12/12。

## 第二轮：五项遗留全部落地

上一轮记档的 5 项遗留（详见 `docs/ITERATION_2026-10-audit.md` 第 7.11 节）本轮全部做掉。
做法与上一轮不同：先派两个 Explore agent 并行取证（`note()` 的 59 处调用点精确分类、
表单约束该加在哪、通用表单原语的设计边界），拿到带行号证据的结论再动手。

- **联邦服务器的编辑入口**：`POST /federation/{id}` 路由早已存在，却没有任何 UI
  表单指向它——功能存在但用户碰不到。新增 `GET /federation/{id}/edit`、
  列表行的 Edit 入口，失败时**返回 200 重渲染**并保留已填内容。
- **表单约束**：`TextField` 新增 `required` / `min` / `max` / `step` / `pattern` /
  `maxlength` / `input_type`，`ChoiceField` 新增 `required`。约束**按校验语义加**：
  拒绝型加约束是纯收益；空值合法（端口留空 = 自动分配）只加 min/max；
  **金额字段刻意不解析**（`"1e6"` 必须能到达 custody 服务由它拒绝）→ 不加
  `type="number"`；跨字段配对约束 HTML 表达不了 → 不给任一字段加 `required` 充数。
  顺带修掉维护窗口上界的 off-by-one（1440 → 1439）。
- **空状态迁移**：19 处 `note()` → `empty_state()`，全部带前进路径的动作
  （「没有前进路径的空列表是死路」）。调用点从 59 降到 40。
  新增 `empty_state_at(level, …)` 解决标题层级倒挂。
- **表单状态回传**：新建共享原语 `src/web/form_state.rs`（`FieldErrors` +
  失败重渲染）。运行时升级策略表单改为失败返回 200 重渲染 + 字段级错误 + 输入保留。
  **刻意不抽泛型表单引擎**——三族表单字段零重叠、错误模型不同、真值来源不同。
- **长耗时操作的 pending 提示**：`html::pending_marker` + CSS + `submit` 事件委托，
  插入 6 处。**不调用 `preventDefault`**——长耗时表单仍必须能提交，
  脚本禁用时页面行为不变。

验证脚本本身也修了三个缺陷：**编译失败曾被当成「规则拦截」**（两者都退出非零）、
**中途被打断会留下注入探针**（`finally` 没跑到，`neo_cli_adapter.rs` 里留下过
一段 `ci_contract_injection_probe`），以及一处锚点转义错误。脚本现在开始前备份、
退出时无条件还原、并主动检查残留。

## 第三轮：signer 策略表单的失败重渲染

`POST /signer/keys/{id}/policy` 是二十九个字段的边界，此前仍是「失败即丢输入」：
一次「窗口缺一半」的拒绝，代价是重贴整份边界，而 flash 里那句话说不清是哪一半。

- `PolicyForm` 迁到中立模块 `src/web/signer_form.rs`（`pages/` 不得反向依赖
  `signer_control/`，否则成环），`forms.rs` 只留 `pub use`——全项目仍只有一处定义。
  与 `form_state.rs` 一样复用的是**原语**而不是引擎：字段名常量 + `FormDraft`
  + 失败重渲染，各表单族自己画自己的字段。
- 三类失败分开落点：本地解析失败 → 200 + 错误在该字段旁；远端 `Refusal` → 200
  + `general` 桶（服务自己的 code 与 message）；传输失败 → 仍是 303，页面画不出来
  时明说输入已丢。成功路径仍 303——本进程只知道服务的答复，200 会声称一个
  它没有观察到的状态。
- 错误归因**类型化**而非按消息子串匹配：`PolicyRefusal { field, message }` +
  `blame(field, …)`。子串匹配会在文案改写时静默落到 `general`，
  而归错字段比不归因更糟——操作员会去改一个本来没错的字段。
- 页面用**同一个常量**画控件和取值（`form.value(name)`），把 A 字段的文本画在
  B 字段标签下在结构上不可能；`POLICY_FIELDS` 与一条 sentinel 单测固定住
  「每个 name 读自己的槽位」。
- 注入验证 4 处（改错槽位、归错字段、丢掉远端原话、把 RPC 的错算到联邦面板）
  均被单测拦下，改动文件逐字节还原。

同一轮的后半段把 `general` 桶的缺陷修掉：settings 页的 `render_body` 只把错误
传给字段控件，而字段控件只认自己那个 `name`，于是整条策略级别的拒绝被**安静跳过**
——页面回来时输入完好、错因不见了，比 303 + flash 更糟（flash 至少说了句理由）。
三处本地实现收敛为 `form_state::banner`（计数 + `general` 原文；字段自己的消息
不进 banner，它由控件画在自己旁边）。顺带把 settings 剩下三个策略表单
（watchdog / rpc-health / federation monitor）改成 200 重渲染：`RefusedForm`
枚举承载「哪个面板被拒」——两个 monitor 的 `enabled` / `interval_seconds` 同名，
共享一张错误表会让两个面板同时被指责。

## 第四轮：快照登记表单

`POST /snapshots/save` 是九个字段，其中一个是 64 位摘要——最容易打错、也最不该
重打的一个。此前失败一律 303 回库存页，九个字段全丢。

- 新增 `src/web/snapshot_form.rs`（与 `signer_form.rs` 同构）：
  `SnapshotDraft` 既是提交载荷也是回填源，九个字段常量 + `SNAPSHOT_FIELDS`，
  `build()` 返回 `SnapshotRefusal { field, message }`。
- **只归因能确定的字段**：network / node_type / 字节数 / id / label / 摘要。
  摘要**不复制规则**，直接调用域自己的 `normalize_sha256`，只是更早调用一次。
  剩下的一律交给 workspace，其原话进 `general` 渲染在表单上方——
  猜错字段比不归因更糟。
- 页面每个控件用同一个常量画 name、取值、取错误；首次访问仍给默认选中，
  重画时显示提交值。
- 注入验证 4 处（错槽位、归错字段、错误不落控件旁、重画丢草稿）全部被拦下，
  改动文件逐字节还原。
- 顺带更正：`control/node.rs` 与 `control/artifacts.rs` 被列为「失败即丢输入」
  是误判——它们只有按钮，没有文本输入。

同一轮还把最后两个会丢输入的表单改完：`POST /backup/import`（只有「归档路径」，
而路径通常就是失败原因，也是本页唯一能覆盖整个 workspace 的操作）与
`POST /wallets/import`（路径 + id + label）。两页原先都是手写 HTML、没有错误槽位，
现在改用 `html::TextField` 并把字段名收敛成常量；钱包导入把「文件有问题」归到
路径控件、「存不进去」归 `general`，两类失败分开落点。

收尾的是 `/runtimes` 的「注册目录」表单（6 个字段，含 ed25519 公钥与签名源），
新建 `src/web/catalog_form.rs`。它与前几个的区别在于**归因边界**：单字段能确定的
（id/label/source 为空、字节数非整数）落到控件旁；「签名源与公钥必须同时给出」
这类**组合**判定交给域，原文进 `general`——指到三个控件里任一个都是猜。

这一轮还补上了两处「只有 JavaScript 在拦」的破坏性操作。`/settings/api-tokens`
的 Revoke 用的是 `data-confirm`——它是打包脚本的行为，脚本禁用时第一次点击就把
密钥吊销了；`/federation` 的 Remove 连提示都没有，直接从表格 POST，连带删掉探测
历史。两处都改成仓库已有的两段式：`GET …/delete` 渲染确认页，`POST` 才执行。
确认页写清「失去什么、留下什么」——token 页列出即将被收回的授权，说明立即生效、
secret 只存哈希因而无法补发；peer 页说明探测历史一并删除、对端本身不受影响、
重新加入要重填名称与 URL。列表里的按钮从 `<form>` 换成 `<a>`，破坏性一步因此
永远不是一次点击。

门禁：**1391** 项（上一轮 1386），fmt / clippy / source-quality / source-purity /
ci-policy 全绿。详见 `docs/ITERATION_2026-10-forms.md` 第十一至十四节。

## 仍需关注

- `/public-metrics` 的认证契约已收敛：代码用 `ReadFleet` 强制关闭匿名（`src/web/router.rs`），`docs/AGENT_API.md` 已同步，`tests/web.rs` 有断言（匿名 401、`read_fleet` token 200）。
- ~~本次迭代的文件变更应按主题拆成独立提交；发布前仍需 release build/package 验证~~
  → 已完成：93 个文件拆成 8 个提交推送到 `origin/main`（`3ab987e`）；
    release 构建 + `--package-release` + `--verify-release-package-integrity` 通过。
- Web 层还剩一个 600+ 行文件：`web/api/hermes_mcp.rs`（788 行，主要是 `mcp_endpoint` 里
  的 `tools/list` 与 `tools/call` 两个大分支）。`assets/styles.rs` 已改为
  `include_str!("console.css")`，`pages/nodes/detail_tabs.rs` 已按标签页拆成
  `summary` / `health` / `monitoring` / `configuration` / `access` 五个模块。
- ~~建议给 `--source-quality` 增加"测试不得依赖平台专有可执行文件"的规则~~
  → 已以 `ci_policy` 契约门禁落地（`tests/ci_policy/platform.rs`）。
- ~~建议把孤儿源文件检测固化进 `ci_policy`~~
  → 已落地（`tests/ci_policy/orphan_modules.rs`）。
- 本轮审计出但未修的（详见 `docs/ITERATION_2026-10-audit.md` 第 7.11 节）：
  13 个表单族仍是「失败即丢输入」（收益最大的是 signer 策略约 30 个字段）；
  `TextField`/`ChoiceField` 缺 `required`/`min`/`max`/`pattern` 浏览器端约束；
  20 余处空状态用 `note()` 而非 `empty_state()`，仍是死路；
  `POST /federation/{id}` 存在但无 UI 表单指向它（编辑联邦服务器不可达）；
  `loading_callout` 全项目只用了 1 处，长耗时操作在页面上无 pending 提示。
  → 上述 5 项已在第二轮全部处理（见上节），signer 策略表单（约 30 字段）与
    settings 的三个策略表单已在第三轮处理，快照登记、备份还原、钱包导入、
    运行目录登记已在第四轮处理。**带文本输入的表单族至此全部有失败重渲染。**
- 完整风险分级与证据见 `docs/ARCHITECTURE_AUDIT_2026.md`。
