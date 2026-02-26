# E — Safety Controller（最終安全装置）: Level 1 SSOT Outline（vFinal）

> Source: E.txt :contentReference[oaicite:0]{index=0}

## 1. 概要

### 1.1 目的
- Safety Controller は全システムの「最後の安全装置」として、誤爆・暴走・観測不能・整合性崩壊・外部依存障害・人為ミス・未知エラーに対して、必ず安全側へ倒す。:contentReference[oaicite:1]{index=1}  
- 実行（発注）よりも安全性を優先し、「止めることに失敗しない」ことを最重要要件とする。:contentReference[oaicite:2]{index=2}  

### 1.2 固定原則（Non-negotiable）
1. Default-Safe：不明/欠損/矛盾/観測不能は SAFE/HALT :contentReference[oaicite:3]{index=3}  
2. 強制境界：Execution / Bot制御 / Apply / 破壊的Ops / 監視無効化 は必ずSafety判定を通す :contentReference[oaicite:4]{index=4}  
3. 二系統Kill：UIが死んでも止められる別経路（CLI/緊急キー/ローカル）を必ず持つ :contentReference[oaicite:5]{index=5}  
4. フェイルクローズ：Safety停止・分断・依存障害でも危険側に戻らない（Leaseで担保）:contentReference[oaicite:6]{index=6}  
5. 監査可能：遷移は reason / evidence / actor / TTL / scope が必須 :contentReference[oaicite:7]{index=7}  
6. 秘密ゼロ：監査・通知・サポートバンドルは secrets/PII を含まない（自動検知・遮断）:contentReference[oaicite:8]{index=8}  

### 1.3 概念モデル（混同禁止）= Canonical Model / Contract
#### 1.3.1 System Safety Mode（内部契約の正本）
- `NORMAL / SAFE / EMERGENCY_STOP` :contentReference[oaicite:9]{index=9}  

#### 1.3.2 Kill Switch Level（実行制約：別概念）
- `ALLOW / CLOSE_ONLY / FLATTEN / BLOCK` :contentReference[oaicite:10]{index=10}  

**固定マッピング**
- `EMERGENCY_STOP ⇒ BLOCK` :contentReference[oaicite:11]{index=11}  
- `SAFE ⇒ 原則 BLOCK（例外で CLOSE_ONLY 可：監査必須）` :contentReference[oaicite:12]{index=12}  
- `NORMAL ⇒ 原則 ALLOW（局所ハザードで段階的に絞る）` :contentReference[oaicite:13]{index=13}  

#### 1.3.3 Scope（影響範囲）と合成
- Scope：`global / venue / symbol / bot / strategy / account` :contentReference[oaicite:14]{index=14}  
- 合成は「強い制約が勝つ」（例：`global BLOCK` が最強）。:contentReference[oaicite:15]{index=15}  

#### 1.3.4 Interlock（自動安全ロック）
- 検知→判定→発動の単位。複数同時発火を許し、合成規約で答えは一意。:contentReference[oaicite:16]{index=16}  

#### 1.3.5 Evidence（根拠）
- `trace_id / run_id / audit_id / metric snapshot pointer` 等。内容は「秘密ゼロ」で参照のみ。:contentReference[oaicite:17]{index=17}  

---

## 2. 強制境界（抜け道ゼロ）

### 2.1 強制境界の対象（未判定は実行不可）
以下の操作は必ず Safety 判定を経由し、未判定は実行不可。:contentReference[oaicite:18]{index=18}  
- Execution：新規/取消/変更/ポジション操作（送信直前が最終判定点）:contentReference[oaicite:19]{index=19}  
- Bot Control：起動/停止/モード切替/資金割当/権限変更 :contentReference[oaicite:20]{index=20}  
- Config/Descriptor Apply：実行計画・接続定義・閾値ポリシーの適用 :contentReference[oaicite:21]{index=21}  
- 破壊的Ops：delete/evict/restore/compact 等 :contentReference[oaicite:22]{index=22}  
- 観測無効化：監視停止・ログ削除など（最危険）:contentReference[oaicite:23]{index=23}  

---

## 3. フェイルクローズ（Execution Lease）

### 3.1 基本
- Execution 境界は Safety が発行する短命 Lease がないと実行不可。:contentReference[oaicite:24]{index=24}  
- Lease 更新が止まれば、Execution は自動的に `BLOCK` 側へ収束。:contentReference[oaicite:25]{index=25}  

**推奨デフォルト**
- Lease TTL：20秒 :contentReference[oaicite:26]{index=26}  
- 更新周期：5秒 :contentReference[oaicite:27]{index=27}  
- Safety不達・分断：更新不能 ⇒ 新規発注拒否 :contentReference[oaicite:28]{index=28}  

### 3.2 重要原則（push / pull）
- “止める” は push（即時指令）:contentReference[oaicite:29]{index=29}  
- “許す” は pull（Leaseで継続許可）:contentReference[oaicite:30]{index=30}  
- キャッシュしても期限切れで必ず止まる（安全側へ倒れる）:contentReference[oaicite:31]{index=31}  

---

## 4. 状態機械（遷移規約）

### 4.1 遷移の原則
- 昇格（`NORMAL→SAFE→EMERGENCY_STOP`）は自動可 :contentReference[oaicite:32]{index=32}  
- 降格（`SAFE→NORMAL` 等）は原則手動（解除は危険操作）:contentReference[oaicite:33]{index=33}  
- ヒステリシス：解除は「安定期間 + 観測健全 + 整合OK」が必要 :contentReference[oaicite:34]{index=34}  

**推奨デフォルト**
- 安定期間（解除条件）：5分 :contentReference[oaicite:35]{index=35}  

### 4.2 TTLと再評価
- 手動発動（SAFE/HALT/抑制/例外）は TTL 必須 :contentReference[oaicite:36]{index=36}  
- TTL満了で自動解除はしない  
  - 再評価して「解除候補」として提示し、手動承認で解除 :contentReference[oaicite:37]{index=37}  

### 4.3 監査イベント（最低限）
- `SAFETY_TRANSITION_REQUESTED / APPLIED / REJECTED` :contentReference[oaicite:38]{index=38}  
- `INTERLOCK_TRIGGERED / CLEARED` :contentReference[oaicite:39]{index=39}  
- `LEASE_ISSUED / LEASE_DENIED`（高頻度は集約可）:contentReference[oaicite:40]{index=40}  

---

## 5. Kill Switch（二系統・即時）

### 5.1 UI Kill（オンライン）
- `SAFE / EMERGENCY_STOP` の即時適用 :contentReference[oaicite:41]{index=41}  
- `reason + TTL + challenge` 必須 :contentReference[oaicite:42]{index=42}  
- 二重送信でも冪等（Idempotency-Key）:contentReference[oaicite:43]{index=43}  
- TODO: `challenge` の具体仕様（方式・手順・失敗時挙動・監査項目）

### 5.2 CLI/緊急キーKill（別経路）
- UI/ネット/Cloudflared が死んでも停止できる :contentReference[oaicite:44]{index=44}  
- 認証方式はUIと分離（別鍵・ローカル権限・可能なら物理キー）:contentReference[oaicite:45]{index=45}  
- TODO: 「別鍵/ローカル権限/物理キー」の運用・保管・ローテ・紛失時対応

### 5.3 HALT適用順序（固定）
1. 新規発注を `BLOCK` :contentReference[oaicite:46]{index=46}  
2. 可能ならオープンオーダー取消（競合/遅延前提）:contentReference[oaicite:47]{index=47}  
3. Bot停止/隔離 :contentReference[oaicite:48]{index=48}  
4. 監査記録 + サポートバンドル採取（秘密ゼロ）:contentReference[oaicite:49]{index=49}  

---

## 6. 自動インターロック（安全ロック）

### 6.1 入力カテゴリ（網羅）
- 観測：targets down、ログ/メトリクス欠損、黙死 :contentReference[oaicite:50]{index=50}  
- 実行：429/拒否急増、取消失敗、注文嵐、冪等衝突 :contentReference[oaicite:51]{index=51}  
- リスク：PnL急落、DD、露出逸脱、清算リスク :contentReference[oaicite:52]{index=52}  
- データ品質：欠損、順序乱れ、遅延悪化、stale data :contentReference[oaicite:53]{index=53}  
- 時刻：ドリフト/巻き戻り/スリープ復帰ジャンプ :contentReference[oaicite:54]{index=54}  
- 整合：reconcile不一致、注文/ポジ/残高不整合 :contentReference[oaicite:55]{index=55}  
- 外部依存：DNS/TLS期限/NTP/Tunnel/Disk/WAL/Secrets供給 :contentReference[oaicite:56]{index=56}  
- 未知：分類不能例外の増加（UNKNOWN系）:contentReference[oaicite:57]{index=57}  

### 6.2 代表Interlock（最低限）
- `OBS-UNKNOWN → SAFE` :contentReference[oaicite:58]{index=58}  
- `CLOCK-CRITICAL → SAFE` :contentReference[oaicite:59]{index=59}  
- `RECONCILE-MISMATCH → SAFE（状況でHALT）` :contentReference[oaicite:60]{index=60}  
- `ORDER-STORM / CANCEL-LOOP → SAFE→HALT（段階）` :contentReference[oaicite:61]{index=61}  
- `PnL/Exposure急変 → CLOSE_ONLY/FLATTEN（段階）` :contentReference[oaicite:62]{index=62}  
- `DISK/WAL危険 → SAFE` :contentReference[oaicite:63]{index=63}  
- `UNKNOWN分類増加 → SAFE→HALT（段階）` :contentReference[oaicite:64]{index=64}  
- TODO: 各 interlock の「閾値/解除条件/段階ラダー/観測根拠の最小セット」

### 6.3 合成規約（答えを一意にする）
- 優先：`HALT > SAFE > FLATTEN > CLOSE_ONLY > ALLOW` :contentReference[oaicite:65]{index=65}  
- `CRITICAL` はラッチ（手動解除）:contentReference[oaicite:66]{index=66}  
- 抑制（suppress）は TTL最短・scope限定・監査必須 :contentReference[oaicite:67]{index=67}  
- TODO: `CRITICAL` 判定条件の定義、suppress の具体API/運用

---

## 7. 取引の現実（部分約定/取消レース/遅延）への対応
- 取消しても約定する、部分約定が残る、約定通知が遅れる前提で設計する :contentReference[oaicite:68]{index=68}  
- 最終状態は reconcile で確定（イベント遅延・逆順は吸収）:contentReference[oaicite:69]{index=69}  

**SAFE/HALT時の既存注文**
- 原則「全取消」:contentReference[oaicite:70]{index=70}  
- 取消不能/遅延は「凍結扱い + 手動介入」に切り替える（事故拡大防止）:contentReference[oaicite:71]{index=71}  
- TODO: 「凍結扱い」の定義（表示・アラート・操作禁止範囲・解除手順）

---

## 8. 注文安全（Fat-finger/Outlier/Loop防止）

### 8.1 Order Validation（必須）
- NaN/0/極端値拒否 :contentReference[oaicite:72]{index=72}  
- 最大サイズ/最大想定元本 :contentReference[oaicite:73]{index=73}  
- 直近価格からの乖離上限 :contentReference[oaicite:74]{index=74}  
- tick/step/最小数量/最小金額違反拒否 :contentReference[oaicite:75]{index=75}  
- Close-only中に逆方向新規建て禁止 :contentReference[oaicite:76]{index=76}  
- TODO: 各上限/閾値の既定値、venue/symbol別の適用優先、拒否時の監査項目

### 8.2 Retry/無限ループ防止（必須）
- 最大リトライ/最大滞留時間 :contentReference[oaicite:77]{index=77}  
- 429は指数バックオフ + jitter :contentReference[oaicite:78]{index=78}  
- 冪等キー必須（再起動跨ぎ衝突を防ぐ epoch を含める）:contentReference[oaicite:79]{index=79}  
- cancel-loop/order-storm は段階停止へ連動 :contentReference[oaicite:80]{index=80}  
- TODO: epoch の生成/永続化、冪等キーの衝突時扱い、バックオフ係数

---

## 9. 例外許可（Exception）と手動介入

### 9.1 Exception（必要だが危険）
- scope限定 + TTL最短 + 監査必須 + 段階許可（canary的）:contentReference[oaicite:81]{index=81}  
- CRITICALラッチを上書きできない :contentReference[oaicite:82]{index=82}  
- TODO: 例外の申請/承認フロー、canary段階の定義、拒否理由の標準化

### 9.2 手動介入モード
- 調査中に状態が揺れないよう、変更を最小化 :contentReference[oaicite:83]{index=83}  
- 調査が終わるまで解除・抑制・例外以外の変更を制限可能 :contentReference[oaicite:84]{index=84}  
- TODO: 「制限可能」の対象API一覧と例外（緊急停止は常に可能、等）

---

## 10. 清算リスクとの整合（止めるほど危険な場合）
- 取引停止で清算リスクが上がる場合がある :contentReference[oaicite:85]{index=85}  
- Risk interlock に「清算回避のための最小限の行動」を含める（FLATTEN優先等）:contentReference[oaicite:86]{index=86}  
- 行動は Safe Flatten アルゴリズム（分割/TWAP、成行禁止条件）に従う :contentReference[oaicite:87]{index=87}  
- TODO: Safe Flatten アルゴリズムの契約（入力/制約/禁止条件/停止条件/監査）

---

## 11. 変更管理（Policy/Config）と段階導入（Canary）
- interlock閾値・解除条件・ラダー・予算は Policy-as-Code（SemVer/差分/レビュー/即ロールバック）:contentReference[oaicite:88]{index=88}  
- Config/Descriptor Apply は（静的検証/影響範囲サマリ/差分レビュー）:contentReference[oaicite:89]{index=89}  
- ドリフト検知（勝手な差し替え）→ SAFE :contentReference[oaicite:90]{index=90}  
- 段階導入：最初は 1 bot / 1 symbol / 1 venue のみ ALLOW（canary）:contentReference[oaicite:91]{index=91}  
- TODO: Policy-as-Code の格納場所/形式、差分レビュー基準、ロールバック手順

---

## 12. 監査・改ざん耐性・秘密ゼロ
- 遷移は必ず actor/reason/ttl/scope/evidence :contentReference[oaicite:92]{index=92}  
- 監査が書けないなら解除/抑制/例外許可を拒否（ただし HALT は最優先で実行）:contentReference[oaicite:93]{index=93}  
- append-only +（可能なら）ハッシュチェーン :contentReference[oaicite:94]{index=94}  
- 監査/通知/バンドルへの secrets/PII 混入を検知し遮断 or 強制マスク :contentReference[oaicite:95]{index=95}  
- 監査連打で自滅しない（集約・レート制限。ただし HALT は必ず残す）:contentReference[oaicite:96]{index=96}  
- TODO: secrets/PII 検知ルール、マスク方式、ハッシュチェーンの実装範囲

---

## 13. Safetyの最小依存（依存ループ防止）
- Safety は観測基盤/DB/バスに依存しすぎない :contentReference[oaicite:97]{index=97}  
- “止める” 経路はローカル完結で成立 :contentReference[oaicite:98]{index=98}  
- 監査/通知が死んでも停止は成立し、後で追記できる（秘密ゼロ）:contentReference[oaicite:99]{index=99}  
- TODO: ローカル完結の最小構成（プロセス/IPC/権限/永続先）

---

## 14. 永続化・再起動・アップデートの安全
- `SafetyState / InterlockState / Exception / TTL / epoch` を永続化 :contentReference[oaicite:100]{index=100}  
- 復元不能なら SAFEで起動 :contentReference[oaicite:101]{index=101}  
- 起動直後は SAFE/BLOCK（warm-up 推奨 2分）:contentReference[oaicite:102]{index=102}  
- Safety更新は SAFE中に実施し、canaryで段階解放、即ロールバック可能 :contentReference[oaicite:103]{index=103}  
- TODO: 永続化ストア/フォーマット、warm-up中の許可条件、アップデートの手順書

---

## 15. 外部UIとローカル制御の分離（Cloudflared含む）
- 外部UIが死んでもローカルKillは生存 :contentReference[oaicite:104]{index=104}  
- 外部経由の解除はより硬い（time-lock/4-eyes推奨）:contentReference[oaicite:105]{index=105}  
- 認証方式・鍵・経路を分離 :contentReference[oaicite:106]{index=106}  
- TODO: 4-eyes の具体実装、time-lock の設定、Cloudflaredダウン時の到達手段

---

## 16. SLO/SLI（安全装置の“速さ”も仕様）

### 16.1 推奨目標
- HALT指令 → 新規発注拒否まで < 100ms（同一ホスト）:contentReference[oaicite:107]{index=107}  
- Lease期限切れ → 完全BLOCKまで ≤ 30s :contentReference[oaicite:108]{index=108}  
- Unknown分類増加 → SAFE/HALT の段階遷移が観測できる :contentReference[oaicite:109]{index=109}  
- これらSLOをメトリクス化し、逸脱をアラート :contentReference[oaicite:110]{index=110}  
- TODO: SLI定義（計測点/ラベル/集計窓/アラート条件）

---

## 17. 完了定義（E Done）
E が「完全実装可能な洗い出し完了」と言える条件：:contentReference[oaicite:111]{index=111}  
- 境界抜け道ゼロ（判定を通らない実行が存在しない）:contentReference[oaicite:112]{index=112}  
- 二系統KillでUI不能でも停止可能 :contentReference[oaicite:113]{index=113}  
- Safety停止/分断でもLeaseでBLOCK収束 :contentReference[oaicite:114]{index=114}  
- interlockが観測/実行/整合/依存/時間/未知まで網羅 :contentReference[oaicite:115]{index=115}  
- 合成規約が一意で揺れない（優先/ラッチ/抑制/scope）:contentReference[oaicite:116]{index=116}  
- 解除が最も硬い（段階解除+チェックリスト+監査必須）:contentReference[oaicite:117]{index=117}  
- 部分約定/取消レース/遅延約定を吸収できる :contentReference[oaicite:118]{index=118}  
- Policy/Config変更が事故原因にならない（差分/レビュー/ロールバック/canary）:contentReference[oaicite:119]{index=119}  
- DEV/LIVE取り違え防止、秘密ゼロ、永続化、SLOが揃っている :contentReference[oaicite:120]{index=120}  
- TODO: DEV/LIVE取り違え防止の具体策（ガード/識別/強制表示/二重確認）

---

## 18. Capability Index（ID/用語インデックス：保持必須）

### 18.1 Mode / Level / Scope
- System Safety Mode: `NORMAL`, `SAFE`, `EMERGENCY_STOP` :contentReference[oaicite:121]{index=121}  
- Kill Switch Level: `ALLOW`, `CLOSE_ONLY`, `FLATTEN`, `BLOCK` :contentReference[oaicite:122]{index=122}  
- 固定マッピング: `EMERGENCY_STOP ⇒ BLOCK`, `SAFE ⇒ 原則 BLOCK（例外で CLOSE_ONLY 可）`, `NORMAL ⇒ 原則 ALLOW` :contentReference[oaicite:123]{index=123}  
- Scope: `global`, `venue`, `symbol`, `bot`, `strategy`, `account` :contentReference[oaicite:124]{index=124}  

### 18.2 Interlock IDs（代表）
- `OBS-UNKNOWN` :contentReference[oaicite:125]{index=125}  
- `CLOCK-CRITICAL` :contentReference[oaicite:126]{index=126}  
- `RECONCILE-MISMATCH` :contentReference[oaicite:127]{index=127}  
- `ORDER-STORM` :contentReference[oaicite:128]{index=128}  
- `CANCEL-LOOP` :contentReference[oaicite:129]{index=129}  
- `DISK/WAL危険` :contentReference[oaicite:130]{index=130}  
- `UNKNOWN分類増加` :contentReference[oaicite:131]{index=131}  

### 18.3 Audit / Event IDs（最低限）
- `SAFETY_TRANSITION_REQUESTED` :contentReference[oaicite:132]{index=132}  
- `SAFETY_TRANSITION_APPLIED` :contentReference[oaicite:133]{index=133}  
- `SAFETY_TRANSITION_REJECTED` :contentReference[oaicite:134]{index=134}  
- `INTERLOCK_TRIGGERED` :contentReference[oaicite:135]{index=135}  
- `INTERLOCK_CLEARED` :contentReference[oaicite:136]{index=136}  
- `LEASE_ISSUED` :contentReference[oaicite:137]{index=137}  
- `LEASE_DENIED` :contentReference[oaicite:138]{index=138}  

### 18.4 Terms / Keys
- `Lease TTL=20秒`, `更新周期=5秒` :contentReference[oaicite:139]{index=139}  
- `Idempotency-Key` :contentReference[oaicite:140]{index=140}  
- `epoch`（冪等キー衝突防止の文脈）:contentReference[oaicite:141]{index=141}  
- `reason`, `evidence`, `actor`, `TTL`, `scope` :contentReference[oaicite:142]{index=142}
