# neo-os 系统性审计（signer / 密钥托管视角）

## neo-nexus 消费侧复审（2026-09-04）

### Overview

本节最初只审计 `D:\Git\neo-nexus` 对 Rust signer wire API 的消费边界；随后同一轮迭代已
同步更新 `neo-os-services/workers/neo-signer` 的 audience-bound workload v2 验证与跨仓合约。
迁移方向已经成立（本仓没有 `src/signer/` 引擎，repository 不再创建 `signer_*` 表），并进一步
演进为 local wallet、local signer、NeoOS signer service 可并存的显式 profile registry。复审发现
的六个 wire-compatible 缺口均已闭环：

1. `src/signer_client` 和 router 原先缺 `/sign/raw`，`Policy` 也缺 `allow_raw`。现已补齐 route、
   response type 和表单 round-trip，避免保存时把远端已开启的 raw 位静默重置为 `false`。
2. 旧配置直接从 `NEONEXUS_SIGNER_SERVICE_TOKEN` 读取明文，且 URL 可带 reverse-proxy mount，
   与 workload 对 exact route 的签名契约冲突。现以 `NEONEXUS_SIGNER_URL` 为规范 origin；旧 URL
   名仅作弃用别名且双设失败，旧 plaintext token 明确拒绝。生产配置必须且只能选择 protected
   token file 或 caller-id + protected Ed25519 seed file，所有半配置在 listener 启动前失败。
3. 控制台原先只有 bearer admin。现已按 Rust signer 实现 audience-bound
   `neoos-workload-v2` admin assertion：canonical signer origin、timestamp、UUID nonce、uppercase
   method、exact path+query、exact body SHA-256、normalized Origin 与 Ed25519 signature；seed 严格
   限制为不超过 1 KiB 的单行 64-lowercase-hex。服务端默认拒绝无 audience 的 v1，仅保留显式迁移开关。
4. public relay 原先把 JSON 解码后重序列化，也只理解 bearer；这会改变 workload 已签 body bytes。
   现改为有界 raw bytes，并只透传 Authorization、Origin、Referer 与六个 `X-NeoOS-*` workload
   header；入口 path+query 原样成为目标 route，控制台 admin identity 结构上无法注入。
5. Signer 页面仍接收 raw private key、NEP-2 和 passphrase。即使只 relay、不落盘，它仍使
   `neo-nexus` 成为 secret ingress。相关 form、handler、route 和 client method 已删除。
6. ignored real-service probe 只覆盖部分 route，名称“every route”与事实不符；raw、consensus、
   key-info、caller list、workload caller 与语义化 EIP-191 fulfillment 现已纳入 18/18 受支持
   route 的 opt-in contract。

### API Reference and Usage Examples

修订后的 route、环境变量、请求示例和 opt-in 命令见
[`signer-service-design.md`](signer-service-design.md) 的“现行消费者设计”。复审以
`neo-os-services/docs/SIGNER_SERVICE.md` v1 及
`workers/neo-signer/src/{api,auth,service}.rs` 的实际 route、credential parser 与 canonical
`WorkloadMessage` 双重核对，避免只根据旧设计稿推断。

### Design Decisions

- 保留远端 key generation 和全部非 secret 管理操作；删除本地 import form、handler 和 client 方法。
- 完全未配置 signer 仍可启动节点工作台；任何 signer 变量的半配置、冲突或无效值使进程启动失败。
- signer runtime failure 固定返回 `503 signer-service-unavailable`，不读取 legacy 表、不本地签名。
- bearer consumer Origin 只用于控制台 bearer admin；workload admin 是无 Origin 的 server-to-server
  identity。公开 relay 始终传调用者自己的认证头与原始 body，永不借用 admin credential。
- Ed25519 仅认证 NeoNexus 自己的管理 HTTP 请求，不接触 signer 托管的 P-256 key，也不能产出 Neo
  transaction/witness；外部 workload caller 的私钥仍只存在其自身 workload 中。
- 本地钱包 profile 现在有意使用 `aes`、`scrypt`、`p256` 与 `zeroize` 实现官方 NEP-2/Neo N3
  互操作；这些依赖属于受测试的本地 custody backend，而不是遗留 signer 引擎或可删除 churn。

### Test Coverage and Expected Outcomes

- config 单测：canonical/legacy URL 冲突、plaintext token 拒绝、二选一完整 admin profile、native
  origin、bounded single-line credential file、lowercase Ed25519 seed 与合法 Origin。
- client 单测：workload v2 audience/canonical message/signature、raw request/response、`allow_raw` wire field、未知
  field 宽容、重定向拒绝。
- web 集成测：五条 public relay 路由（含 EIP-191）、workload 六头与 exact body/path/query 保真、auth header allowlist、
  body size limit、503 runtime failure、session 边界、无 import route/secret form。
- repository 测试：新库没有 `signer_keys`、`signer_policies`、`signer_callers`、`signer_audit`、
  `signer_value_events`。
- real-service contract：service-owned harness 启动一次性 Rust signer/vault，生成临时 key/caller，
  覆盖所有受支持 v1 route，验证 policy/refusal/audit，再删除对象与 vault；两仓 Linux CI 都运行。

覆盖指标为 18/18（100%）受支持 v1 route；两条 secret import route 因边界要求明确不支持，
不是漏测。

2026-09-04 已在本机用一次性 Rust `signer_service` vault 实际执行该 opt-in contract：1/1
通过，覆盖 Neo N3/NeoX 建钥、策略、caller/workload 管理、raw/EIP-1559/EIP-191 签名、拒绝与
审计；测试结束后临时 caller/key 由测试清理，临时 vault 目录也已删除。

### 需要 v2 服务 API 的剩余阻塞

当前 v1 没有让 `neo-nexus` 安全恢复远端 import 的充分证据面。远端 import 只有在 v2 提供并稳定
attestation、expected vault fingerprint、PCR manifest、state revision/checkpoint 后才能重新讨论。
同样不能在 v1 客户端伪造的还有全局 request-id 去重、durable enclave state 和扩展共识
anti-equivocation 语义；这些必须由 signer 服务拥有。workload assertion 已不是缺口：signer 默认验证
v2 的 canonical audience 与 exact route/body Ed25519 proof，NeoNexus 已为自身 admin identity 精确生成，
并为公开 caller 原样 relay；v1 只在服务端显式兼容开关下临时接受。

---

> 状态：审计报告 · 2026-08-30
> 范围：`D:\Git\neo-os` 多仓工作区（`neo-os-web` / `neo-os-services` / `neo-os-explorer` / `neo-os-app` / `neo-os-contracts` / `neo-os-miniapps` / `neo-os-devpack` / `neo-os-fura` / `neo-express-audit`）以及 `D:\Git\secure-sign-service-rs`。
> 目的：为 signer 服务确定**必须挡住什么**。每条 finding 都对应实现里的一处约束或一条测试。
> **归属（后定于 `neo-os-services/docs/SIGNER_SERVICE.md`）**：signer 服务由 `neo-os-services` 承担，`neo-nexus` 是它的消费者。本报告写作时实现放在 `neo-nexus/src/signer/`，因此"neo-nexus 里的对应实现"应读作**待迁移的代码**——是对的代码，在错的仓里。
> 方法：全部结论来自对磁盘的直接读取（文件 + 行号），可复核。**无法从本地核实的项已明确标注**，未标注的均为已核实。

## 严重度定义

| 级别 | 含义 |
| --- | --- |
| **P1** | 私钥可被滥用于产生有效签名，或边界可被绕过 |
| **P2** | 边界形同虚设：需要越界方配合，但一旦配合即无上限 |
| **P3** | 工程缺陷 / 一致性风险，会放大前两类的发生概率 |

---

## 摘要

**neo-os 缺的不是 enclave，是策略层——而且缺的位置很具体。** 三个签名面里，**只有 JS 那个（`nitro-signer-server.mjs` 的 `POST /sign/payload`）能签任意字节**，它也正是四个消费方实际在调的那个：拿到共享 token 就等于拿到密钥的全部能力，包括把国库转走。Rust 那个（`secure-sign-service-rs`）**结构上没有交易路径**（三个 RPC 全是共识相关），并且共识侧有一套写得相当扎实的防作弊策略——问题在于那套策略除 vsock-consensus 外全部处于关闭状态，且调用者身份为零（详见 F-3）。第三类是 relayer / feed-pusher / explorer 里各自手搓交易与 witness 的脚本，密钥直接从 env 进来。

enclave 侧的密钥保护确实做了（KMS 证明设计、attestation userData 绑定身份），但它保护的是一台**会不会拒绝要看启动参数**的机器。

同时，`neo-os-web` 已经具备一个高质量的密钥材料扫描门（`deploy/scripts/audit_secret_material.mjs`），连"为什么必须有专用 WIF 规则、什么样的 allowlist 理由算合格"都写进了注释。问题是**这套判断只活在那一个仓里**：另两个仓同样有 gitleaks CI，配置却按另一种标准走（见 F-2）。

**这个部署里有两个看上去像边界、其实不是边界的东西**（F-9）："角色"——testnet 四个角色注册的是同一把密钥，而且没填角色会静默落到 `updater`；"共享 runtime token"——同一组 env 凭据既开签名面也开 `apps/web` 的控制面，一次泄露跨两个信任域。

本报告的初稿有 2 处结论经复核后**不成立**，列在文末"复核后撤回"；另有 2 处（F-2、F-3）在核实过程中被推翻重写——F-2 严重度从 P1 降到 P2，F-3 的结论整个换了方向。**两处翻车的原因同一个**：它们都是否定式断言（"没有扫描器"、"不存在策略机制"），而我当时是用关键字 grep 得出"不存在"的，没有去读实现模块本身。修正过程写在各自小节的开头，保留痕迹是因为**错误结论的传播成本比漏报更高**。

---

## F-1 · Nitro signer 对任意字节签名，无策略层（P1）

`neo-os-services/deploy/nitro/nitro-signer-server.mjs:245-266`

```js
function handleSignPayload(payload) {
  const role = normalizeRole(payload.key_role || payload.dstack_key_role || payload.role);
  const dataHex = normalizeHex(payload.data_hex || payload.message_hex || '');
  ...
  const secret = report.materialized.private_key || report.materialized.wif;
  const account = new neoWallet.Account(secret);
  const signature = neoWallet.sign(dataHex, account.privateKey);
```

认证是**有**的（`assertAuthorized`，第 62-83 行：`Bearer` 或 `x-nitro-token`，`timingSafeTokenMatch` 比对，未配置 token 时 `size === 0` 直接 401，fail-closed），这两点都比预期好。问题是认证的**粒度**：

- 一个共享的 runtime token 可以指定**任意 role**——包括部署/国库角色——签**任意长度的 hex 字节**：`handleSignPayload` 对入参的全部校验是"小写 hex 且长度为偶数"（第 248-250 行），既没有长度上限，也没有要求它是 32 字节摘要。没有意图、没有金额、没有资产、没有收付方、没有速率限制。
- `report.materialized.private_key`：密钥明文在 enclave 进程内被实例化成 `neoWallet.Account`。这正是策略必须落在**签名服务内部**的理由——密钥一旦离开密封态，宿主侧（哪怕在 enclave 里）就没有第二次说"不"的机会。
- `attestationUserDataHex()`（第 268-279 行）把 role→public_key/script_hash 绑进 attestation，这个设计值得保留：远端能验证"这个 enclave 持有的是哪几把密钥"。**它缺的是同一份绑定里没有"能签什么"。**

**这条判断在本仓里已有作者本人签字。** `neo-os-services/deploy/feed-pusher/feed-pusher.mjs:350-352` 的注释解释了为什么 feed 更新不再走 `/sign/payload`：

> N3 feed updates are always computed and semantically constructed inside the enclave. The former flag-off path sent arbitrary `getMessageForSigning` bytes to `/sign/payload`, **turning the updater key into a blind-signing oracle**.

也就是说"签任意摘要 = 盲签oracle"这个结论不是外部审计推断，是维护者踩过后写下来的。但**同一条路径上的另外两个调用方照旧**：`workers/morpheus-relayer/src/fulfillment/signer.js:186-194`（`key_role: 'oracle_verifier'`，`data_hex` 由调用方算好）与 `workers/morpheus-relayer/src/neo-n3.js:883-895`（`key_role: 'updater'`，同样传外部给的 `messageHex`）。所以边界的规则应当从这段注释直接推出：**签名请求必须按"意图"提出（交易字节 / 结构化共识载荷），不能按"摘要"提出**——摘要把解析责任推给调用方，服务无从判断，策略层就没有落点。

**neo-nexus 里的对应实现**：`policy::evaluate` 在签名前判定，18 个 `DenyReason` 码；`RequestKind` 把"交易"与"共识载荷"分成两条不可互换的权限；调用者是 `CallerCredentials`（token 摘要 + Origin），不是共享 token。

## F-2 · 同一个形状的私钥，三个仓三套标准（P2；本条在复核中被推翻过一次，以下是磁盘上的实际情况）

原先我写的是"`neo-os-explorer` 与 `neo-os-services` 没有密钥扫描器"。**这句是错的**，两个仓都有 gitleaks CI（`neo-os-services/.github/workflows/gitleaks.yml`、`neo-os-explorer/.github/workflows/ci.yml`）和各自的 `.gitleaks.toml`。真实的缺陷不是"没扫"，而是**三个仓对同一形状的值的收录标准互不兼容，且都不落在 `neo-nexus` 的作用域里**。

| 值（前缀） | 位置 | 该仓如何处理 |
| --- | --- | --- |
| `Kx2BeyUv1dBr99…` | `neo-os-explorer/tests/scripts/lowerThresholdGovernanceLab.spec.js:11`、`mockPolicyGovernanceLab.spec.js:20`、`testnet11of21MultisigLab.spec.js:11` | 该仓**有**专用规则 `neo-wif-private-key`（CRITICAL），但把这个值写进了规则级 allowlist，理由字段是 `"Published deterministic governance-lab test vector"` |
| `KxKbVtgf3iENAick…` | `neo-os-services/workers/morpheus-relayer/src/fulfillment.test.mjs:2523` | 该仓配置只有 `[extend] useDefault = true`，**没有加任何 WIF/NEP-2 规则**；文件里给出的理由是行上注释 `// A valid Neo N3 test WIF … (generated via neon-js, not a live key).` |
| `KwDiBf89QgGbjEhK…` | `neo-os-web/deploy/scripts/lib/secret_material_scan.mjs:121` | 已 allowlist，且理由成立：私钥 `0x…01` 的公开测试向量，派生它不需要任何秘密 |

三处都核实过，逐条说明差异在哪：

**1. explorer 的三处出现是"负向守卫"，不是"夹具里塞了把钥匙"。** 三个 spec 的写法都是

```js
it("requires the council wif from env instead of hardcoding it", () => {
    expect(scriptSource).toContain("TESTNET_COUNCIL_WIF");
    expect(scriptSource).not.toContain("Kx2BeyUv1dBr99…");
});
```

即维护者**已经**把这把硬编码的 council WIF 从脚本里挪到 env，并用测试钉住"不许再写回去"。方向是对的。剩下的问题是这个字面量本身仍留在仓库（含 `.gitleaks.toml:13` 共 4 处），所以它仍然是一个"能用一条 grep 就取回的 52 字符 WIF"，而 allowlist 给的理由属于"这是测试向量"这一类。

**1b. 2026-08-30 复核：这个"测试向量"理由已经可以直接证伪，字面量也已从仓库移除。** 离线推导（neon-js `wallet.Account`）+ 两个互相独立的公共 RPC：

| 观察项 | 结果 |
| --- | --- |
| 派生地址 / pubkey | `NLtL2v28d7TyMEaXcPqtekunkFRksJ7wxu` / `03f35d7b…50c5`（script hash `0x13ef519c…b2a80a`） |
| 官方 N3 测试网（magic `894710606`）余额 | **73,061 NEO**、**117,655.15550668 GAS**；`api.n3index.dev/testnet` 与 `testnet1.neo.coz.io` 两个 provider 读数一致，GAS 两次查询间仍在增长（`lastupdatedblock 18975238`） |
| 治理身份 | `NeoToken.getCommittee` 返回 21 个 pubkey，**含本密钥** → 它是官方测试网 21 人待任委员会成员；`getnextblockvalidators` 的 7 个 active validator 不含它 |
| 主网（magic `860833102`） | **0 NEO、0 原生 GAS**。唯一一条余额是 `0xb249c1c0…` 的 100,000,000 个 "GAS"，`getcontractstate` 显示它是 id 562 的**已部署合约**、NEF `compiler` 字段字面写着 `fake-gas-v1.0`，且 `lastupdatedblock 0`。原生 GAS 是 `0xd2a4cff3…`（id `-6`）。**对照组**：与本条无关的一次性夹具地址 `NPkdkp8…` 在主网上显示**同一条** fake-gas `100,000,000` / `lastupdatedblock 0`，所以这条记录是 RPC 对该合约的通用索引噪声，不携带任何关于密钥的信息。**这个值不值一分钱，别按"主网也有钱"报** |

两点必须说清楚，否则严重度会被我写歪：

- **它能单独做什么：几乎不能。** 委员会操作需要 21 个里的多数签名，单把密钥只占一票权重。它能做的是花掉那批测试网资产，以及为委员会保护的操作提供"所需签名之一"。
- **仍未确定（本地无法定性）：这把密钥的所有权。** 它既可能是本项目自己的 council key，也可能是网络运营方**有意公开**的测试网 authority key（后者的特征完全吻合：委员会成员 + 创世即持有巨量资产）。我用派生地址和 pubkey 各做了一次公开检索，都没有命中任何权威文档，因此**不能判定它是泄露**。能判定的只有一件事：无论来源如何，本仓库用 allowlist + 三处 `.not.toContain` 把它钉在了仓库里，这个动作本身是错的。

**已落地的修正（`neo-os-explorer`，本仓自行完成）**：三个 spec 的负向守卫改为**按 WIF 形状匹配**（`[KL][1-9A-HJ-NP-Za-km-z]{51}`）而不是点名这把密钥——这既让字面量可以离开仓库，又严格更强（挡住任何密钥，而不只挡住这一把）；`.gitleaks.toml` 的 `neo-wif-private-key` 规则级 allowlist **整块删除**，并留注释说明该规则故意不设 allowlist。核实：全仓（排除 `node_modules/.git/dist`）已无该字面量，形状规则命中 0 个文件（用合成控制串验证过 pattern 本身有效，避免把"正则写坏"当成"没有命中"），三个 spec `npx vitest run` 退出 0（15 tests）。**历史里仍在**：该值在 `1159410`，`remotes/origin/main` 含该提交 → 已公开，只能按"公开值"处置。

**2. 而 web 的自家标准明确拒绝这一类理由**（`secret_material_scan.mjs:117-128`）——

> Every entry needs a reason, and the reason has to be "this cannot control anything" — not "this is only a test" or "this is only a doc".

按这条标准，explorer 的 `"Published deterministic governance-lab test vector"` 与 services 的 `"not a live key"` 都不构成收录理由：两者都没有说明**为什么这把密钥控制不了任何东西**。web 自己的第三条（公开规范里的 `0x…01` 向量）才是合格写法。所以真正该报的是：**同一工作区里存在两种互不相容的判断，且没有一处机制迫使它们收敛。**

**3. services 的缺口是配置层面的，不是"人漏看了一次"。** 它没有加 WIF 规则，而 web 的 `audit_secret_material.mjs:15-27` 恰好解释了为什么不能指望通用规则：通用规则要求密钥附近出现 `api_key`/`Bearer`/`aws` 这类关键字，52 字符裸 base58 不提供任何关键字。`VERIFIER_WIF = 'KxKb…'` 里最像关键字的是 `WIF`，它不在 gitleaks 的默认关键字集合里——**但这一点我无法本地核实**（见下）。

**4. 扫描作用域也不一致。** web 扫工作树，理由写在同一份注释里（历史重写覆盖不到 `refs/pull/*` 这类只读 ref，扫描只能保证"不再产生下一个"）。两个 gitleaks job 扫的是**触发事件的提交区间**（`gitleaks git --log-opts="${range}"`）。`neo-os-services` 全仓 9 个提交，`fulfillment.test.mjs`、`.gitleaks.toml` 与那条 WIF 都在同一个提交 `4d2d136`（2026-08-15）里落地，所以区间扫描不会 revisit 它——除非 CI 当时确实跑了那个提交并且没报。**本地看不到 CI 历史。**

**无法从本地核实的（两件）**：

- ~~两把 WIF 对应地址是否为活地址~~ → **已核实**（见 1b 的表格）。`Kx2BeyUv…` 在官方测试网是委员会成员且持有巨量资产；`KxKbVtgf…` → `NPkdkp8SZSWk7gJpCUnoX5QfMRVbM5AdSi` 测试网 `balance: []`，主网只有上表那条对**所有**地址都一样的 fake-gas 噪声记录，没有任何原生 NEO/GAS，所以 services 那句 `not a live key` 的行内注释**这一次是真的**——两处的差别正是"同一形状的判定标准为什么会分叉"的实证。踩过的坑：公共 RPC 走本机注入的 `HTTPS_PROXY` 时直接返回空响应，看起来像"地址无余额"，必须 `curl --noproxy '*'` 才拿得到真实数据。**空响应和零余额不是一回事，别把前者当后者报。**
- ~~gitleaks 8.30.1 的默认规则集是否会命中 `VERIFIER_WIF = 'KxKb…'`~~ → **已实测定性（2026-09-01，本地装官方 8.30.1 二进制）**：只带 `[extend] useDefault = true` 的配置对该行 **0 命中**（关键字洞是真的）；加上两条 Neo 规则后 `neo-wif-private-key` 命中。"必须靠专用规则"从推断升级为实测结论。
- services 的 gitleaks job 在 `4d2d136` 上是否真的执行过并通过。

**已落地（2026-08-30）**：`neo-os-services/.gitleaks.toml` 补上了 `neo-wif-private-key` / `neo-nep2-private-key` 两条规则，并为两处既有命中各写了一条**带证据的** allowlist（relayer 夹具：派生地址在主网/测试网均无原生 NEO/GAS；redaction 夹具：base58check 校验和不过，根本不是密钥）；`neo-os-explorer` 那条谎报的 allowlist 已删除。两份配置 `tomllib` 解析通过。

**已落地（2026-09-01）：收敛机制上线，三个仓不再只是"内容一致的拷贝"。** `neo-os-services/config/gitleaks/neo-rules.toml` 成为两条 Neo 规则的唯一正本（自带 `useDefault`）。8.30.1 实测的三个语义决定了这个形状：`[extend] url` 对 file:// 与 http:// 都**静默失效**（坏链时扫描照样绿——这是最危险的失败模式）；`path` 与 `useDefault` 同设直接加载报错；`path` 相对 **CWD** 解析。所以正本自带 `useDefault`，services 用 `[extend] path` 引用（CI 本就从仓库根运行），默认规则经链式 extend 仍然加载（`aws-access-token` 探针实测命中）。explorer/web 各自保留镜像副本，但 CI 在扫描前先过**子集门**：抓取正本、投影 `^(id|regex|tags|severity) = ` 行、`comm -23` 比对——正本有而本地缺/旧 → 扫描红并列出缺失行；各仓自有规则（web 的 `neoos-password-in-code`）不参与比对。四情形实测：explorer 过、web 过、正则被篡改挂、消费方为空挂。**顺带修出一个真实发现**：第一次全历史扫描显示 services 的 `workers/nitro-enclave-host/tests/unit/provision.rs:75` 仍持有 `MORPHEUS_UPDATER_NEO_N3_WIF = KwDiBf…`（0x01 公知转换向量，与 web 同值同理由），事件区间扫描永远不会 revisit 已引入的行所以 CI 一直没报——已按 web 同款理由补 allowlist。最终三仓扫描全绿：services git 全历史 0、explorer dir（CI 同款命令）0、web git 全历史 0。web 镜像的 tags 顺带从 `key` 对齐为 `private-key` 并补 `severity = "CRITICAL"`。

**仍然成立的部分（收敛之后）**：门只强制**规则块**一致（id/regex/tags/severity 行），allowlist 漂移不在门内——这是有意的：F-2 的教训是 allowlist 的**理由造假**而非规则漂移，豁免必须留在各仓、连同证据一起被评审；删除自己的豁免只会让自己变红（自暴露），不构成跨仓静默漂移。真正的遗留项：六个兄弟仓（admin/devpack/minigames/contracts/miniapps/app）的 gitleaks **完全没有这两条 Neo 规则**，裸 WIF 在那些仓是盲区——把它们接进同一正本+门是下一步。另外 `gitleaks git` 按提交 diff 找"新引入"的行，事件区间扫描天然不会 revisit 历史——全历史扫描（`gitleaks git -v .`）应作为一次性/定期的人工核查手段，provision.rs 就是这么找出来的。

## F-3 · `secure-sign-service-rs`：策略只在共识路径上，且除 vsock-consensus 外全部关闭（P1）

**先撤回**：本条初稿写的是"`max_amount`/`whitelist`/`blacklist` 零命中，因此**不存在任何功能边界机制**"。**这句是错的**——我 grep 的是转账语义的关键字，没有读它的共识模块。磁盘上的实际情况更有意思：

**它确实有策略，而且写得比多数人想象的好。** `secure-sign-core/src/neo/consensus.rs:104-149` 的 `ConsensusSigningPolicy::validate_extensible_payload` 逐项检查：network 必须在允许列表内、`category` 必须是 `dBFT`、`valid_block_start == 0`、消息体不能短于共识头、`valid_block_end` 必须等于头里的 `block_index`、`sender` 必须是 20 字节、签名脚本哈希集合必须**恰好一个**、且该 hash 必须等于 `sender`。违规经 `rpc/src/lib.rs:69-71` 变成 `permission_denied`，在第 85-89 行调用。**这是一个真正的防作弊策略**，不是摆设。

真正的问题有四层，按严重度排：

**1. 除了 `with_vsock_consensus`，所有启动路径都把这套策略关掉。** `startup.rs` 里 `with_tcp`（第 60-66 行）与 `with_vsock`（第 42-48 行）都把 `consensus_network` 设为 `None`，只有 `with_vsock_consensus`（第 51-57 行）给 `Some`；而 `DefaultSignService::new` 对应 `consensus_policy: None`（`rpc/src/lib.rs:55-59`），于是两处 `if let Some(policy)`（第 85、99 行）整体跳过。**非 vsock 部署 = 无策略**，而 TCP 正是 `mock` 子命令与本仓测试用的模式。

**2. `SignBlock` 与 `SignExtensiblePayload` 的检查强度差一个数量级。** 载荷路径做完上面那一整套；块路径只调 `validate_network`（第 99-103 行）就把字节交给签名。同一个策略对象，两个方法：一个防错链/防错类别/防错高度/防错签名者，一个只看链号。

**3. 调用者身份为零——而且结构上不需要它。** `Signer::new`（`core/src/neo/sign.rs:74-87`）把解密出的账户按 script_hash 与压缩公钥各建一张表；`sign_block`（第 110-125 行）**用请求里带来的 public_key 去表里查**，唯一的门禁是"这把钥匙在本钱包里"和"没被 `is_locked` 锁住"。也就是说：能连上套接字的进程，就能指定钱包里任意一把未锁账户去签。没有 token、没有 interceptor、没有 Origin 概念（全仓 grep `interceptor|authorization|api_key` 只命中 clap 的 `author=` 与钱包口令）。

**4. 但它的 RPC 面根本不碰钱。** `servicepb.proto:50-56` 定义的服务恰好只有三个方法：`SignExtensiblePayload`、`SignBlock`、`GetAccountStatus`。**没有"签这批字节/签这笔交易"的方法**。所以 F-1 那个"拿到 token 就能转走国库"的能力，在这个 Rust 服务里结构上不存在——它存在于 `neo-os-services` 那个 JS signer（F-1）里。这条修正很重要，因为它决定边界怎么切：**这个仓缺的不是"给共识加策略"（已经有了），而是"要不要在这里长出一条交易路径"。**

另外两点核实：

- `secure-sign-gateway` 是**正确的形状**：一个前置守护进程，`main.rs:221-224,252-254` 对 public_key 做单值允许列表并返回 `permission_denied`，`:103-119` 用 fsync 的 anti-equivocation journal 记录视图/槽位、冲突时拒绝（`:82`），`:282` 一个 permit 的 semaphore 保证同时只有一笔在签，`:50` 900ms 超时。但它是**部署期常量**而不是**运行期身份**——一把钥匙、一个进程、没有"谁在调"。
- `secure-sign-sgx` 与 `secure-sign-sgx-enclave` **不在工作区内**（根 `Cargo.toml:5-11` 的 members 只列五个 crate；`secure-sign-sgx/Cargo.toml:6` 自己声明 `[workspace]` 以脱离父工作区，依赖 teaclave SDK 的 `sgx_types`）。SGX 这条线目前接不上。
- `mock` 子命令本身仍是真风险源：`mock.rs:48-62` 读真实 NEP-6 钱包、`decrypt_accounts(passphrase)` 解密出明文账户再起服务；名字与真服务对客户端不可区分。`read_passphrase()`（第 40-46 行）接受 `--passphrase` → 口令进 `argv`（Linux 上本地任意用户可读 `/proc/<pid>/cmdline`，`hidepid` 未设时）并进 shell history；省略时才走 `rpassword` 提示。监听面核实过确实收紧：`secure-sign/src/startup.rs:93` 绑 `Ipv4Addr::LOCALHOST`，所以风险主体是**本地与 argv**，不是远程。

**本地构建核实（决定边界怎么落地的硬事实）**：`cargo check -p secure-sign-core` 通过（10.8s，依赖能解析）。但 **`secure-sign-rpc/Cargo.toml:15` 把 `tokio-vsock = "0.7"` 写成无条件依赖**（第 34 行那句 `# vsock = ["tokio-vsock", "hyper-util"]` 正是本该用来做条件编译的注释，被注掉了），而 `vsock 0.5.1` 在 Windows 上编译失败（`VMADDR_CID_ANY` / `vm_sockets_get_local_cid` 找不到，9 个错误）。因此 `cargo check --workspace` 与 `cargo check -p secure-sign --features tcp` 在本机都是 exit 101 —— **这个仓在我这台机器上编不出来，只有 core 能编**。任何往它里面写的策略代码都无法本地验证，这条直接影响下面的边界决策。

**因此边界怎么切**：交易路径 + 调用者身份 + 功能边界属于 **neo-os 的服务层**（见 `neo-os-services/docs/SIGNER_SERVICE.md`），而不是 neo-nexus 这个运维台。`secure-sign-service-rs` 保留它真正独有的东西——NEP-6 密封、Nitro KMS 证明、`StartSignerWithRecipientCiphertext`、共识策略与防作弊 journal——并作为新服务 `vault` 那一层的**后端候选**接进来（它已有把密钥持有情况绑进 attestation 的能力，正好是 `vault.rs` 定义的替换点）。上面第 1、2 点那两处不对称（策略默认关、`SignBlock` 弱检）留在原仓自己修，不在本次边界重构里顺手改。

## F-4 · `Global` witness scope：worker 用 `===`，wallet 用位与，可被组合标志绕过（P2）

同一台链上，两个组件对同一条规则给出不同判定：

- `neo-os-app/NeoOS.App/Pages/LaunchDAppPage.xaml.dAPI.cs:458`（C# 钱包，**正确**）：
  ```csharp
  if (tx.Signers.Any(signer => (signer.Scopes & WitnessScope.Global) != 0))
      throw new DapiException(10002, "Global witness scope is not allowed by NeoOS");
  ```
- `neo-os-services/workers/nitro-worker/src/chain/neo-n3.js:124`（worker，**可绕过**）：
  ```js
  if (!user || user.scopes === tx.WitnessScope.Global) {
    throw new Error('user signer must be present with a non-global witness scope');
  }
  ```

`===` 只挡住"恰好等于 0x80"。`Global | CalledByEntry`（`0x81`）——序列化上完全合法、VM 语义上仍然是 Global——通过 worker 检查，而钱包会拒绝。worker 里同一函数（第 115-120 行）对 sponsor 用的也是两处 `!==` 精确比较，同一类错误方向相反（偏保守，因此不构成绕过，但同样不等于它在 docstring 里说的"must be None or CalledByEntry"）。

同仓的 `getNeoSigners(account, scope = 'CalledByEntry')`（第 139 行）默认值是对的，所以风险集中在**接受外部传入 scopes** 的那条路径。

**neo-nexus 里的对应实现**：`tx.rs` 的 scope 解析按位读取（`Scope::KNOWN` 掩码 + `has()`，`CalledByEntry`/`CustomContracts`/`CustomGroups`/`WitnessRules`/`Global` 各自有意义，不存在"等值比较"这一说），并且第 351 行把 `Global` 与其他位**组合**的情况直接判为 `GlobalWithOtherScope` —— 即 `0x81` 根本到不了策略层；到达策略层的裸 `0x80` 由 `DenyReason::GlobalScopeForbidden` 独立拒绝。两条路径各有一条单元测试：`tests/unit/signer/tx.rs:362`（`0x81` 解析失败）、`tests/unit/signer/policy.rs:459`（`Global` 被拒）。

## F-5 · 网络 magic 有五处独立来源，密钥与链的绑定因此无人保证（P2）

已核实的五类来源：

1. C# 字面量：`neo-os-app/NeoOS.App/Models/Chains/ChainDefinition.cs:32,50`（`860_833_102` / `894_710_606`）、`NeoOS.Tools/Program.cs:790`（`ToolDefaults.Network = 860_833_102`）。
2. 打包进 App 的静态配置：`NeoOS.App/Resources/Raw/protocol.json:3`（`"Network": 860833102`）。
3. JS 字面量 + 兜底：`NeoOS.Codex/onegate/skills/onegate-dapp-debug/assets/nep21-mock.js:224`——`Number.isInteger(providerSettings.network) ? providerSettings.network : 860833102`，**读不到时默认主网**；`scripts/runtime/identity.mjs:15`（`export const ONEGATE_NETWORK = 860833102`）；`assets/profiles/default.json:10-11`。**同形状的第二个实例更要紧**：`neo-os-services/deploy/feed-pusher/feed-pusher.mjs:342`——`const N3_MAGIC = Number(process.env.FEED_MAGIC || 860833102)`，而它服务的正是第 344-345 行那把 updater 密钥（`N3_UPDATER_PUB`）：漏配一个变量，喂价更新就按主网 magic 签。
4. 运行时从 RPC 取：`neo-express-audit/deploy.js:46`（`const magic = version.protocol.network`）；以及 env（`NEO_NETWORK_MAGIC`、`MORPHEUS_NETWORK`）。
5. `1_230_000` 一类私有链默认值散落各处（`config-smoke` 的 launch pack 用 `1230301`）。

magic 进签名预镜像，所以风险不是"签错链就丢币"这么直接，而是两条：

- **私有链重放**：多个互不相干的私有网默认同一个 magic，一次签名在两个网上同样有效。默认值越" convenient "，碰撞越可能。
- **兜底默认主网**（`nep21-mock.js:224`）：一个本该只在 testnet 跑的工具，在配置缺失时按主网签名——失败模式是"验签不通"，运维看到的是谜一样的报错，不是"你少配了一个变量"。

**neo-nexus 里的对应实现**：`SignerKey.network` 在密钥入库时固定，签名时由 `service.rs:584` 经 `SignerKey::network_magic()`（`model.rs:307`）取 `config::network_magic` —— 全仓只有这一张 magic 表（`config/format/network.rs:9`），调用方无法在请求里带 magic。e2e 证明的是**这一条**：`a_boundary_saved_from_the_page_decides_the_next_request` 用独立重建的预镜像验签，并断言同一把 testnet 密钥产生的签名**在 mainnet magic 下验不过**。

**未闭环（自己这条要记下来）**：`config::network_magic` 的 `Network::Private` 分支硬编码 `1_230_000`，而 `service.rs:584` 走的是这条表，**没有走** `effective_network_magic` 支持的 `RuntimeConfigProfile.network_magic` 覆盖。后果两条：私有网 genesis 用了别的 magic 时，托管密钥签出来的预镜像是错的（失败方向安全——验不过，但功能不可用）；两个都吃默认值的私有网之间，一次签名可以互相重放。这正是本条 finding 批评的那个失败模式，只是发生在 `neo-nexus` 自己身上。彻底的做法是把 magic 与密钥行一起固定（`SignerKey` 存 magic 而非只存 `Network`），改动量不小，先记在这里。

**已落地（2026-09-01）：magic 已与密钥行一起固定**——按上一段末尾说的"彻底的做法"做完了，托管侧在 `neo-os-services/workers/neo-signer`：

- `SignerKey` 增加 `network_magic` 字段，入库时在 `store_key` 解析并验证；签名预镜像读行上的字段，`network_magic()` 不再查表。§5.1 三个建钥路由（`/keys`、`/keys/import`、`/keys/import-nep2`）带可选 `network_magic` 请求字段：缺省取规范表值（既有消费者零改动）；mainnet/testnet 只接受本链唯一 magic，其余值以 `admin-request-invalid` 拒绝且不留行——一把绑错 magic 的公网密钥是"签名无节点可验"，要到真金白银押上去才会被发现；private 接受部署真实 magic，只拒绝 0。绑定值随每个 key body 回传，操作员可以核对密钥到底对哪条链承诺。
- 存量库迁移：`migrate_signer_tables` 检测缺列后按 `network` 反推每行**历史上真正签过**的 magic 回填（迁移前签名的预镜像就是查表得来的），单事务完成，崩溃不会留下 0 值行。
- 消费侧：`neo-nexus` 客户端三个建钥方法带可选 `network_magic`（未命名时不上 wire），控制台建钥/导入表单对私有网可填真实值，密钥页展示绑定值。控制台此前那句"magic 不是调用方可选的"已随之改写——对私有网，它恰恰必须是。
- 测试：私有网自定义 magic 端到端预镜像验签（且默认 magic 验不过）、缺省取规范值、公网外来 magic 与 0 拒绝、迁移回填等于历史真值、客户端 wire 上带/不带字段各一。

## F-6 · 泄露记录的处置依赖文档自觉，无自动化（P3）

`neo-os-web/.env.example:121-129` 是对 C-6 那类泄露的正确写法——公开承认泄露、给出地址、点名所有复用了同一材料的变量（`FLAGSHIP_LIVE_WIF`、`AA_TEST_WIF`、`ORACLE_TEST_WIF`）、要求轮换。`audit_secret_material.mjs:15-27` 的说明尤其准确：解释了为什么 `.gitleaks.toml` 丢掉专用 WIF 规则是错的（通用规则要求密钥附近有 `aws`/`api_key`/`Bearer` 这类关键字，52 字符裸 base58 不提供任何关键字），也解释了为什么扫描工作树而不是历史（`refs/pull/*` 这类只读 ref 无法 force-update，历史重写覆盖不到；能撬动的只有"不再产生下一个"）。

问题是这套判断**只存在于 web 一个仓的注释里**。三个仓都有扫描器（见 F-2），但另两个仓的配置恰恰没有应用这套判断：explorer 把一把可用 WIF 以"这是测试向量"为理由 allowlist 掉，services 干脆没有加 WIF 规则——两者都是 `audit_secret_material.mjs:15-27` 那段注释点名过的错误做法。三份 `.env.example` 里键名不同、约束不同（`NEO_TESTNET_WIF`、`MORPHEUS_RELAYER_NEO_N3_WIF`、`PHALA_NEO_N3_WIF`、`NEO_N3_WIF`），也没有任何一份配置声明"哪个仓的哪个文件必须过哪个密钥扫描、按哪条标准判定"。

**建议**：把 `secret_material_scan.mjs` 提到工作区级（或每个子仓的 CI 各调一次），并把 allowlist 的"this cannot control anything"标准写成可执行的检查（校验和有效 + 不在 allowlist ⇒ 必须给出理由字段）。

## F-7 · 生产 KMS-attested 路径尚未启用，明文 env 仍是事实路径（P2）

`neo-os-services/deploy/nitro/KMS-ATTESTATION-DESIGN.md:45-64` 记录了已 provision 的 AWS 资源，并写明：

> **Key policy set:** … `MorpheusNitroRelayerInstanceRole` kms:Decrypt conditioned on `kms:RecipientAttestation:ImageSha384` — currently a **PLACEHOLDER all-zero PCR0** (nothing can decrypt yet).

即 attestation 绑定的解密条件仍是全零占位，谁也无法解密——**因此生产签名仍走 F-1 那条 env 提供 `private_key`/`wif` 的路径**。同一文件第 64 行自己标了红：

> 🔴 The supplied access key is in chat history — **deactivate/rotate it** after this work.

该文件同时把 admin IAM 用户名（`codex-morpheus-deploy`）、AWS 账号 ID、完整 CMK ARN、instance role 名一并提交进仓库。这些不是密钥本身，但它们精确描述了"要拿到密钥需要打哪里"，且与一条已经外泄在聊天历史里的 access key 直接关联。

**无法从本地核实的部分**：该 access key 是否已停用、PCR0 是否已换成真实 EIF 值——需要 AWS 侧核对（`aws iam list-access-keys --user-name codex-morpheus-deploy`，`aws kms get-key-policy`）。这与"Vercel / Cloudflare / Nitro 参数存储里是否还留着历史 WIF"是同一类问题：本地看不到部署环境，只能给出待核对清单。

**neo-nexus 里的对应实现**：`src/signer/vault.rs` 被明确定义成"TEE 实现替换的那一层"——上层只要 32 字节，不关心谁持有。文档写清了本地文件模式的边界（"protects the vault from a database dump, not from anyone who can read the data directory"），不假装它是 enclave。

## F-8 · 明文密钥出现在真实部署路径，而不是只在测试里（P2）

`neo-os-services/workers/morpheus-relayer/src/fulfillment.test.mjs:2523` 之外，更结构性的证据是 `secure-sign-service-rs/mock.rs`（F-3）和 `nitro-signer-server.mjs:252`：两处都在运行时把密钥明文实例化成对象（`wallet.decrypt_accounts(...)` / `report.materialized.private_key`）。这不是"测试不干净"，而是**明文私钥出现在每一次签名的路径上，且没有任何一处代码检查"这把密钥被要求做的事是否在其边界内"**。

**neo-nexus 里的对应实现**：`MasterKey` 是 `Zeroizing<[u8;32]>`、只以借用数组暴露、不实现 `Debug`；`SignerService` 刻意不派生 `Debug`（注释写明：一个只redact 一个字段的 struct print，就是下一次忘记 redact 的那个字段）；`KeyPublicInfo` 取代 `SignerKey` 出响应体，因为 `SignerKey` 携带密封信封——"把它交给序列化器，等于一家托管服务发布自己私钥的密文"。

## F-9 · 角色名与共享 token 都是标签，不是边界（P2）

F-1 讲的是"签名面能签什么"，这一条讲的是"角色分离与实际用的凭据分离了什么"。四处独立事实，合起来说明**按 role 授权、按 runtime token 认证，在当前部署里都不是一道闸**：

**1. 缺失或无法识别的 role 静默落到 `updater`。** `nitro-signer-server.mjs:46-53`：

```js
function normalizeRole(value) {
  const role = trimString(value).toLowerCase();
  if (role === 'oracle_verifier' || role === 'verifier') return 'oracle_verifier';
  ...
  return 'updater';
}
```

`handleSignPayload`（第 246 行）把 `payload.key_role || payload.dstack_key_role || payload.role` 交给它——三个字段都不给时传入 `undefined`，`trimString` 得到 `''`，**不报错、不 400，直接以 updater 密钥签**。也就是说"忘记填 role"这个 bug 的后果不是请求失败，而是拿到一把有部署/国库含义的密钥的签名。同一套 `normalizeRole` 也用于 `handleKeysDerived`（第 232 行）。

**2. testnet 上四个角色注册的是同一把密钥。** `config/signer-identities.json:3-26`——`worker`、`relayer`、`updater`、`oracle_verifier` 四条的 `address` / `script_hash` / `public_key` 完全相同（`NiUs458jFbTH1DA3b9QyeDhMaD282h3iJg` / `0xe421999c…` / `02911ea28aee…`）；mainnet（第 27-49 行）则是四把不同的密钥。所以在 testnet，任何下游"校验这是 worker 的签名"的逻辑，与"这是 updater 的签名"**在密码学上无法区分**——角色标签只存在于请求体和响应体里，不进入任何可验证的绑定。

**3. "密钥必须匹配 registry"是一条 env 开关，不是不变量。** `packages/shared/src/neo-signers-core.js:312-315`：`MORPHEUS_ALLOW_UNPINNED_SIGNERS` 为真时 `pinned = null`，而漂移检查全部写作 `if (... && pinned && ...)`（第 361-379 行）——`pinned` 为空即整体跳过；随后第 382-384 行 `if (!selected && !pinned) selected = primaryValid[0] || fallbackValid[0]`，即**按配置里变量名的先后顺序取第一把能用的密钥，不校验它是谁**。两点让这个开关比看起来更危险：第 313 行读的是 `process.env` 本身，与调用方显式传入的 `env` 快照无关，所以宿主环境里残留的这个变量会影响任何一次解析；而 `scripts/generate-nitro-signer-identity.mjs:54` 会把 `MORPHEUS_ALLOW_UNPINNED_SIGNERS=true` 写进它生成的 env 文件（第 43 行，`.secrets/nitro/generated-<network>-signer.env`，0600）。**后者本身有正当理由**：该脚本铸的是一对全新密钥（第 45-46 行），registry 里当然还没有它们，"关闭 pinning 校验"正是 bootstrap 期的正确行为——问题不在这一处取值，在于**这份文件与运行时 env 文件之间没有任何标记区分**，也没有任何一步在密钥进 registry 后把变量改回 `false`；一旦它被当成长期运行环境复用，"错钥匙照签并报告 `ok`"就成了默认状态。约束目前只有文档一句话（`docs/NITRO_DEPLOYMENT.md:56` "Do not set … for mainnet"）与渲染脚本的默认值 `false`（`scripts/render-nitro-env.mjs:126-127`）。**后果不是签名面接受错钥匙**——错钥匙产出的签名会被按 registry pin 的验证方拒掉——而是**错误在链上/验证侧才暴露，签名服务自己照签并报告 `ok`**，且 `attestationUserDataHex()`（`nitro-signer-server.mjs:268-279`）绑定的是解析后的实际身份，运维看到的一切都是自洽的。

**4. 同一串 token 跨两个信任域。** 签名面的 `runtimeTrustedTokens`（`nitro-signer-server.mjs:14-23`）与 `apps/web/lib/control-plane-auth.ts:40-58` 的 `isAuthorizedRuntimeRequest` 用的是**同一组 env 名**（`MORPHEUS_RUNTIME_TOKEN` / `NITRO_API_TOKEN` / `NITRO_SHARED_SECRET`，后者还多收 `x-morpheus-runtime-token` / `x-nitro-token` / `x-api-key` 三种拼法）。也就是说一把泄露的 bearer 同时拿到"能签"与"能驱动控制面作业/代理到 enclave 的 `/api/runtime/keys/derived`"。同一文件第 5-9 行的注释（`provider_config` 不得授权控制面执行，并点名了内部审计编号 19/20/30/31/37）说明**这套判断在本仓已经存在**，只是 runtime token 这一组仍然无范围——按 `§4.3` 的 caller→key 授予拆开它，对本仓不是新概念，是把已有概念用到还没用的那一组凭据上。

**这条不是"测试网无所谓"**，因为那份 registry 是被生成出去、被别处当作权威值 pin 的：`neo-os-web/deploy/scripts/sync_morpheus_registry.mjs:58,167` 从它生成 `generated-morpheus-signer-registry.ts`，落到 `neo-os-devpack/shared/constants/` 与 `neo-os-miniapps/vendor/neo-miniapp-shared/constants/`。这意味着**将来把 testnet 拆成四把密钥是一次跨仓协调变更**，拖得越久成本越高；边界要求"一角色一密钥一网络"必须写进服务侧规范（见 `neo-os-services/docs/SIGNER_SERVICE.md` §4.3、§6），而不是留给下一个人顺手改。

顺带记录一处同类形状（不作为独立 finding）：`selectAttestationPublicKey`（第 281-293 行）在调用方没给 role 时按 `['oracle_verifier','updater']` 的**固定顺序**挑第一个能解析出公钥的，用来做 attestation 文档的 `--public-key`（第 308 行）。行为可预测，但"哪把密钥代表这份证明"取决于这个顺序而不是调用方意图——与 §4.3 说的问题同构：角色名在代替显式身份做决定。

**无法从本地核实的**：两问。① testnet 那把 `02911ea2…` 是否真的同时承担四种职责（即四种 env 变量名是否都填了同一把 WIF）；② 签名主机上 `MORPHEUS_ALLOW_UNPINNED_SIGNERS` 的实际取值——它决定第 3 点是**已经在生效**还是只是**随时可生效**。两者都要看部署环境（`resolveRole` 的匹配逻辑在 `packages/shared/src/neo-signers-core.js`，取值来自环境），本地看不到（与 F-7 同一类）。

**neo-nexus 里的对应实现**：授权不以角色名为轴，而是 `caller → 允许的 key_id 集合` 的显式授予（`src/signer/auth.rs:360`），Origin 绑定在同一张授予记录上；未识别的调用方 fail-closed（`unknown-token`）。**没有"默认落到某把密钥"的代码路径**：`src/signer/` 全树只有一处读环境变量（`vault.rs:68`，取的是 vault 主密钥，不是签名密钥），签名密钥只能由请求里的 `key_id` 从 vault 解析出来，解析不到只有 `signer-key-unknown` 一条失败路径（`service.rs:434`、`service.rs:659`）——因此也不存在"用一个环境变量把身份校验关掉"的形状。

---

## 复核后撤回（原本记为 finding，实际不成立）

保留这一节，因为**错误结论的传播成本比漏报更高**，而这两条我一度写进了待报告清单。

1. **`relayer.js:817-818` 的"重复 `allowedContracts` 键"** —— `neo-os-explorer/api/relayer.js` 同时写 `allowedContracts` 与 `allowedcontracts`。这不是笔误：本仓两种拼法对应两个消费层（TS/wallet 侧 camelCase，RPC/neonq 侧全小写——`neo-os-fura/deploy/cloudflare-worker/src/adapters.js:328` 正是 `signer?.allowedcontracts ?? signer?.allowed_contracts`），两处同值是**故意覆盖两个读者**，且 `neo-os-explorer/tests/security/MetaTxFlowSource.spec.js:89` 有一条源码级断言守其中一种拼法。真正残留的问题只是"这行没有注释说明为什么两个都要写"，属可读性，不属安全。
2. **"`neo-express-audit` 本地明文私钥"** —— `node1.wallet.json` / `owner.wallet.json` 的 `accounts[].key` 是 **58 字符 `6PY` 前缀，即 NEP-2 口令加密形态**，不是明文 WIF；同目录（含 `build/`）grep `password|passphrase` 零命中，即没有把口令就近存放。唯一可挑剔的是 `lock: false`，那是使用习惯问题，不是"明文落盘"。

还有第 3 处错误（F-2 初稿的"两个仓没有密钥扫描器"）我没有单列在这里，而是写在 F-2 开头：那一处修正后结论仍然成立，只是理由完全换了——留着初稿的说法比删掉更有信息量。

---

## 这份审计如何落到 signer 服务的验收上

下表"已覆盖它的测试"目前活在 `neo-nexus`，随 `neo-os-services/docs/SIGNER_SERVICE.md` §7 的迁移步骤 1 一并搬走——它们是这套实现的行为约束，不属于任何仓。

| finding | 实现约束 | 已覆盖它的测试 |
| --- | --- | --- |
| F-1 | 策略判定先于签名；调用者身份可归因 | `tests/unit/signer/policy.rs`、`auth.rs::identify_names_the_caller_behind_a_refusal` |
| F-2 | 密钥材料不进响应体，也不进日志：签名路径**没有任何 log/print 调用**（已 grep），唯一记录面是审计表，其 `detail` 只带哈希与金额（`model.rs:538`）且只入数据库 | `tests/web.rs::custody_pages_need_a_session_and_the_signing_api_does_not`（断言响应体无 `detail`）、`a_new_caller_token_is_shown_once_and_never_in_a_url` |
| F-3 | 无 argv 密钥路径；缺省全关；一键停用 | `signer_control/tests.rs::a_blank_boundary_saves_as_everything_closed`、`tests/web.rs::a_boundary_saved_from_the_page_decides_the_next_request`（`signer-key-disabled`） |
| F-4 | scope 按位判定；`Global` 与其他位的组合在解析层即拒 | `tests/unit/signer/tx.rs:362`（`0x81` → `GlobalWithOtherScope`）、`tests/unit/signer/policy.rs:459`（裸 `Global` → `GlobalScopeForbidden`） |
| F-5 | magic 随密钥行固定：缺省取规范表值，private 可带部署真实 magic，公网拒外来值；预镜像读行上字段 | `tests/unit/signer/service.rs`（同锁 `a_boundary_saved_from_the_page_decides_the_next_request` 的语义）→ 迁移后为 `workers/neo-signer` 的 `a_private_network_key_binds_the_magic_it_was_created_with`（自定义 magic 端到端预镜像验签、默认 magic 验不过）、`a_key_cannot_be_bound_to_a_magic_that_chain_does_not_use`、store 的迁移回填测试。**五处独立 magic 来源的散布仍是开放项**（本条 1-5 清单，属其他仓库） |
| F-7 | 密钥后端可替换，且诚实描述本地模式边界 | `tests/unit/signer/vault.rs` |
| F-8 | 明文只存在于 `Zeroizing` 借用窗口内 | `tests/unit/signer/service.rs`（独立预镜像验签） |
| F-9 | 授权按 `caller → key_id` 授予，role 名不参与判定；无默认密钥；签名密钥身份不可由环境变量替换或跳过校验；凭据按调用者签发、不复用于其他信任域 | `tests/unit/signer/auth.rs:174`（`identify_names_the_caller_behind_a_refusal`）、`tests/web.rs:1413-1425`（`credentials_are_answered_before_the_bytes_are_parsed`：只授予一把密钥的 caller 请求另一把 → `key-not-granted`） |

## 待用户核对（本地无法验证）

1. F-2 的 `Kx2BeyUv…`：**是否为活钥已本地定性**（官方测试网 21 人待任委员会成员，持有 73,061 NEO / 117,655 GAS；主网无原生资产，只有对所有地址都相同的 fake-gas 噪声记录）。仓库侧也已修完——三处守卫改成按形状匹配、`.gitleaks.toml` 规则级 allowlist 删除，字面量在四个文件里全部消失。剩下**只有一个问题本地答不了：这把密钥是谁的**。两种答案的动作完全不同：
   - 若它是**本项目自己的 council key**：按已公开处置。换掉 lab 用的 council 身份、把那批测试网资产转到新密钥、并把 `TESTNET_COUNCIL_WIF` 指向新材料；历史重写不算控制手段（`1159410` 已在 `origin/main`，只读 ref 仍可达），只有轮换才算。
   - 若它是**网络运营方有意公开的测试网 authority key**（委员会成员 + 创世即持巨量资产，特征完全吻合）：无需轮换，但仓库文档必须改口径——`docs/LOWER_THRESHOLD_GOVERNANCE_LAB.md:6,14` 现在叫它 "one real testnet council signer"，应写成"公开的网络测试网 authority key"，否则下一个读者会按前者判断严重度。
2. F-7：`codex-morpheus-deploy` 的 access key 是否已停用；`EnclaveAttestedDecrypt` 语句里的 PCR0 是否仍是全零占位。
3. F-7 / F-6：`neo-os-web`、`neo-os-services`、`neo-os-explorer` 的部署环境（Vercel / Cloudflare / Nitro 参数存储）中，C-6 泄露过的那把 testnet 私钥是否已彻底不再被引用——`.env.example:128` 点名的 `FLAGSHIP_LIVE_WIF` / `AA_TEST_WIF` / `ORACLE_TEST_WIF` 三个变量当前实际值是否已换新材料。
4. `neo-os-web` 的 git 历史里 `3423e507` 是否仍可达（本地这份克隆 `git rev-list --count HEAD` = 28，且 `git cat-file -t 3423e507` 报 `Not a valid object name`——即这份克隆不含该对象，但据此不能断定远端没有）。
5. F-9：① testnet 部署环境里 `MORPHEUS_ORACLE_VERIFIER_WIF_TESTNET` / `MORPHEUS_UPDATER_NEO_N3_WIF_TESTNET` 等四个角色的变量是否确实填的同一把 WIF；若不同，则是 registry 与运行时不一致（另一类问题，`resolveRole` 会报 `primary keys do not match pinned … identity`）。② 签名主机上 `MORPHEUS_ALLOW_UNPINNED_SIGNERS` 的实际取值：registry 已经把 testnet 四角色 pin 成同一把公钥（`02911ea2…`），但"注册相同"与"实际同一把钥匙"要看部署环境变量才能定；该变量为真则连"是不是这把"都不校验（见 F-9 第 3 点）。
