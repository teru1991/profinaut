# Level 2 Deep Spec — O. Deterministic Replay / Audit Event Log
> 整理のみ / 新規仕様追加なし。未記載は TODO。 

## 1. Non-negotiable（再掲：監査の“守るべき線”）
- append-only + canonical schema（重要イベントの単一スキーマ化） :contentReference[oaicite:45]{index=45}
- hash chain + checkpoint による完全性検証（verify） :contentReference[oaicite:46]{index=46}
- 任意run replay と主要出力の自動検証（replay+diff） :contentReference[oaicite:47]{index=47}
- 秘密情報ゼロ（ログ/bundle/reportに秘密実体が混入しないことを機械検査で保証） :contentReference[oaicite:48]{index=48}
- 障害条件下でも証跡が破綻しない（欠損/遅延/容量逼迫/クラッシュ/二重起動/観測欠損/手動介入/長期破損/部分障害/輸送障害） :contentReference[oaicite:49]{index=49}
- 検索結果の完全性（query completeness）を説明できる :contentReference[oaicite:50]{index=50}
- Decision Explanation Capsule を即時提示できる :contentReference[oaicite:51]{index=51}
- CI/ReleaseでO準拠が強制され、運用ケイデンスで維持される :contentReference[oaicite:52]{index=52}

## 2. Canonical Model（監査イベントの正準データモデル）
### 2.1 Canonical Event Header（正準ヘッダ）
ヘッダ項目はLevel1のとおり（schema_version〜redaction_level）。 :contentReference[oaicite:53]{index=53}  
**TODO（仕様不足のため保留）**
- TODO: `schema_version` の表記規約（SemVerか、互換ポリシーは何か）
- TODO: `event_type` の列挙と互換ポリシー（破壊的変更の扱い）
- TODO: `severity` の列挙/定義（例：INFO/WARN/ERROR/CRITICAL 等）
- TODO: `trace_id/span_id` のフォーマット（W3C Trace Context等）と必須/任意
- TODO: `*_ref` の参照先SSOT（H/N/I/J/C/D/Y等）とhash bindingの手順
- TODO: `prev_hash/hash` の算出対象（ヘッダ＋本文の正規化表現）とアルゴリズム
- TODO: `txn_id` 採用条件・トランザクション境界の規約

### 2.2 canonicalization & digest（正規化と要約）
- 主要入力・状態・判断を正規化してdigest化し、diffはdigest差分から原因候補を提示する。 :contentReference[oaicite:54]{index=54}  
**TODO（仕様不足のため保留）**
- TODO: 正規化規則（キー順/配列順/数値丸め/時刻/locale/浮動小数/NaN等）
- TODO: digest種別（入力digest/状態digest/判断digest等の分割）と命名
- TODO: 再現性Tier（Tier0〜3）に応じたdigest/記録粒度の差

## 3. Determinism Contract（決定論の契約）
- replay時は **clock注入・rng注入・順序完全化・整数化・外部I/O禁止** を満たす。 :contentReference[oaicite:55]{index=55}
- 非決定要因検知イベント：`NondeterminismDetected`。 :contentReference[oaicite:56]{index=56}  
**TODO（仕様不足のため保留）**
- TODO: “外部I/O禁止”の定義（許可される例外：キャッシュ/固定スナップショット等）
- TODO: RNG注入の方式（seed/event由来seed等）とseedの監査記録
- TODO: clock注入の方式（論理時計/固定タイムライン/時刻歪みの扱い）
- TODO: “整数化”の対象（価格/数量/手数料等）とスケーリング規約（unit_policy_ref）

## 4. Integrity Model（完全性：hash chain / checkpoint / verify）
- hash chain と checkpoint を持ち、verify結果は `VALID/INVALID/INCOMPLETE`。run中もContinuous verifyする。 :contentReference[oaicite:57]{index=57}  
**TODO（仕様不足のため保留）**
- TODO: checkpoint間隔/形式（イベント数/時間/セグメント境界）
- TODO: 署名/アンカー/タイムスタンプの採用条件（O-48等）と鍵管理
- TODO: INCOMPLETEの扱い（復旧可能性、再検証の手順、可観測性）

## 5. Storage Model（Append-only WAL）
- セグメント形式：length-prefix
- hot/warm/cold の回転/保持
- partial write検出によるクラッシュ復旧 :contentReference[oaicite:58]{index=58}  
**TODO（仕様不足のため保留）**
- TODO: セグメント命名規則、indexとの整合、暗号化（O-96）適用点
- TODO: 二重起動/並行writerの排他（ロック/リース）と監査イベント

## 6. Replay / Verify / Diff（挙動仕様）
- replayモード：pure / hybrid / audit :contentReference[oaicite:59]{index=59}
- 重要イベント比較、差分最小化、因果鎖再構成 :contentReference[oaicite:60]{index=60}  
**TODO（仕様不足のため保留）**
- TODO: “主要出力”の定義（例：PnL/注文系列/ポジション推移等）と比較項目
- TODO: 差分最小化のアルゴリズム（どの粒度で原因候補を提示するか）
- TODO: 因果鎖の表現（O-16/O-90との関係）

## 7. DLP（秘密情報ゼロの機械保証）
- 構造防止＋検知＋allowlist
- 違反は拒否/隔離
- bundle/report生成時はscan必須 :contentReference[oaicite:61]{index=61}  
**TODO（仕様不足のため保留）**
- TODO: 秘密情報の定義（鍵/トークン/個人情報/機密パラメータ等）とルールセット
- TODO: 隔離の手順（保存先/アクセス制御/インシデント連携）

## 8. API/CLI（インタフェース整理）
対象コマンド：verify / continuous-verify / replay / diff / export(L0..L3) / attestation / report / query(--explain --completeness) / scrub / migrate / rebuild-index / compliance-check / coverage / alert/incident link / ratelimit。 :contentReference[oaicite:62]{index=62}  
**TODO（仕様不足のため保留）**
- TODO: CLIの入出力スキーマ（JSON/テキスト/exit code）、互換保証
- TODO: export L0〜L3 の内容定義（含有物・自己完結性）と暗号化オプション（O-27/O-103）

## 9. Behavior / Tests（CIで担保すべき振る舞い）
CI最低限テスト一覧：schema互換、expected event contract、DLP、determinism golden、tamper、crash recovery、continuous verify、query completeness/explain、migration/matrix/rebuild、scrub、alert/incident整合、rate limit ledger、policy hash binding、access/sanitize/deletion/approval/flags/release整合、外部境界（transport/time/id/checksum/license）。 :contentReference[oaicite:63]{index=63}  
**採用時ルール**：OPTIONAL/SHOULD/EXTENSIONはmanifestで必須化し、専用テスト追加。 :contentReference[oaicite:64]{index=64}  
**TODO（仕様不足のため保留）**
- TODO: “determinism golden” のベクタSSOT（O-74）配置、生成、更新フロー
- TODO: “tamper” の改ざんモデル（どこを変えたらINVALIDになるか）と期待結果
- TODO: “query completeness/explain” の説明フォーマットと評価基準
- TODO: “migration/matrix” の互換性レベル定義と検証観点

## 10. Capability ↔ Spec Trace（ID保持：要点だけ）
- Canonical schema：O-01 :contentReference[oaicite:65]{index=65}
- Emitter/WAL/Integrity/Query：O-02〜O-05 :contentReference[oaicite:66]{index=66}
- Determinism/Replay/Diff：O-06〜O-08 :contentReference[oaicite:67]{index=67}
- Export/Bundle/Report：O-09/O-35/O-44 :contentReference[oaicite:68]{index=68}
- Governance/CI/KPI/Breach/Cadence：O-67〜O-71 :contentReference[oaicite:69]{index=69}
- Alert/Incident：O-82/O-83 :contentReference[oaicite:70]{index=70}
- External boundary：O-77〜O-81 :contentReference[oaicite:71]{index=71}
- Extensions：O-101〜O-108 :contentReference[oaicite:72]{index=72}
