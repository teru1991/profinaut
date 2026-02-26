# Level 1 SSOT Outline — Domain B（Secrets / Identity / Access）
Source: Bのドキュメント「Identity / Access / Secrets Design v1.3」:contentReference[oaicite:0]{index=0}

## 0. Metadata
- Domain: **B — Secrets / Identity / Access**:contentReference[oaicite:1]{index=1}
- Status: **Implementation Target (Design) — FINAL / COMPLETE**:contentReference[oaicite:2]{index=2}
- Canonical: **SSOT domains list (fixed)**:contentReference[oaicite:3]{index=3}
- Version: v1.3:contentReference[oaicite:4]{index=4}

---

## 1. Scope and Purpose

### 1.1 Purpose
Bドメインの実装ターゲットを固定する：:contentReference[oaicite:5]{index=5}
- Secrets: secure storage / resolution / rotation / revocation / secure deletion / leak response（**zero plaintext persistence**）
- Identity: actor attribution（human/service/external）/ session security / step-up authentication
- Access: deny-by-default authorization / conditional constraints / dangerous-operation gating（Safety posture下）

### 1.2 Boundary（Out of Scope）
- Observability pipelines は domain C（Bは audit + security health signalsのみ）:contentReference[oaicite:6]{index=6}
- Safety decision logic は domain E（Bは Safety state 下で制限を強制）:contentReference[oaicite:7]{index=7}
- Runbooks は domain D（Bは hooks と evidence fields を提供。秘密は出さない）:contentReference[oaicite:8]{index=8}

---

## 2. Non-negotiables（絶対条件）

### 2.1 Fail-closed on any uncertainty
以下の不確実性があれば **fail-closed**：:contentReference[oaicite:9]{index=9}
- missing secret / invalid SecretRef / provider health failure
- policy invalid/tampered
- audit unavailable for dangerous ops
- clock skew beyond threshold
- primary lease ambiguity（dual-run split brain risk）
- （その他、同等の不確実性）

### 2.2 Secret material 禁止面（出力/保管禁止）
Secret material を以下へ出してはならない：:contentReference[oaicite:10]{index=10}
- repo / configs / logs
- HTTP responses / UI DOM / view-source / local storage
- traces / span attributes
- metrics labels
- audit events
- support bundles / exported diagnostics
- dumps / snapshots / backtraces / panics

### 2.3 Strict isolation
- env（dev/stage/prod）× mode（paper/shadow/live）は **決して交差汚染しない**:contentReference[oaicite:11]{index=11}

### 2.4 Dangerous ops 要件
Dangerous ops は必須で：:contentReference[oaicite:12]{index=12}
- step-up authentication
- challenge/confirm（payload-bound / session-bound）
- reason + bounded scope + expiry
- audit emission（all transitions）

### 2.5 Key separation / Rotation / Fallback
- purpose ごとに複数鍵（key pools）前提:contentReference[oaicite:13]{index=13}
- rotation と fallback は必須:contentReference[oaicite:14]{index=14}

### 2.6 Policy integrity
- tamper/invalid ⇒ **SAFE posture** + privileged denial:contentReference[oaicite:15]{index=15}

### 2.7 Dual-run safety
- live credentials の同時利用は禁止（controlled failover 以外）:contentReference[oaicite:16]{index=16}

### 2.8 External egress control（LLM/external share）
- “LLM/external share” の redaction と forbidden classes を厳格化:contentReference[oaicite:17]{index=17}

### 2.9 Secret sprawl prevention（registry required）
- unregistered secrets は利用禁止（registry-required）:contentReference[oaicite:18]{index=18}

### 2.10 Security-relevant changes は dangerous ops
- policy ファイルだけでなく、security boundary 影響変更は dangerous ops 扱い（拡張 change management）:contentReference[oaicite:19]{index=19}

---

## 3. Threat Model and Protected Assets（Fixed Inputs）

### 3.1 Asset Classes（policy registry required）
全 secret は分類（Critical/High/Medium）し、**登録必須**：:contentReference[oaicite:20]{index=20}

- Critical:contentReference[oaicite:21]{index=21}
  - On-chain signing keys（hot/warm/cold）/ multisig signer secrets
  - Exchange withdraw-enabled API keys
  - Root-of-trust master keys（encrypted secret store）
  - Audit integrity keys（if used）
- High:contentReference[oaicite:22]{index=22}
  - Exchange trade keys / private WS tokens
  - DB credentials / object store credentials
  - cloudflared tokens / reverse proxy credentials / internal CA keys
- Medium:contentReference[oaicite:23]{index=23}
  - Webhook signing secrets / read-only keys / non-critical tokens

### 3.2 Threats / Failure Modes
- Repo leakage（commit/PR/issues）/ runtime leakage（logs/errors/panics）/ OS leakage（swap/dumps/backups）:contentReference[oaicite:24]{index=24}
- Malware（clipboard theft / keylogger / RAT / browser extensions）:contentReference[oaicite:25]{index=25}
- Misconfiguration（wrong perms / no IP allowlist / missing sub-account separation）:contentReference[oaicite:26]{index=26}
- Human error（live-mode misfire / dangerous-op mis-approval）:contentReference[oaicite:27]{index=27}
- Dual-run split brain（concurrent “primary”）:contentReference[oaicite:28]{index=28}
- Policy tamper:contentReference[oaicite:29]{index=29}
- Session hijacking（browser）:contentReference[oaicite:30]{index=30}
- Backup leakage path:contentReference[oaicite:31]{index=31}
- “Near-secrets” exfiltration（IDs/internal URLs/whitelists/tx payloads）:contentReference[oaicite:32]{index=32}

### 3.3 Security Objectives
- あらゆる出力面からの exfiltration を防止:contentReference[oaicite:33]{index=33}
- confidentiality を落とさず recoverability を保持:contentReference[oaicite:34]{index=34}
- privileged actions を attributable / bounded / reviewable にする:contentReference[oaicite:35]{index=35}

---

## 4. Reference Architecture

### 4.1 Components
1) **security-core**:contentReference[oaicite:36]{index=36}
- redaction engine（all surfaces）
- forbidden-key scanner / near-secret classifier
- time integrity checks
- identity/session primitives
- RBAC/ABAC evaluator
- dangerous ops framework
- dual-run lease guard
- access review reporting primitives

2) **secrets-provider**:contentReference[oaicite:37]{index=37}
- SecretRef parsing/validation
- providers: fileenc（required）/ vault（optional）/ env（dev-only）
- short TTL caching + zeroize
- key pool selection
- asset-registry enforcement（unknown secrets fail）

3) **authn/authz middleware**:contentReference[oaicite:38]{index=38}
- dashboard/CLI/HTTP entrypoints
- authn / session / step-up
- authz evaluation
- dangerous ops gating
- safe errors / response shaping

4) **audit-emitter**:contentReference[oaicite:39]{index=39}
- audit event emission（no secrets）
- audit health tracking
- optional integrity chain（hash-chain/signing）

5) **policy-loader**:contentReference[oaicite:40]{index=40}
- strict schema validation
- optional signature/hash verification
- policy changes guarded by dangerous ops

### 4.2 Design Rules
- AuthZ + dangerous ops は全エントリポイントで **中央 evaluator を1つ**:contentReference[oaicite:41]{index=41}
- in-process default。将来 daemon 化しても意味論は不変:contentReference[oaicite:42]{index=42}

---

## 5. Canonical Models / Contracts（Core Data Models）

### 5.1 SecretRef（metadata confidentiality + registry requirement）
- SecretRef { scheme, path, field?, version_hint?, scope, visibility, registry_id }:contentReference[oaicite:43]{index=43}
  - scheme: fileenc | vault | env | …:contentReference[oaicite:44]{index=44}
  - scope: env, mode, venue, purpose を必須。bot_id/chain/strategy は任意:contentReference[oaicite:45]{index=45}
  - visibility: public_ref / diagnostic_ref / audit_ref / internal_only（default）:contentReference[oaicite:46]{index=46}
  - registry_id: asset registry entry への必須ポインタ:contentReference[oaicite:47]{index=47}

Rules:contentReference[oaicite:48]{index=48}
- env scheme は dev-only、prod は reject
- unscoped SecretRefs は invalid
- SecretRef metadata 自体を sensitive とみなし、表示は visibility に従う

### 5.2 Secret Material（in-memory only）
- zeroizable / non-serializable（default）:contentReference[oaicite:49]{index=49}
- Debug/Display で redacted:contentReference[oaicite:50]{index=50}
- minimal retention / use 後 wipe:contentReference[oaicite:51]{index=51}

### 5.3 Identity / Session
- ActorType: human | service | external:contentReference[oaicite:52]{index=52}
- ActorId / SessionId（short-lived opaque id）:contentReference[oaicite:53]{index=53}
- Roles[] / AuthStrength: normal | step_up:contentReference[oaicite:54]{index=54}
- DeviceBinding?（optional）:contentReference[oaicite:55]{index=55}
- correlation ids: run_id, trace_id:contentReference[oaicite:56]{index=56}

### 5.4 Authorization
- Permission + Scope + Decision(allow|deny) + reason code + evidence:contentReference[oaicite:57]{index=57}

### 5.5 Dangerous Operation（DangerOp）
- DangerOp { op_id, op_type, scope, constraints, requested_by, approvers?, reason, expires_at, challenge_state }:contentReference[oaicite:58]{index=58}
  - constraints: caps / whitelists / chain / destination / time windows / etc.:contentReference[oaicite:59]{index=59}
  - approvers?: 4-eyes readiness 用の予約領域:contentReference[oaicite:60]{index=60}
  - challenge_state: issued | confirmed | rejected | expired:contentReference[oaicite:61]{index=61}

### 5.6 Time Integrity
- event_time / ingest_time / clock_status(ok|skewed|unknown):contentReference[oaicite:62]{index=62}

---

## 6. Capabilities（機能要件の束）

### 6.1 Secrets: Storage Providers
- Minimum provider: **fileenc:**（必須）:contentReference[oaicite:63]{index=63}
- providers: fileenc（required）/ vault（optional）/ env（dev-only）:contentReference[oaicite:64]{index=64}

#### 6.1.1 Crypto Primitive Fixation（must be fixed）
実装ドリフト防止のため固定：:contentReference[oaicite:65]{index=65}
- AEAD: AES-256-GCM or ChaCha20-Poly1305（**choose one and standardize**）
- KDF: Argon2id（passphrase fallback が許可される場合のみ）
- RNG: OS CSPRNG only
- Nonce: unique per encryption / never reused
- Hierarchy: RoT → DEK（envelope encryption）
- Record format: version + salt/nonce + ciphertext + tag + metadata hash
- Compatibility: versioned format + migration rules（§38）

> TODO: AEAD をどちらに固定するか（AES-256-GCM / ChaCha20-Poly1305）

#### 6.1.2 Root-of-Trust（RoT）
- Windows prod: DPAPI/Credential Manager preferred:contentReference[oaicite:66]{index=66}
- macOS dev: Keychain:contentReference[oaicite:67]{index=67}
- passphrase fallback: dev/testing only（policy 明示で例外あり）:contentReference[oaicite:68]{index=68}
- device replacement / dual boot の migration protocol（§17/§19/§32）:contentReference[oaicite:69]{index=69}

### 6.2 Secret Distribution（Injection）
Hard rules：:contentReference[oaicite:70]{index=70}
- prod は plaintext env injection 禁止（検知したら startup fail）
- read-only mounts + strict ACL を優先
- plaintext intermediates を disk へ書かない

### 6.3 Runtime Surfaces（sealed: redaction/prevention）
対象面：logs / error chains / panics/backtraces / HTTP responses+headers / UI DOM+view-source+local storage / traces attrs / metrics labels / dumps/snapshots/backups:contentReference[oaicite:71]{index=71}

### 6.4 Support Bundles / Exports
- contamination scan + redaction（visibility rules）:contentReference[oaicite:72]{index=72}
- audit event（who/when/why）:contentReference[oaicite:73]{index=73}
- “LLM-safe bundle export” 提供（§22）:contentReference[oaicite:74]{index=74}

### 6.5 Rotation / Revocation / 429 / Leak Response
- Rotation: rotation groups / key pools active version / policy intervals / audited（no values）:contentReference[oaicite:75]{index=75}
- Revocation: resolution 時に immediate deny / live execution は privileged actions 停止 + safe stop 優先:contentReference[oaicite:76]{index=76}
- Lockout/429: 401/403/429/418… を cooldown/fallback/quarantine に map、遷移は監査:contentReference[oaicite:77]{index=77}
- Safe key verification: venue 別の最小権限エンドポイントで status（valid/invalid/insufficient/locked/unknown）だけ記録:contentReference[oaicite:78]{index=78}
- Leak suspected: safe posture hook + dangerous ops freeze（default）/ revocation workflow / key pools quarantine:contentReference[oaicite:79]{index=79}

### 6.6 On-chain Key Tiering and Constraints
- Tiers: hot（bot signing）/ warm（treasury moves: dangerous ops + step-up + constraints）/ cold（offline/hardware/multisig; execution hostに存在しない）:contentReference[oaicite:80]{index=80}
- Signing architecture（policy-driven）: in-process（hot only）/ signing service（mTLS + approvals）/ multisig/external signer:contentReference[oaicite:81]{index=81}
- Tx constraints（DangerOp constraints）: chain / destination allowlist / caps / token / expiry / nonce policy / reason（監査に constraints を含める）:contentReference[oaicite:82]{index=82}

### 6.7 Identity and Sessions（Step-up）
- Human: Cloudflare Access/OIDC / CSRF / optional device binding:contentReference[oaicite:83]{index=83}
- Service: mTLS or signed tokens / service roles + scopes:contentReference[oaicite:84]{index=84}
- Step-up: dangerous ops 必須 / methods: re-auth / TOTP / hardware key（future）/ out-of-band（future）:contentReference[oaicite:85]{index=85}
- Session security: short TTL / rotate / explicit revoke / replay resistance / audit correlation（session/run/trace）:contentReference[oaicite:86]{index=86}

### 6.8 Authorization（RBAC + ABAC）
- RBAC: deny-by-default / policy-driven mapping:contentReference[oaicite:87]{index=87}
- ABAC（required minimal）: env/mode / venue/bot/chain scope / caps+whitelists / time constraints（break-glass expiry）/ audit availability constraint（danger ops）:contentReference[oaicite:88]{index=88}
- Break-glass: activation は dangerous op / time-bound + reason-required + scope bounded / auto-expire + audited / audit down 時の例外は policy 次第:contentReference[oaicite:89]{index=89}

### 6.9 Dangerous Ops Gate
- Classification（policy）例：start_live / apply_live_config / disable_gate / relax_killswitch / rotate/revoke/delete_secret / withdraw_enable / change_withdraw_whitelist / destructive data ops / policy updates / security-relevant changes:contentReference[oaicite:90]{index=90}
- Challenge/Confirm: session-bound + payload-bound one-time challenge / confirm は reason+scope+payload hash が expiry 内一致 / transitions 全監査:contentReference[oaicite:91]{index=91}
- Preconditions: Safety SAFE/EMERGENCY は deny by default / audit unavailable は deny（break-glass 例外は policy）:contentReference[oaicite:92]{index=92}

### 6.10 Audit Reliability and Integrity
- event fields: actor/session/op/scope/reason/result/time/clock_status（no secrets）:contentReference[oaicite:93]{index=93}
- dedupe + monotonic checks:contentReference[oaicite:94]{index=94}
- optional hash-chain / signing:contentReference[oaicite:95]{index=95}
- retention/deletion/privacy rules（§34）:contentReference[oaicite:96]{index=96}

### 6.11 Environment Isolation
- namespaces split by env/mode / cross-env invalid:contentReference[oaicite:97]{index=97}
- live start requires: explicit mode=live + dangerous op confirm + Safety NORMAL + audit OK + secrets OK + lease OK（§19）:contentReference[oaicite:98]{index=98}

### 6.12 Endpoint Boundary and Safe Responses
- exposure allowlist:contentReference[oaicite:99]{index=99}
- admin endpoints: authn/authz + rate limit:contentReference[oaicite:100]{index=100}
- HTTP safe errors（reason codes; no raw stack traces）:contentReference[oaicite:101]{index=101}
- UI safe display; copy actions audited（no values）:contentReference[oaicite:102]{index=102}
- SecretRef visibility enforcement:contentReference[oaicite:103]{index=103}

### 6.13 Metrics and Trace Safety
- metrics label allowlist / sensitive・near-secret を label 禁止:contentReference[oaicite:104]{index=104}
- tracing attrs は redaction filters を通す:contentReference[oaicite:105]{index=105}
- 違反は privileged operations を block（fail-safe）:contentReference[oaicite:106]{index=106}

### 6.14 Time and Clock Integrity
- prod は NTP required:contentReference[oaicite:107]{index=107}
- skew beyond threshold ⇒ clock_status=skewed / dangerous ops deny / optional SAFE posture:contentReference[oaicite:108]{index=108}

### 6.15 OS Hardening and Least Privilege
- least-privileged service accounts:contentReference[oaicite:109]{index=109}
- secret store ACL hardening（owner-only; inheritance controlled）:contentReference[oaicite:110]{index=110}
- admin-required ops は dangerous ops に（“run as admin” を抑制）:contentReference[oaicite:111]{index=111}
- prod の dumps/swap/hibernation stance を明示:contentReference[oaicite:112]{index=112}

### 6.16 Backup and Restore Security
- backups は encrypted at rest + access-controlled:contentReference[oaicite:113]{index=113}
- plaintext intermediates を含めない:contentReference[oaicite:114]{index=114}
- restore は integrity verify / auditable:contentReference[oaicite:115]{index=115}

### 6.17 CI and Developer Guardrails
- pre-commit + CI scan（forbidden patterns + entropy）:contentReference[oaicite:116]{index=116}
- dummy secrets/stubs を提供:contentReference[oaicite:117]{index=117}
- tests never print secrets:contentReference[oaicite:118]{index=118}

### 6.18 Dual-Run / Failover Safety
- primary lease per scope（venue/bot/live）:contentReference[oaicite:119]{index=119}
- failover switch は dangerous op（reason/expiry/audit/step-up）:contentReference[oaicite:120]{index=120}
- split brain ambiguity ⇒ live を両方 deny（fail-closed）:contentReference[oaicite:121]{index=121}

### 6.19 Recovery Secrets（MFA / Recovery Codes）
- 最高制限の secrets として扱う:contentReference[oaicite:122]{index=122}
- viewing/export は dangerous ops 必須:contentReference[oaicite:123]{index=123}
- audited + time-limited:contentReference[oaicite:124]{index=124}

### 6.20 SecretRef Metadata Confidentiality
- UI/HTTP/audit/export で visibility を強制:contentReference[oaicite:125]{index=125}
- policy により audit に hashed refs を格納する option:contentReference[oaicite:126]{index=126}

### 6.21 External AI / LLM Egress Controls
- forbidden classes: secret material / near-secrets（account ids, whitelists, internal URLs, tx payloads, detailed error bodies）/ policy internals（unless redacted）:contentReference[oaicite:127]{index=127}
- “LLM-safe export” pipeline（automatic redaction）:contentReference[oaicite:128]{index=128}
- prompt-injection resistant apply flow：外部提案は gating をバイパス不可 / apply/change は dangerous op:contentReference[oaicite:129]{index=129}

### 6.22 Secure Deletion Semantics
- revoke（usage stop, keep encrypted history） vs delete（irreversible）:contentReference[oaicite:130]{index=130}
- delete は最高 tier dangerous op（double confirm + step-up + audited）:contentReference[oaicite:131]{index=131}
- storage backend ごとの best-effort secure wipe を文書化:contentReference[oaicite:132]{index=132}

---

## 7. Policy Inputs（Files）and Integrity Protection

### 7.1 Recommended policy files
- control_plane_authz_matrix.toml:contentReference[oaicite:133]{index=133}
- danger_ops_policy.toml:contentReference[oaicite:134]{index=134}
- secrets_denylist.toml:contentReference[oaicite:135]{index=135}
- secrets_ttl_rotation.toml:contentReference[oaicite:136]{index=136}
- key_pools.toml:contentReference[oaicite:137]{index=137}
- remote_access.toml:contentReference[oaicite:138]{index=138}
- llm_egress_policy.toml:contentReference[oaicite:139]{index=139}
- dual_run_policy.toml:contentReference[oaicite:140]{index=140}
- access_review_policy.toml:contentReference[oaicite:141]{index=141}
- audit_retention_policy.toml:contentReference[oaicite:142]{index=142}
- change_mgmt_policy.toml:contentReference[oaicite:143]{index=143}
- asset_registry.toml（or yaml/json）:contentReference[oaicite:144]{index=144}

### 7.2 Integrity requirements
- strict schema validation:contentReference[oaicite:145]{index=145}
- optional signature/hash verification:contentReference[oaicite:146]{index=146}
- changes require dangerous ops:contentReference[oaicite:147]{index=147}
- tamper suspicion ⇒ SAFE posture + privileged denial:contentReference[oaicite:148]{index=148}

---

## 8. Governance & Operations Extensions（Final completeness）

### 8.1 Access Review & Entitlement Governance（29）
- periodic reports（default monthly; policy-defined）:contentReference[oaicite:149]{index=149}
  - role bindings by actor
  - dangerous ops summary
  - break-glass activations
  - policy changes history
- reports に secret material を含めない / near-secrets redacted:contentReference[oaicite:150]{index=150}
- access review emits audit event:contentReference[oaicite:151]{index=151}
- prod role bindings modification は policy により dangerous ops 要求しうる:contentReference[oaicite:152]{index=152}

### 8.2 Security-Relevant Change Management（30）
- 対象例：Cloudflare Access rules / cloudflared token / OIDC settings / certificate/CA / OS dump/swap/hibernation posture / backup policy / lease policy:contentReference[oaicite:153]{index=153}
- change_mgmt_policy.toml にカテゴリ列挙:contentReference[oaicite:154]{index=154}
- 適用は dangerous ops（reason/scope/expiry/step-up/audit）:contentReference[oaicite:155]{index=155}

### 8.3 Two-Person Approval（4-eyes）Ready（31）
- dangerous ops が N-of-M approvals を要求可能（policy）:contentReference[oaicite:156]{index=156}
- approvals は payload-bound + time-bound:contentReference[oaicite:157]{index=157}
- enabled 時は single-operator bypass 不可:contentReference[oaicite:158]{index=158}

### 8.4 Physical Device Security & Theft Response Hooks（32）
- Windows prod host は full disk encryption を operational prerequisite とみなす（policy）:contentReference[oaicite:159]{index=159}
- incident hooks: key revocation workflow / Cloudflare session invalidation / RoT invalidation path（where possible）:contentReference[oaicite:160]{index=160}
- theft/compromise response は監査可能（no secrets）:contentReference[oaicite:161]{index=161}

### 8.5 Secret Sprawl Prevention（33）
- asset registry: registry_id, criticality, purpose, scope, owner, rotation interval, last rotated, allowed consumers:contentReference[oaicite:162]{index=162}
- SecretRef は registry_id 必須:contentReference[oaicite:163]{index=163}
- provider は prod で unknown registry_id を拒否:contentReference[oaicite:164]{index=164}
- registry 更新は security-relevant changes（danger ops）:contentReference[oaicite:165]{index=165}

### 8.6 Audit Retention / Deletion / Privacy（34）
- retention windows per event type（policy）:contentReference[oaicite:166]{index=166}
- audit deletion は dangerous op（if allowed）:contentReference[oaicite:167]{index=167}
- actor identifier hashing/masking（outputs）:contentReference[oaicite:168]{index=168}
- incident export procedure は redacted output:contentReference[oaicite:169]{index=169}

### 8.7 Web Session Hardening Minimums（35）
- secure cookies / SameSite / CSRF:contentReference[oaicite:170]{index=170}
- clickjacking protection headers:contentReference[oaicite:171]{index=171}
- CSP baseline（policy）:contentReference[oaicite:172]{index=172}
- step-up cache window policy:contentReference[oaicite:173]{index=173}

### 8.8 Near-Secrets Classification（36）
- near-secrets 例：account ids/subaccount names / withdraw whitelists / internal URLs / IP ranges / signing payloads / detailed error bodies / policy internals:contentReference[oaicite:174]{index=174}
- controls: all outputs redaction / external・LLM egress forbidden by default:contentReference[oaicite:175]{index=175}

### 8.9 Security GameDay / Incident Drills（37）
- drills（policy cadence）: leak suspected / device theft / audit down / split brain / policy tamper:contentReference[oaicite:176]{index=176}
- drill emits audit event + redacted report:contentReference[oaicite:177]{index=177}

### 8.10 Crypto Primitive Versioning & Migration（38）
- crypto choices versioned + compatibility rules:contentReference[oaicite:178]{index=178}
- migration strategy（format upgrades w/o data loss）:contentReference[oaicite:179]{index=179}
- startup self-test（crypto provider availability）; privileged ops は fail-closed:contentReference[oaicite:180]{index=180}

---

## 9. Acceptance Criteria（Definition of Done）
B 完了条件：:contentReference[oaicite:181]{index=181}
- plaintext leaks がどの面にも存在しない（repo/config/log/http/ui/trace/metrics/audit/bundle/dumps）
- contamination 検知で startup fails
- RBAC/ABAC enforced; dangerous ops gated
- policy integrity validated; tamper ⇒ SAFE posture + privileged denial
- revocation immediate; delete irreversible and strictly gated
- key pools が 429/lockout/leak suspected を監査付きで扱える
- dual-run lease が split brain 防止
- clock skew が dangerous ops を無効化（auditable）
- LLM-safe export が存在し forbidden classes enforced
- access review reporting が存在し scheduled

---

## 10. Work Breakdown Structure（WBS）
- Base WBS: **B-00..B-20**:contentReference[oaicite:182]{index=182}
- Additions（FINAL）:contentReference[oaicite:183]{index=183}
  - **B-21** Access Review Reporting
  - **B-22** Extended Change Management
  - **B-23** 4-eyes Approval Readiness
  - **B-24** Physical Security Hooks
  - **B-25** Registry-Required Secrets
  - **B-26** Audit Retention/Deletion/Privacy
  - **B-27** Web Session Hardening Minimums
  - **B-28** Near-Secrets Classification
  - **B-29** Security GameDay
  - **B-30** Crypto Primitive Versioning & Migration

> TODO: B-00..B-20 の内訳（項目名/目的/成果物）が本テキスト内に無いため、別ソースから補完が必要。

---

## 11. Recommended File Placement
- docs/specs/security/identity_access_design.md（this file）:contentReference[oaicite:184]{index=184}
- docs/policy/control_plane_authz_matrix.toml:contentReference[oaicite:185]{index=185}
- docs/policy/danger_ops_policy.toml:contentReference[oaicite:186]{index=186}
- docs/policy/secrets_denylist.toml:contentReference[oaicite:187]{index=187}
- docs/policy/secrets_ttl_rotation.toml:contentReference[oaicite:188]{index=188}
- docs/policy/key_pools.toml:contentReference[oaicite:189]{index=189}
- docs/policy/remote_access.toml:contentReference[oaicite:190]{index=190}
- docs/policy/llm_egress_policy.toml:contentReference[oaicite:191]{index=191}
- docs/policy/dual_run_policy.toml:contentReference[oaicite:192]{index=192}
- docs/policy/access_review_policy.toml:contentReference[oaicite:193]{index=193}
- docs/policy/audit_retention_policy.toml:contentReference[oaicite:194]{index=194}
- docs/policy/change_mgmt_policy.toml:contentReference[oaicite:195]{index=195}
- docs/policy/asset_registry.toml（or yaml/json）:contentReference[oaicite:196]{index=196}
- docs/runbooks/secrets_leak_response.md（no values）:contentReference[oaicite:197]{index=197}
- docs/runbooks/device_theft_response.md（no values）:contentReference[oaicite:198]{index=198}
- docs/runbooks/audit_down_response.md（no values）:contentReference[oaicite:199]{index=199}

---

## 12. Open TODOs（不足を推測で埋めない）
- TODO: AEAD の固定（AES-256-GCM / ChaCha20-Poly1305 の選定）:contentReference[oaicite:200]{index=200}
- TODO: B-00..B-20 の詳細WBS（本テキストに名称/成果物が無い）:contentReference[oaicite:201]{index=201}
- TODO: §16/§17/§19/§22/§30/§32/§34/§38 参照の具体的手順・閾値・パラメータ（本文は参照のみで値が未記載）:contentReference[oaicite:202]{index=202}
- TODO: “clock skew threshold” の具体値と検知実装（NTP要件はあるが閾値未提示）:contentReference[oaicite:203]{index=203}
- TODO: “LLM-safe export” の具体的な出力仕様（フォーマット/フィールド/禁止判定の厳密ルール）:contentReference[oaicite:204]{index=204}

---

## 13. Capability Index（IDs preserved）
### 13.1 WBS IDs（B-xxx）
- B-00..B-20（Base WBS）:contentReference[oaicite:205]{index=205}
- B-21 Access Review Reporting:contentReference[oaicite:206]{index=206}
- B-22 Extended Change Management:contentReference[oaicite:207]{index=207}
- B-23 4-eyes Approval Readiness:contentReference[oaicite:208]{index=208}
- B-24 Physical Security Hooks:contentReference[oaicite:209]{index=209}
- B-25 Registry-Required Secrets:contentReference[oaicite:210]{index=210}
- B-26 Audit Retention/Deletion/Privacy:contentReference[oaicite:211]{index=211}
- B-27 Web Session Hardening Minimums:contentReference[oaicite:212]{index=212}
- B-28 Near-Secrets Classification:contentReference[oaicite:213]{index=213}
- B-29 Security GameDay:contentReference[oaicite:214]{index=214}
- B-30 Crypto Primitive Versioning & Migration:contentReference[oaicite:215]{index=215}
