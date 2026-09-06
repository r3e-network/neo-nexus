# NeoOS Signer 服务设计（neo-nexus 集成）

> 当前状态（2026-09-04）：`neo-nexus` 的 profile 目录可包含三类 signer：本地加密
> wallet、本机部署的 signer、NeoOS signer service。每个节点保存且只保存一个
> `(backend_id, key_id)` 绑定；单个节点不会同时使用三类 signer，也不做故障回退。远端/独立
> NeoOS signer 的密钥托管、策略、
> 调用者认证和审计由 `D:\Git\neo-os\neo-os-services\workers\neo-signer` 所有，HTTP wire 规范是
> `D:\Git\neo-os\neo-os-services\docs\SIGNER_SERVICE.md` 的 v1。本节是现行集成规格；
> 下方“历史引擎设计”只保留为策略背景，不再描述本仓实现。

## 现行消费者设计

### Overview

`neo-nexus` 在进程启动时解析一个命名 profile registry。控制台和公开 relay 各有独立的显式
backend id；节点签名没有进程级默认值，只能使用该节点持久化的 `(backend_id, key_id)`，运行中不跨后端回退：

- `src/signing/` 定义后端类型、显式 capability、`SignerRegistry`、统一签名 dispatch 和
  backend-qualified key reference；
  `src/signing/local_wallet/` 是受限的本地 NEP-6/NEP-2、Neo N3/P-256 实现。
- `src/signer_client/` 只负责 NeoOS service 的 v1 HTTP wire、配置和传输，不保存 Neo 托管密钥；
  其中的 Ed25519 seed 只用于认证管理请求。本机 `secure-sign-service-rs` 使用官方 Neo
  `SecureSign` gRPC 协议，绝不经过这一 HTTP client。
- `src/web/pages/signer.rs` 展示全部已加载 profile 和各自路由角色；本地 wallet 只展示非秘密身份、
  wallet digest、网络与能力；显式 console service 才展示 key、policy、caller 和 audit 管理面。
- `src/web/signer_control.rs` 只发送 NeoOS 服务管理请求；`src/web/signer_api.rs` 的公开 relay
  也只连接 `neo-os-service`，永远不连接本地 wallet 或 consensus-only local signer。
- `neonexus.db` 不创建、不读取、不写入 `signer_*` 表；旧表也不会由升级自动删除。
- 浏览器、signer client、relay 与控制台不接受私钥、WIF、NEP-2 或口令输入。本地 wallet profile
  只从服务器上的受保护 regular file 读取加密 wallet 与单行口令。口令启动后清零；解密私钥以一个
  `Arc<Zeroizing<[u8;32]>>` 保留到 profile 最后一个 clone drop，不做 `mlock`。每次操作重读并校验
  wallet hash/identity；删除口令文件不会锁回已运行 profile。

服务仍提供 v1 import API，但本消费者故意不暴露它们。导入必须在 signer 服务自己的受信任运维面完成；
在 v2 的远程 attestation 验证可用前，控制台不能安全地证明远端接收者就是预期 enclave 和 vault。

### Requirements

1. 推荐设置 `NEONEXUS_SIGNER_PROFILES_FILE`，文档格式见 `docs/signer-profiles.example.toml`。
   registry 可包含任意多个 `local-wallet`、`local-signer`、`neo-os-service` 候选 profile；id 唯一，
   `console_backend` 与 `relay_backend` 必须指向存在且 capability 合适的 profile。每个节点另外保存一个
   backend-qualified key binding，不存在全局默认签名 backend。
   旧 `NEONEXUS_SIGNER_BACKEND` 只是一套 profile 的迁移入口，不能与 registry file 同设。
2. `local-wallet` 必须配置 `NEONEXUS_SIGNER_LOCAL_WALLET_PATH`、
   `NEONEXUS_SIGNER_LOCAL_WALLET_PASSWORD_FILE`、`NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK`
   和非零 `NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK_MAGIC`；mainnet/testnet magic 必须与统一网络表一致；可用
   `NEONEXUS_SIGNER_LOCAL_WALLET_ACCOUNT` 选择账户。它只支持 Neo N3/P-256，transaction 默认开启，
   raw 默认关闭并可显式开启。NeoNexus 自身的 consensus signing API 因缺少持久
   anti-equivocation 而始终拒绝，配置为 `true` 会使启动失败；节点选择该 profile 时则把同一个
   已校验 wallet 交给 neo-cli/neo-go 的原生 wallet 共识路径。不支持 NeoX、
   远程管理、durable audit 或 public relay。启动时交叉验证地址、公钥、verification script 并固定
   wallet SHA-256；每次签名前重新读取并拒绝被替换的文件。transaction 必须完整解析为 canonical
   Neo N3 unsigned envelope，且本账户必须在 signer 列表中；request id 冲突由有界进程内 ledger 拒绝。
3. `local-signer` 必须配置官方 SignClient 可消费的 `endpoint + public_key + network_magic`，
   endpoint 仅接受字面量 loopback `http(s)://IP:port` 或 `vsock://cid:port`，且只能用于
   `neo-cli + Consensus`。它没有 HTTP admin、policy、caller、audit、transaction/raw 或 public relay。
   `neo-os-service` 使用强制 HTTPS 的 HTTP origin；若要供原生节点使用，还要同时配置独立的
   node-facing gRPC bridge `endpoint + public_key + network_magic`，三字段缺一即拒绝。
4. 每个 NeoOS HTTP service profile 必须且只能选择一个 admin identity；若被节点绑定用于签名，还必须配置另一套
   仅持 `sign` 与指定 key grant 的 signing identity。admin credential 绝不进入 sign route：
   `NEONEXUS_SIGNER_ADMIN_TOKEN_FILE`，或
   `NEONEXUS_SIGNER_ADMIN_CALLER_ID` + `NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE`
   （可选 `NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT`）。token/seed 文件有大小、单行和
   regular-file 限制；Unix 拒绝 group/other 权限，Windows 检查 opened-handle owner/DACL 并拒绝
   reparse point、宽泛身份及无法安全解释的 ACE。workload seed 必须恰好为 64 个小写
   hex 字符（32-byte Ed25519 seed）。workload 请求固定使用 audience-bound
`neoos-workload-v2`，其 audience 从已经规范化且无 path 的 `NEONEXUS_SIGNER_URL`
origin 推导，并必须与服务端 `SIGNER_SERVICE_WORKLOAD_AUDIENCE` 完全一致。subject
只进入签名字节，不作为可篡改 header 发送；服务端从已登记 caller 记录恢复同一 subject。
`NEONEXUS_SIGNER_SERVICE_TOKEN` 明确拒绝，不再读取。
5. `NEONEXUS_SIGNER_SERVICE_ORIGIN` 仅对 bearer admin profile 可选。设置后，它作为控制台
   admin caller 的精确 `Origin` 发送；workload admin 不发送 Origin。程序调用者经公开 relay
   提供的 `Origin`/`Referer` 仍原样转发，绝不替换为控制台 Origin。
6. 所有 signer 配置都未设置时，工作台可在“未配置 signer”状态启动；任一半配置、跨 kind 字段、
   route capability 错误、URL/Origin 无效或 timeout 无效时，进程启动失败。为迁移保留一版兼容：
   未设置 selector 的旧 HTTPS service 配置仍按 NeoOS service 读取；cleartext 不再兼容。
7. signer 服务不可达、返回非 v1 JSON 或后台调用失败时，不尝试本地 wallet、不读取旧表，也不返回伪造结果；
   relay 以 `503 signer-service-unavailable` 关闭。
8. v1 拒绝是正常结果：保留服务给出的 HTTP 状态、`code` 和 `message`，不把拒绝转换成 transport error。
9. 配置页必须 round-trip 当前 v1 策略全部字段，包括 `allow_raw`。漏读该字段后再保存会把远端已开启的
   raw lane 静默关闭，属于控制面破坏。
10. 任何 ID 进入路径前都限制为有限长度的 ASCII 字母、数字、`-`、`_`；客户端不跟随重定向，避免把
   bearer credential 发送到第二个 endpoint。

### 节点运行时三选一

节点绑定是互斥选择，不是优先级列表：

| 节点 signer | 原生消费方式 | 当前支持范围 |
|---|---|---|
| `local-wallet` | 节点自身解锁已绑定的 NEP-6 wallet | `neo-cli` 的 Consensus/Oracle/StateValidator；`neo-go` 的原生 signing duties |
| `local-signer` | neo-cli `SignClient` → 本机 SecureSign gRPC | 仅 `neo-cli + Consensus` |
| `neo-os-service` | neo-cli `SignClient` → 本机纯 Rust bridge → NeoOS HTTPS signer | 仅 `neo-cli + Consensus`，且 profile 必须声明 bridge |

`neo-rs` 当前只接受明文 `private_key_hex`，NeoNexus 不写该字段；NeoX 客户端使用不同的链与密钥
语义，因此这些组合一律在启动前失败。远程 SignClient 模式还要求完整的每节点 Neo CLI runtime：
`neo-cli` 二进制、`DBFTPlugin.dll`、`SignClient.dll` 与
`NeoNexus.SignerBootstrap.dll` 必须位于同一节点工作目录树。Neo 的插件根
来自 `AppContext.BaseDirectory`，不是 process working directory；共享一个外部 binary 却把
`Plugins/*/*.json` 写入节点目录会被明确拒绝。

本地 wallet consensus 由 DBFT 的 wallet `AutoStart` 启动。两个 SignClient backend 均使用
`neo-cli --background`，并由独立仓库 `D:\Git\neo-nexus-signclient-bootstrap` 中的版本锁定
.NET 边界适配器启动共识；NeoNexus 与 NeoOS 主体仍为 Rust。适配器等待 block import 与 P2P
监听完成，要求唯一 DBFT、唯一 SignClient、无其他 signer plugin、无已解锁 wallet，并验证
DBFT/SignClient sidecar、endpoint、network magic 与绑定公钥；随后直接调用公开的
`DBFTPlugin.Start(ISigner)`，不调用具有 fallback 语义的 `GetSignerOrDefault`。任一校验或超时失败
都会让节点以非零状态退出。当前适配器只支持 official `neo-node` 3.9.2（Neo core 3.9.1 / .NET 10），
节点声明其他版本时启动前即失败。

### API Reference

NeoOS 服务后端的公开 relay（不走 `neo-nexus` session；由 signer bearer 或 workload assertion 认证）：

- `POST /signer/api/v1/sign/transaction` → v1 `POST /sign/transaction`
- `POST /signer/api/v1/sign/consensus` → v1 `POST /sign/consensus`
- `POST /signer/api/v1/sign/eip191-fulfillment` → v1 `POST /sign/eip191-fulfillment`
- `POST /signer/api/v1/sign/raw` → v1 `POST /sign/raw`
- `GET /signer/api/v1/keys/{id}` → v1 `GET /keys/{id}` public key identity read

`local-wallet` 没有 public HTTP signing route，只作为受信任的进程内能力存在；当前浏览器页面只观察
其非秘密身份与 capability，不提供“点击签名”控制。

session 保护的控制台调用远端 v1 管理面：

- `POST /keys`（支持 `chain_family=neo-n3|neox` 与可选 `chain_id`）、`GET /keys`、
  `POST /keys/{id}/state`、`DELETE /keys/{id}`；
- `GET /keys/{id}/policy`、`POST /keys/{id}/policy`；
- `POST /callers`、`POST /callers/workload`、`GET /callers`、`POST /callers/{id}/rotate`、
  `POST /callers/{id}/state`、`DELETE /callers/{id}`；
- `GET /audit`，可按 key 过滤；
- **不**代理 `POST /keys/import` 或 `POST /keys/import-nep2`：WIF/raw key、NEP-2 与
  passphrase 只允许在 signer 的受信 operator boundary 直连输入，NeoNexus 永不成为 secret ingress。

`Policy` 完整镜像 signer 当前 wire：五个 `allow_*` 开关，contract/asset/recipient allow/deny
列表，contract-method 列表，per-asset limits，`max_single_amount`、`window_limit`、
`max_signers`、system/network fee ceiling、signature rate limit，以及 `chain_family`、
EVM gas price/limit、method allow/deny 与 `evm_chain_id`。金额使用十进制字符串，避免
`i128`/`u128` 在 JSON number 客户端中丢精度；未知的新增字段在 read/edit/write 中原样保留，
避免新 signer 字段被旧 console 静默擦除。

交易/共识请求可携带 `request_id`、`chain_family`、`chain_id`。Neo N3 旧调用仍可只发送
`key_id + unsigned_hex`；NeoX 请求必须显式携带 `chain_id`。当前 signer 成功响应保留
`chain_family`、`signed_transaction` 与兼容别名 `signature_hex`，Neo witness 两字段为空字符串；
NeoNexus 也保留 signer 后续增加的响应字段，不把可选的新字段变成锁步部署要求。

workload caller 管理请求只携带 32-byte Ed25519 公钥、可选 subject、grant/capability/origin，
没有 private key 或 bearer token。Signer 返回的 `auth_mode`、`workload_public_key`、
`workload_subject` 及未知 caller metadata 均由 NeoNexus 保真。公开 relay 原样转发 workload 自己
生成的六个 assertion header 和原始 body；NeoNexus 不持有这些调用者的私钥。控制台自身可独立选择
文件化 workload admin profile；该 seed 只认证管理 HTTP 请求，不能签 Neo 交易。

`POST /sign/raw` 请求为：

```json
{"key_id":"key-…","data_hex":"<non-empty hex>"}
```

成功响应除通用 witness 字段外还包含 `signature`（64-byte `r||s` hex）和
`public_key`（33-byte compressed P-256 key hex）。是否允许 raw signing 只由远端
`allow_raw` 策略决定，默认关闭。

`POST /sign/eip191-fulfillment` 只接受结构化 NeoX verifier 语义：

```json
{
  "key_id": "key-…",
  "request_id": "relayer:neox:7",
  "chain_id": 12227332,
  "oracle_contract": "0x…",
  "fulfillment": {
    "request_id": "7",
    "app_id": "app:1",
    "module_id": "oracle.fetch",
    "operation": "privacy_oracle",
    "success": true,
    "error": ""
  },
  "result_bytes_hex": "0x…"
}
```

该请求没有 caller-supplied digest/prehash。Signer 从这些字段派生 contract-bound digest 与
EIP-191 message hash，并以 key 的 NeoX chain、oracle contract、`fulfillRequest` selector policy
做决策；NeoNexus 只保真转发结构化请求和 signer 返回的 65-byte signature。

### Usage Examples

推荐使用 registry 声明三类候选 backend，并指定 console 与 relay 控制面路由；节点在工作区内各自三选一：

```dotenv
NEONEXUS_SIGNER_PROFILES_FILE=/etc/neo-nexus/signers.toml
```

完整的三后端配置及独立 admin/signing identity 示例见
`docs/signer-profiles.example.toml`。以下环境变量示例只是一次加载一个 profile 的迁移兼容入口，
不能与 registry file 同时设置。

```dotenv
NEONEXUS_SIGNER_BACKEND=local-wallet
NEONEXUS_SIGNER_LOCAL_WALLET_PATH=/var/lib/neonexus/validator.wallet.json
NEONEXUS_SIGNER_LOCAL_WALLET_PASSWORD_FILE=/run/secrets/neonexus-wallet-password
NEONEXUS_SIGNER_LOCAL_WALLET_ACCOUNT=NNxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK=mainnet
NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK_MAGIC=860833102
# transaction 默认开启；consensus 必须保持 false；raw 仅限受信任场景显式开启：
NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_TRANSACTION=true
NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_CONSENSUS=false
NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_RAW=false
```

本机部署 signer 使用官方 Neo SecureSign gRPC，并固定 endpoint、公钥和网络 magic：

```dotenv
NEONEXUS_SIGNER_BACKEND=local-signer
NEONEXUS_LOCAL_SIGNER_ENDPOINT=http://127.0.0.1:9991
NEONEXUS_LOCAL_SIGNER_PUBLIC_KEY=031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e
NEONEXUS_LOCAL_SIGNER_NETWORK_MAGIC=860833102
```

NeoOS signer service 使用 HTTPS：

```dotenv
NEONEXUS_SIGNER_BACKEND=neo-os-service
NEONEXUS_SIGNER_URL=https://custody.internal.example
NEONEXUS_SIGNER_ADMIN_TOKEN_FILE=/run/secrets/neo-nexus-signer-admin-token
# 仅当该 admin caller 在服务端绑定了 browser Origin 时设置：
NEONEXUS_SIGNER_SERVICE_ORIGIN=https://nexus.internal.example
NEONEXUS_SIGNER_SERVICE_TIMEOUT_SECONDS=10
```

或者使用控制台自己的 Ed25519 workload identity（二选一，不与 token file 同设）：

```dotenv
NEONEXUS_SIGNER_BACKEND=neo-os-service
NEONEXUS_SIGNER_URL=https://custody.internal.example
NEONEXUS_SIGNER_ADMIN_CALLER_ID=neo-nexus-admin
NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE=/run/secrets/neo-nexus-admin-ed25519-seed
NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT=neo-nexus-production
```

程序调用者通过服务后端的 `neo-nexus` relay 请求交易签名：

```http
POST /signer/api/v1/sign/transaction
Authorization: Bearer <sign-only-caller-token>
Origin: https://payments.example
Content-Type: application/json

{"key_id":"key-…","unsigned_hex":"<SerializeUnsigned hex>","request_id":"request-…","chain_family":"neo-n3"}
```

workload caller 改用 `X-NeoOS-Workload-Protocol: neoos-workload-v2`、
`X-NeoOS-Audience`、`X-NeoOS-Caller`、`X-NeoOS-Timestamp`、`X-NeoOS-Nonce`、
`X-NeoOS-Signature` 六头。audience 是 signer 的 canonical origin，禁止 user-info、path、query、fragment；
因此代理不能借 path rewrite 让同一 proof 指向另一个服务。relay 只透传
Authorization/Origin/Referer 与这六头，保持 body 原始字节及入口 path+query；不注入控制台 admin
identity，因此 signer 对 audience/exact route/body SHA-256 的验证不漂移。一个 audience 只能代表一个
拥有统一 nonce/idempotency 数据库的逻辑 custody domain；独立 vault/独立 nonce store 必须使用不同
origin。Nitro custody EIF 在构建时把 caller-visible origin 写入
`SIGNER_SERVICE_WORKLOAD_AUDIENCE`（默认 host bridge `http://127.0.0.1:8789`）；
NeoNexus 的 `NEONEXUS_SIGNER_URL` 必须使用同一规范化 origin。

NeoX 调用使用同一路由，并显式命名不可变链身份：

```json
{"key_id":"key-…","unsigned_hex":"<EIP-1559 unsigned tx hex>","request_id":"request-…","chain_family":"neox","chain_id":47763}
```

运行真实兼容性契约（harness 会自动探测 sibling `neo-os-services` 或
`../neo-os/neo-os-services`，也可通过 `NEO_OS_SERVICES_DIR` 指定；它会自行构建、启动并清理
一次性 signer/vault，不接触共享开发 vault）：

```bash
make signer-compat
```

### Design Decisions

- 使用 blocking `ureq` 并统一包在 `spawn_blocking` 中，避免 enclave/network latency 占住 axum worker。
- backend 类型必须显式；registry 是候选 profile 目录，`console_backend` 与 `relay_backend` 只负责
  控制面。每个节点持久化一个 `(backend_id, key_id)`，所有签名操作固定使用该绑定。“本机/远程”
  不会从 URL 猜测，backend 失败也不会跨 profile 回退。
- 本地 wallet 的应用 API 只提供 Neo N3/P-256 transaction（raw 显式 opt-in，consensus API
  始终拒绝）；节点运行时可把同一已校验 NEP-6 wallet 作为该节点唯一 native signer。口令仅在
  启动配置生成时短暂读取，但 Neo 原生客户端会要求它以明文存在于权限受限配置文件中。
- `local-signer` 是 consensus-only SecureSign gRPC；`neo-os-service` 的原生节点路径是独立
  gRPC bridge。两者都通过 SignClient，不共享 NeoOS HTTP admin/relay 语义。
- service profile 的 admin credential、内部 signing credential 与 relay caller credential 相互分离；
  admin identity 不进入签名路由，内部 signing identity 只持 `sign` 和所需 key grant，relay 则保真转发
  外部 caller 自己的认证材料。
- 不实现 import client 方法或 HTML secret field。v1 import route 的存在不等于每个消费者都应成为密钥入口。
- 允许完全不配置 signer 以保留 Neo 节点运维功能，但任何半配置或跨 family 配置都使启动失败；
  这是 optional subsystem，不是 permissive signer fallback。

### Test Coverage

自动化验收覆盖：

- 三类 backend profile 目录、每节点 backend-qualified key identity、显式 console/relay 控制面路由、
  跨 family 字段冲突、无 fallback，以及单 profile 兼容 selector；
- `local-signer` 的 loopback/vsock、固定公钥与 magic 限制，及 `neo-os-service` HTTP/bridge 分离；
- 真实 NEP-2 fixture 的解密、Neo N3/P-256 签名与独立验签，wallet hash 替换拒绝、错误口令/账户拒绝，
  canonical unsigned transaction 全量解析、request-id 冲突拒绝，以及 transaction 开启、consensus
  始终拒绝、raw 默认关闭的能力；
- canonical URL/deprecated alias 冲突、plaintext token 拒绝、二选一文件化 admin profile、timeout、
  可选 consumer Origin 的解析与精确发送；
- 所有受管子进程与 runtime smoke 子进程在 spawn 前清除 signer 控制面环境变量；
- public relay 的 transaction、consensus、EIP-191、raw、key-info 路由，以及 caller bearer/workload 六头、
  Origin/Referer、exact path+query 与原始 body bytes 保真；
- chain-bound NeoX key generate/key-info 与 transaction 请求/响应，旧 Neo N3 body 保持兼容；
- 当前完整 Policy（包括 EVM 字段）的读取、表单 round-trip、未知字段保留和默认关闭；
- caller token 新建/轮换响应使用 `Cache-Control: no-store`；
- workload caller 公共身份可经控制台注册，caller identity 与新增 metadata 不被旧 client 擦除；
- 未配置 signer、transport failure、非法服务响应均 fail closed，且无跨 backend fallback；
- Signer 页面和路由不含 private-key/NEP-2/passphrase 输入或 import endpoint；
- 新建 workspace 不创建五个历史 `signer_*` 表；
- stub transport 单测覆盖全部受支持 v1 方法；
- opt-in 测试对真实 Rust signer 执行每个受支持 route，并验证拒绝状态、policy round-trip、caller
  credential rotation、audit 和清理；即使断言 panic 或提前返回，也以 best-effort guard 清理新建
  caller/key。

文档覆盖指标：本消费者支持的 v1 route 为 18 条，以上 API Reference 与 real-service contract
覆盖 18/18（100%）。服务的两条 secret import route 明确列为“不支持”，不以未覆盖 route 隐藏。

当前 v1 无法完成的 v2 验收项：远端 import 前的 Nitro attestation/vault fingerprint/PCR 验证、
由 NeoNexus 代外部 caller 生成 assertion（当前刻意不持有其 workload 私钥）、durable checkpoint/state
revision 和 anti-equivocation 扩展。
这些不能由客户端猜测或以本地副本补齐；本地 wallet 的 consensus lane 因此始终拒绝，而不是一个可由
配置开启的降级能力。

---

## 历史引擎设计（仅背景，2026-08-30）

> 以下内容描述最初在 `neo-nexus` 中规划的托管引擎。代码所有权已经迁到
> `neo-os-services/workers/neo-signer`；出现的 `src/signer/*` 路径和本地 SQLite 设计不再是本仓目标。

---

## 1. 目标与非目标

**目标**
- 托管 Neo N3 私钥：加密落盘，运行时按需解密进签名路径，绝不以明文持久化、绝不出现在日志/响应体。
- 按**功能边界**（策略）受控签名：操作类型、合约白/黑名单、转账收付方白/黑名单、资产白/黑名单、单笔上限、单位时间累计上限、共识签名开关等。
- **调用者认证**：使用者（dAPI 客户端、其他服务）必须携带身份令牌，且其来源（URL/Origin）可被绑定到白名单，才允许请求某个密钥的签名。
- 全程审计：每次签名请求（允许或拒绝）落审计日志。

**非目标（本期不做，留接口）**
- 多签/阈值签名、HSM 硬件、远程多方密钥派生。
- 直接构建并广播整笔交易（Signer 只对**已构建好的未签名交易**产出见证；交易由调用方或上游 dAPI 组装）。
- 自动余额校验（策略校验金额上限，不替用户查链上余额；可选增强）。

**关于"托管到 TEE"**：`neo-nexus` 本身是运维工具、运行在宿主机，**不是** enclave。设计上把"主密钥派生 + 签名执行"隔离成可替换边界，使得同一份代码可以：本地运行（主密钥由操作员凭据派生的信封加密）、或在 Nitro TEE 内运行（主密钥密封到 enclave 证明，`seal key` 由 attestation 绑定）。仓库已有 Nitro 工具链（`neo-os-services` 的 nsm-attest）。文档与代码把这条路径预留为"密钥后端（key backend）"抽象，而不是硬编码。

---

## 2. 领域模型

### 2.1 `SignerKey`（托管密钥）
| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | UUID | 主键 |
| `label` | String | 人类可读名 |
| `script_hash` | `[u8;20]` | 账户的 Hash160（Neo 地址） |
| `public_key` | `[u8;33]` | 压缩公钥 |
| `ciphertext` | `Vec<u8>` | 加密的私钥（见 §3） |
| `nonce` | `Vec<u8>` | AEAD nonce |
| `created_at` | ts | |
| `disabled` | bool | 运维一键禁用 |

私钥本身永不落盘、不入日志；只有 `ciphertext` 落 SQLite。

### 2.2 `SignerPolicy`（策略 / 功能边界）
默认**拒绝**（fail-closed）：三个开关初始为关，空列表的含义是"不设限"而不是"全不允许"。实现见 `src/signer/model.rs`：

```
allow_consensus     : bool        // 是否允许共识（dBFT ExtensiblePayload）签名
allow_transfer      : bool        // 是否允许 NEP-17 形态的 transfer
allow_contract_call : bool        // 是否允许其余合约调用
allow_global_scope  : bool        // 是否允许本密钥出具 Global 范围见证（默认关）
contract_whitelist  : [ScriptHash]
contract_blacklist  : [ScriptHash]   // 命中即拒，优先级高于白名单
asset_whitelist     : [ScriptHash]   // 只对 transfer 生效
asset_blacklist     : [ScriptHash]
transfer_to_whitelist / transfer_to_blacklist : [ScriptHash]
max_single_amount   : Option<i128>          // 单笔上限（token 原始最小单位）
window_limit        : Option<WindowLimit>   // { seconds, max_amount }
```

- `ScriptHash` 是只保存 **wire 序**字节的 newtype；`Display`/`FromStr` 一律用运维粘贴的 **display 序**（`0x…`，与 RPC 一致）。字节序写错时策略列表会"永远不命中"，看起来像"没配白名单"，所以类型上不给选错的机会。
- `window_limit` 用**一个结构**同时持有窗口长度与上限：两个独立 Option 允许"有上限没窗口"这种静默失效的配置。窗口额度**按 token 分别累计**（NEO 0 位小数、GAS 8 位、他币另算，跨币相加的数字没有意义）。
- `SignerPolicy::problems()` 返回"看起来有边界其实不生效"的告警（例如开了 `allow_transfer` 却没配 `asset_whitelist`、或设了金额上限却没开 `allow_transfer`），页面显示而不是拒保存。
- **转账识别按脚本形状**（`transfer(from:UInt160,to:UInt160,amount:Int,_)`）而非按合约地址：否则非原生代币（nGAS/USDT 等）的 `transfer` 只会是"合约调用"，运维写的金额上限对它**完全不生效**。把长得像的误判成转账只会多要权限（transfer 门 + 资产门 + 单笔/窗口上限 + `from` 必须等于本密钥），不会少要。

### 2.3 `SignerCaller`（调用者 / 使用者）
| 字段 | 说明 |
|---|---|
| `id` | UUID |
| `label` | 名字（如 "OneGate dAPI"） |
| `token_digest` | 该调用者的令牌只存 SHA-256 |
| `allowed_key_ids` | 该调用者能请求的密钥集合 |
| `allowed_origins` | 使用者 URL/Origin 白名单（可空=仅服务端对服务端） |
| `disabled` | |

**双层认证**：①令牌（身份）②来源匹配（`Origin`/`Referer` 需命中该调用者的 `allowed_origins`，若调用者配置了的话）。服务端对服务端（无浏览器）可只配令牌。

### 2.4 `SignerAudit`（审计）
每次 `/sign` 请求落一条：时间、调用者、密钥、交易摘要、提取出的操作、允许/拒绝、拒绝原因。复用 `Repository` 的事件记录模式（`repository.record_event` / `RuntimeEventFilter`）。

---

## 3. 密钥托管与密码学

### 3.1 存储加密（信封加密）
- 主密钥 `MK`：由**密钥后端**提供。本地后端 = `HKDF-SHA256(operator_token || data_dir_salt)`；TEE 后端 = 由 enclave seal key 派生（接口化，本期实现本地后端）。
- 私钥加密：`AES-256-GCM(MK)`。每密钥独立随机 `nonce`。
- 也支持**导入 NEP-2 加密私钥**（`6P…`）：历史草案曾误写成 AES-CBC；NEP-2 实际使用
  `scrypt + AES-256-ECB`。导入路径会在内存解密后再以 AES-GCM 信封加密入库，并清零临时明文。

历史草案对应的依赖应为：`p256`（含 `ecdsa` 特性，RFC6979 确定性 ECDSA + low-S）、
`aes`（NEP-2 AES-256-ECB）、`scrypt`（NEP-2 KDF）、`hkdf`、`aes-gcm`、`zeroize`；不需要
把 CBC 当作 NEP-2 模式。

### 3.2 Neo N3 签名原语（`src/signer/crypto.rs`，已按主网实测数据校准）
- 曲线是 **secp256r1(P-256)**，ECDSA + RFC6979 确定性 k + 低 S，输出 64 字节 `r‖s`。
- 签名输入（`sign_data`）= **网络 magic 的 little-endian 4 字节 ‖ SHA256(SerializeUnsigned 字节)**，共 36 字节；**不是**未签名交易字节本身，也不是裸 `SHA256`。主网 magic 860833102 → `4e454f33`（"NEO3"），私网 1230000。magic 一律取 `src/config/format/network.rs`，signer 内不再各自硬编码（这曾是仓库里四处不一致的常量）。
- 单签验证脚本 = `0c21 <33B 压缩公钥> 41 56e7b327`（40 字节，`System.Crypto.CheckSig` 的 transition hash 是 `SHA256(name)[0..4]` 的小端前 4 字节）；调用脚本 = `0c40 <64B 签名>`（66 字节）。旧文档写的 `21 <pubkey> ac` 是 N2 形态；`src/wallet/crypto/keys.rs` 的 `extract_single_sig_contract_public_key` 已改为只认 N3 形态（N2 脚本一律返回 `None`），验证器钱包 fixture 与 CI smoke 同步换成了 N3 脚本 + 派生地址。
- `script_hash = HASH160(验证脚本)`，地址 = version `0x35` + hash160 的 base58check（主网以 `N` 开头）。
- **字节序陷阱**：RPC/浏览器显示的 hash（`0xef4073a0…`）与线序字节（`f563ea40…`）互为逆序。策略、脚本、hash160 内部一律线序，只有输入输出走 display 序，由 `signer::model::ScriptHash` 强制。
- 上述形状不是从本模块自证来的：`tests/unit/signer/crypto.rs` 用 4 笔已挖掘的主网交易（公钥/脚本 hash/地址/签名/`sign_data` 全部取自链上）钉住，另有"错误 preimage 必须被链上签名拒绝"的反向用例。

### 3.3 交易解析（只读、用于策略判定）
`src/signer/tx.rs` 复刻节点的 `SerializeUnsigned` 格式，且**任何解释不了的东西都是错误**（解析失败=不签名），不存在"尽力解析后当未知合约放行"：
- `signers[]`（`account`、`scope`、`allowed_contracts`、`allowed_groups`）——策略据此判断本密钥是否在签名者里、见证范围是否可静态约束。带 `WitnessRules(0x40)` 的直接拒（其有效性取决于链上状态，我们看不见）；保留位、`Global` 与其他位混用一律拒。
- `attributes[]`——只认 `HighPriority(0x01)`、`NotValidBefore(0x20)`、`Conflicts(0x21)`；其余（含 `OracleResponse`、`NotaryAssisted`）一律拒签，见 §11。重复类型按节点的 `AllowMultiple` 规则处理（只有 `Conflicts` 可重复），signers+attributes 合计受 `MaxTransactionAttributes=16` 约束（节点就是用它给两者设限的）。
- `script`——只接受 `PUSH*` / `PACK` / `System.Contract.Call` 组成的脚本。出现跳转、其他 syscall、运行期算出的 hash 都拒：那种脚本的结果无法预先描述，签它等于相信调用方的自述。
- 解出的每个 call 折叠成 `Intent`：NEP-17 形状的 `transfer` → `Transfer{asset,from,to,amount}`，其余 → `ContractCall{contract,method,flags}`。
- 同一模块还解析 `ExtensiblePayload`（共识请求）并暴露 `display_to_script_hash` / `script_hash_to_display` 供策略配置与展示使用。

---

## 4. 策略引擎（核心）

`policy::evaluate(policy, key_account, request, spent) -> Verdict`（`src/signer/policy.rs`）。纯函数：不碰数据库、不碰密钥、不碰网络，因此判定面可被单测穷举。`request` 是 `SigningRequest::Transaction{transaction,intents}` 或 `SigningRequest::Consensus{payload}`；`spent` 是服务从审计日志算出的窗口内已支出额（按 token 分列）。

先做**整笔交易级**检查，再逐 intent 检查：

1. **归属**：本密钥账户必须在 `signers[]` 里，否则 `not-a-signer`（拒绝替别人的交易出见证）。
2. **范围**：该 signer 条目若是 `Global`，需 `allow_global_scope`（默认关）——`Global` 见证会被后续任意合约花掉，白名单只能描述"眼前这笔脚本"，管不住签出去之后的用途。若是 `CustomContracts`，其 `allowed_contracts` 必须覆盖脚本里被调用的每个合约，否则 `delegation-missing`（这样产出的见证节点根本不接受）。
3. **逐 intent**（任一失败即整笔拒绝，签名是覆盖整笔交易的，不存在"只签其中无害的那半"）：
   1. 类型闸门：`Transfer` 需 `allow_transfer`，`ContractCall` 需 `allow_contract_call`。**转账不再额外要求 `allow_contract_call`**——否则"只许付款"的密钥必须同时开放任意合约调用，`allow_transfer` 就失去意义。
   2. 合约门：黑名单优先；白名单非空则必须命中。token 合约同样过这一关（拉黑一个合约意味着连它的转账也不签）。
   3. 资产门（仅转账）：`asset_blacklist` / 非空 `asset_whitelist`。
   4. `from` 必须等于本密钥账户，否则 `source-mismatch`；金额不得为负（负数会往本该扣减的窗口里加钱）。
   5. 收款人门：黑名单 / 非空白名单。
   6. 单笔上限 `max_single_amount`。
   7. 窗口上限：`窗口内已支出 + 本笔（含同一脚本内前几笔的累加） > window_limit.max_amount` → 拒。因此 4×25 在 60 的窗口下第 3 笔就被拒。
4. **共识**：`allow_consensus` + `category == "dBFT"` + `payload.sender == 本密钥账户`。类别必须硬校验，否则任意 `ExtensiblePayload`（如 NeoFS 消息）都能借"共识"名义拿委员会成员的签名。

拒绝原因是稳定的 `DenyReason` 枚举（`transfer-forbidden`、`window-amount-exceeded`……18 个，单测断言 code 唯一），响应体与审计表都用它；`detail` 只带 hash/金额/合约，不带私钥。默认全关意味着：新建密钥未配置策略时**任何签名都被拒**。

---

## 5. HTTP API

**管理面**（走现有 `require_session` 操作员会话）：
- `GET  /signer`              页面（密钥/策略/调用者/审计概览）
- `POST /signer/keys`          新增密钥（生成新私钥 或 导入 NEP-2 + 口令）
- `POST /signer/keys/{id}/policy`   保存策略
- `POST /signer/keys/{id}/disable|enable`
- `POST /signer/callers`       新增调用者（返回一次性令牌明文，仅此刻展示）
- `POST /signer/callers/{id}/rotate` 轮换令牌
- `GET  /signer/audit`         审计列表

**签名面**（独立于操作员会话；令牌 + 来源认证，见 §2.3）：
- `GET  /signer/api/key?key_id=`        读取账户地址/公钥（只读，需调用者令牌且被授权该密钥）
- `POST /signer/api/sign`  体：`{ key_id, tx_base64 }`，头：`Authorization: Bearer <token>`、`Origin`/`Referer`
  - 认证：令牌→调用者；来源需命中该调用者 `allowed_origins`（若配置）；`key_id` 需在该调用者授权集合内。
  - 解析交易 → 策略引擎 → 允许则产出 `{ witness_invocation, witness_verification, script_hash }`；拒绝则 `403` + 具体原因（不落敏感明细到响应）。
- 速率限制：按调用者 + 密钥做令牌桶（复用策略里的 `window_*`）。

签名面路由**不进** `require_session`，单独挂 `route_layer(require_caller_token)`。

---

## 6. neo-nexus 集成点

- `src/lib.rs`：新增 `pub mod signer;`
- `src/signer/`：`model.rs`（实体）、`crypto.rs`（P-256/NEP-2/见证/哈希）、`tx.rs`（交易解析）、`policy.rs`（策略引擎）、`auth.rs`（调用者令牌/来源）、`store.rs`（SQLite）、`service.rs`（编排）、`audit.rs`。
- `src/web/state.rs`：`WebState` 增 `pub signer: SignerStore`（`Clone`，`Arc`）。
- `src/web/router.rs`：新增管理路由进 `protected`；签名面 `/signer/api/*` 单独 `route_layer(require_caller_token)`。
- `src/web/nav.rs`：在 Operations 节加 `Destination { key: "signer", href: "/signer", label: "Signer" }`（`tests/web.rs` 自动覆盖，未注册路由会挂测试——正好兜底）。
- `src/web/pages/signer.rs`：渲染页面 + 管理表单处理（复用现有 `html.rs` 的页面壳与 `nav::render("signer")`）。
- 持久化：`Repository` 新增表 `signer_keys / signer_policies / signer_callers / signer_audit`；schema 放 `src/repository/schema/`，键值设置走 `settings_keys.rs` 模式；迁移在 `schema` 里加 `CREATE TABLE IF NOT EXISTS`。

## 7. 测试
- 单测（`#[path]` 拉入）：
  - `crypto`：P-256 签名/验签往返、`hash160`、单签脚本 ↔ 脚本哈希一致性、NEP-2 解密、AES-GCM 加解密、`zeroize` 后无明文残留。
  - `tx`：解析已知未签名交易 → 正确识别原生转账/合约调用；非法/畸形 → 拒绝。
  - `policy`：默认拒绝；各白/黑名单；单笔/窗口限额；签名者归属。
  - `auth`：令牌→调用者解析、来源匹配、未授权密钥拒绝。
- 集成测（`tests/web.rs` 风格）：`spawn_server` 起真实服务，走操作员会话建密钥/策略/调用者，再用调用者令牌请求 `/signer/api/sign`，断言允许路径返回见证、拒绝路径 403，且审计落库。

## 8. 安全要点
- 该历史托管引擎计划在每次签名时短暂解密并随后 `zeroize`；错误与日志只带
  `script_hash`/`key_id`，绝不带私钥。这不描述当前 `local-wallet` profile：现行实现启动时解密一次，
  以 `Arc<Zeroizing<[u8; 32]>>` 缓存到 profile 最后一个 clone drop，且不做 `mlock`。
- 令牌只存 SHA-256，新建时明文仅一次性展示；来源绑定（Origin）防跨站。
- 策略默认拒绝、解析失败拒绝、解析不出目标合约时走黑名单语义。
- 主密钥后端抽象化，便于后续切到 TEE seal；当前为本地信封加密。
- 所有签名请求审计落库。

## 9. 实施顺序（本会话落地）
1. `crypto`（P-256/哈希/单签脚本/见证）+ 单测
2. `tx` 交易解析 + 单测
3. `policy` 策略引擎 + 单测
4. `auth` 调用者令牌/来源 + 单测
5. `store` SQLite + `service` 编排 + `audit`
6. 接入 `WebState`/`router`/`nav`/`pages/signer.rs`
7. 集成测 + `cargo check`/`clippy`/`test`

## 10. 风险与权衡
- **交易解析正确性**是最大风险：策略依赖解析出的金额/目标；解析器必须对未知结构 fail-closed。本期先支持"原生转账 + 单合约调用 + 未知即拒"，复杂调用图后续增强。
- **窗口限额状态**放内存，进程重启后从审计日志重建（审计是事实来源）。
- **本地后端的主密钥**由操作员令牌派生，其强度等于令牌强度；生产应尽快接 TEE 后端。

## 11. 已知边界与待办
- `tx.rs` 把不认识的属性一律拒绝，其中 `OracleResponse(0x11)`、`NotaryAssisted(0x22)` 是**合法但本期不支持**的类型：预言机应答交易与公证人辅助交易目前签不了。需要时按类型加解析分支，并在策略里给出对应闸门。
- 合约账户（多签/签名协议合约）作为 `from` 的转账会被 `source-mismatch` 拒绝——本期只托管单签 EOA（与 §1 非目标一致）。
- `CallFlags` 只解析、不判定：`All(0x1f)` 的调用意味着被调合约能在本次执行里动用本账户资产，静态策略无法进一步约束，只能靠合约白名单。
