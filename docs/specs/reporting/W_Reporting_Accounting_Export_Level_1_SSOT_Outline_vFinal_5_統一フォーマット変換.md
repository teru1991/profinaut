分類理由: タイトルと本文の主軸が Reporting / Accounting / Export（レポート/出力）であり、この文書を reporting に分類するため。
# W: Reporting / Accounting / Export — Level 1 SSOT Outline（vFinal.5 → 統一フォーマット変換）

> 原文: 「W ドメイン（Reporting / Accounting / Export）実装目標・詳細設計 / 統合SSOT（完全網羅・最終版 vFinal.5）」
>
> NOTE: 推測で仕様を増やさず、原文にない詳細は **TODO:** を残す。  
> NOTE: 原文内に A-xxx / Sxx / T-xx / F-xx / Y-xx 等の **番号/ID は見当たらない**ため、Capability Index には **ID未提示**として記載する。

---

## 1. ドメイン概要（W）

### 1.1 位置づけ
- W は「取引履歴整形・損益計算・税務向けエクスポート・監査証跡（改竄不能性、保持）」を核とするドメイン。
- CEX/DEX/オンチェーン/（将来の株・FX等）入力を、会計に耐える共通台帳へ正規化し、決定論的に再計算可能で、監査・提出・長期検証・受け渡し証明・供給網安全に耐える出力（損益/残高/税務・会計）を生成する。

---

## 2. Non-negotiable（ゴール）

- W の最終到達点：
  - 多様な入力（CEX/DEX/オンチェーン/将来の株・FX等）を **会計に耐える共通台帳（Canonical Ledger）**へ正規化
  - **決定論的に再計算可能**
  - **監査・提出・長期検証・受け渡し証明・供給網安全**に耐える形で
  - **損益/残高/税務・会計出力**を生成できること

---

## 3. Scope / Responsibilities（責務境界）

### 3.1 W が直接提供するもの（コア能力＋全拡張）
- Canonical Ledger（正規化台帳）の生成・保持（append-only）
- Truth/Idempotency（真実決定・重複収束・近似同一判定・順不同耐性）
- Valuation/FX（評価）＋品質保証（異常検知）＋参照凍結（snapshot）＋手動手当（代替価格選択）
- Cost Basis（簿価）＋ロット/平均単価説明可能性＋端数/丸め固定＋分割束ね（意図単位）
- PnL 派生（科目分離）＋差分原因解析（Why-diff）＋PnL Attribution（要因分解）＋サニティガード
- Tax Regime/Policy（日本優先）＋複数ビュー同時生成・比較＋what-if 試算＋収益区分推定支援
- Journal/CoA（複式仕訳）＋仕訳ルール（テンプレ/DSL）＋例外処理＋プレビュー＋期間ロック
- Voucher（伝票）単位の起票/承認/証憑紐づけ（仕訳束ね）
- Reconciliation/Adjustment（照合と収束、棚卸し、補正案提示、整合違反自動修正候補）
- Reports（数値＋文章＋PBC/監査資料、インタラクティブ明細、年度横断比較、提出説明資料）
- Export（提出/監査/内部）＋テンプレ/ルール管理＋CSV安全化＋二重出力（CSV+JSON）＋ファイル差分出力
- Documents ingestion（証憑取り込み/自動取得/更新履歴/突合）
- Evidence Pack（証明パック）＋Verifier（同梱）＋Verify gates＋最小反例生成
- Audit Index（索引）＋Chain of Custody（受け渡し証明）＋提出状態管理
- Recovery（破損検知/自己修復）＋bit-rot 検知
- Drift 検知（ベンダ/仕様変更、分布変化）＋正規化ルール移行（Migration）吸収
- Pending Tracker（未確定・未着・失敗のフォローアップ）
- Edge-case 辞書（例外取引テンプレ）＋資産カテゴリ会計プリセット（現物/先物/OP/LP等）
- 異常入力隔離ビュー（隔離箱）
- CorporateActionEvent（株/暗号の分割・併合・移行等の一般化）
- 逆引き検索（PnL/仕訳/提出→台帳→Raw/証憑）
- トレーサビリティ・グラフ出力（因果グラフ可視化）
- 共同作業支援（注記Bのコメント/状態管理、税理士Q&Aテンプレ）
- 品質スコアリング（確定度/根拠/価格品質）＋品質ヒートマップ
- ルールセット共有（policy/仕訳/端数/テンプレのexport/import）
- 定型ワークフロー機能（月次締め/年次提出準備/証憑更新など）
- diff 意味付け（更正/価格更新/policy変更などの原因ラベル付与）
- 提出前チェックの“安全自動修正”（名称揺れ/端数/注記整理の提案→適用）
- counterparty/アドレス辞書のexport/import
- サンプルデータ生成（テスト/説明用）
- 異常パターン検知カタログ（スリッページ異常等）

### 3.2 W が要件として固定するもの（他ドメイン実装可）
- 時刻規約（JST/UTC境界、event_time信用度、補正ルール、watermark）
- セキュリティ規約（RBAC/ABAC、分類ラベル、DLP・秘密ゼロ）
- 供給網安全（署名/SBOM/再現ビルド）と検証互換維持

---

## 4. Definition of Done（完成条件：完全実装の基準）

> 原文では「vFinal.4 と同一。全文保持」とあり、以下が列挙されている（本文は原文準拠で保持）。

### 4.1 再現性・正当性（Determinism / Replay）
1. 決定論再計算：同一入力（Raw参照）・同一policy・同一実装version・同一環境指紋で Canonical/Derived/Export のハッシュ一致
2. 参照データ凍結：外部価格/FX等は versioned snapshot を参照し、過去評価が勝手に変わらない
3. 二系統検算：独立経路の再計算で一致。不一致なら final禁止＋最小反例生成
4. 境界値仕様固定：端数/精度/丸め/符号（-0禁止等）固定、float原則禁止（decimal/bigint）
5. 再現可能ビルド/実行：依存固定、成果物digest固定、env_fingerprint証跡化

### 4.2 台帳・真実決定・入力品質（Truth / Contracts）
6. Append-only台帳：削除/上書き禁止。訂正は Reversal/Supersede/Adjustment
7. 真実決定ルール：複数ソース矛盾の優先順位/矛盾解決/再取得/疑義注記を仕様化
8. 遅延・順不同・再送：idempotency＋近似同一＋差分イベントで標準処理
9. 入力品質契約：venue別最低保証フィールド、欠損時の provisional/fatal を明確化
10. 悪意入力耐性：巨大payload/異常桁/DoS/CSV式注入を拒否・隔離・タイムアウト

### 4.3 provisional/final・Close/Reopen・一貫性モデル
11. 二層出力：provisional（暫定）と final（確定）を分離、切替条件・差分レポート必須
12. Watermark：venue/account単位の「ここまで確定」を定義
13. Close固定：close_manifestで入力集合を固定（締めの正本）
14. Reopen例外：監査ログ＋差分比較＋再署名＋履歴保持必須
15. 締め後訂正：原則次期Adjustment、例外はReopenで統制

### 4.4 Reconciliation（照合）
16. Reconciliationが完成条件：残高/約定/入出金と突合し乖離を原因候補つきで提示
17. 収束原則：乖離はAdjustmentで収束、不能なら final禁止（degraded）

### 4.5 提出安全・可観測性（Security / DLP / Audit Logs）
18. RBAC/ABAC：tenant/entity/book単位で閲覧/出力権限分離
19. 秘密ゼロ保証：提出物・証明パック・ログ・検証レポに secrets混入ゼロ（DLP＋追加検査＋ハニートークン）
20. 提出監査ログ：生成/承認/配布/取り下げまで追跡（actor/宛先ラベル/マスクレベル/ハッシュ/DLP結果）
21. 二段階（推奨）：生成→承認→出力（2人ルールと連動）

### 4.6 Evidence / Verify / Auditor-of-Auditor
22. 証明パック標準：manifest/ledger/derived/valuation_sources/reconciliation/notes/SIGNATURE＋README_AUDIT.md
23. Verifier同梱：パック単体でオフライン検証＋検証レポ出力
24. 段階ゲート：Gate-0→Gate-1→Gate-2（final条件）→Gate-3（強：独立再計算/参照整合/ドリフト）
25. 最小反例生成：不一致時の入力最小化＋再現フィクスチャ保存
26. 監査の監査：verify結果を署名して保存（否認不能）

### 4.7 Chain of Custody / 提出状態
27. 状態機械：draft/ready/submitted/retracted/superseded
28. 受け渡し連鎖：生成→承認→配布→（任意で受領署名）をハッシュ/署名で連鎖
29. submitted後の上書き禁止：retracted→new submission を強制

### 4.8 長期運用（復旧・保持・移行・腐敗）
30. 破損検知/自己修復：manifestで欠損検出→再生成→不能なら欠損宣言
31. bit-rot検知：定期ハッシュ再検証
32. 保持/削除ポリシー：種別ごとの保持期間、監査性を壊さない削除
33. 監査可能な削除：削除の証跡＋匿名化トレース
34. 互換移行：旧manifest/旧出力/旧replayの検証互換維持（破壊的変更はSemVer major）
35. ベンダ変更追跡：仕様変更の発生点と影響年度をaudit_indexへ紐付け

### 4.9 供給網安全（Supply Chain Security）
36. verifier/cliに署名/SBOM/再現ビルド、利用成果物digestを証跡化

### 4.10 ヒューマン・ファクター（事故封じ）
37. 生成者≠承認者、手動Adjustmentも同様（solo_mode例外は明記）
38. Checklist-as-Code：提出要件未達ならsubmittedを機械的にブロック
39. インシデント標準（retracted/superseded/RCA/通知テンプレ）
40. single-path CLI（年次提出一本道）
41. README_AUDIT.md同梱（監査人が迷わない）

---

## 5. Functional Requirements（機能要件）

### 5.1 vFinal.4 由来（3.1〜3.20）
- 原文には「3.1〜3.20（vFinal.4と同一：略さず実装要件として保持）」とあるが、本文では **具体列挙が省略**されている。
- ここでは、原文に載っている“包含対象”のみ保持する：
  - Canonical Ledger
  - 分類/クラスタリング
  - 経費/配賦
  - 証憑
  - 内部振替
  - Bundle/分解
  - Valuation（手動手当含む）
  - Cost Basis
  - PnL（Attribution含む）
  - 税務比較/what-if
  - 会計/仕訳ルール/Voucher
  - 棚卸し
  - レポート
  - Export（差分含む）
  - Search
  - 共同作業
  - 品質スコア
  - Migration
  - 自動修正候補
  - サンプル生成
- **TODO:** vFinal.4 の 3.1〜3.20 の全文（「略さず保持」対象）をここへ貼り込み、正式に固定する。

### 5.2 vFinal.5 追加（3.21）
- トレーサビリティ・グラフ出力：提出行→台帳→評価→簿価→Raw/証憑を因果グラフで可視化（HTML/Graphviz）
- ルールセット共有（import/export）：policy/仕訳/端数/テンプレ/辞書をパッケージ化して共有
- 定型ワークフロー：monthly-close / annual-submit-ready / refresh-docs-and-reconcile 等を機能として提供
- diff 意味付け：diffに原因ラベル（更正/価格更新/policy変更等）を自動付与（手動修正可）
- 資産カテゴリ会計プリセット：現物/先物/OP/レンディング/ステーキング/LP等の会計処理テンプレ
- 異常パターン検知カタログ：スリッページ/手数料率/刻み違反/分布異常などのテンプレ検知と原因提示
- 提出前チェックの安全自動修正：名称揺れ/端数/注記整理の提案→ワンボタン適用
- counterparty/アドレス辞書共有：辞書のexport/import
- データ品質ヒートマップ：期間×資産×venueの確定度/補間率/乖離率を可視化
- 税理士Q&A回答テンプレ生成：注記Bや差分要因から質問＆回答候補（数字入り）を生成

---

## 6. Core Data Model（固定）

- Canonical Ledger / manifest / audit_index は vFinal.4 と同じ固定仕様（破壊的変更禁止）
- **TODO:** vFinal.4 の「Canonical Ledger / manifest / audit_index」固定仕様の全文（または参照先SSOT）をここへ取り込み。

---

## 7. Verify / Invariants / Gates（固定）

- Gate-0〜Gate-3
- Invariant Catalog
- 最小反例生成
- 検証レポ署名（監査の監査）
- **TODO:** Gate 定義（各Gateの条件/入力/出力/失敗時挙動）と Invariant Catalog の全文（vFinal.4固定点）をここへ取り込み。

---

## 8. 推奨モジュール境界（実装配置）

- vFinal.4 と同様（w-core-ledger〜w-cli）
- 追加モジュール（必要に応じて）
  - w-graph（トレーサビリティグラフ）
  - w-workflow（定型ワークフロー）
  - w-ruleset（ルールセットpack/unpack）
- **TODO:** vFinal.4 の「w-core-ledger〜w-cli」の具体内訳（責務/依存/公開API/入出力）をここへ取り込み。

---

## 9. 文書の固定点（運用規約）

- 本書は W の単一正本（統合SSOT）
- W-core（ledger/manifest/verify/audit_index）の破壊的変更は禁止
- 必要なら SemVer major＋移行手順＋互換維持でのみ許容

---

## 10. Capability Index（ID保持）

> 原文に ID（A-xxx / Sxx / T-xx / F-xx / Y-xx 等）が提示されていないため、以下は **ID未提示**として列挙する。  
> **TODO:** もし上位SSOTや別紙に ID が存在するなら、本Indexへ **原IDのまま**追記する。

### 10.1 Capabilities（ID未提示）
- Canonical Ledger（append-only）
- Truth/Idempotency（重複収束・近似同一・順不同耐性）
- Valuation/FX（snapshot/品質保証/手動手当）
- Cost Basis（ロット/平均単価説明可能性、端数/丸め固定、分割束ね）
- PnL派生（科目分離、Why-diff、Attribution、サニティガード）
- Tax Regime/Policy（日本優先、複数ビュー、what-if、収益区分推定支援）
- Journal/CoA（複式仕訳、仕訳ルール（テンプレ/DSL）、例外、プレビュー、期間ロック）
- Voucher（起票/承認/証憑紐づけ）
- Reconciliation/Adjustment（照合/収束、棚卸し、補正案、自動修正候補）
- Reports（数値＋文章、監査資料、比較、提出説明）
- Export（テンプレ/ルール、CSV安全化、CSV+JSON、差分）
- Documents ingestion（証憑取り込み/自動取得/更新履歴/突合）
- Evidence Pack + Verifier + Verify gates + 最小反例生成
- Audit Index + Chain of Custody + 提出状態管理
- Recovery（破損検知/自己修復、bit-rot検知）
- Drift検知 + Migration吸収
- Pending Tracker
- Edge-case辞書 + 資産カテゴリ会計プリセット
- 異常入力隔離ビュー
- CorporateActionEvent 一般化
- 逆引き検索（提出→台帳→Raw/証憑）
- トレーサビリティ・グラフ出力（HTML/Graphviz）
- 共同作業支援（注記B、税理士Q&Aテンプレ）
- 品質スコアリング + 品質ヒートマップ
- ルールセット共有（export/import）
- 定型ワークフロー（monthly-close / annual-submit-ready / refresh-docs-and-reconcile）
- diff意味付け（更正/価格更新/policy変更等）
- 提出前チェックの安全自動修正
- counterparty/アドレス辞書共有（export/import）
- サンプルデータ生成（テスト/説明用）
- 異常パターン検知カタログ（スリッページ等）
- データ品質ヒートマップ（期間×資産×venue）
- 税理士Q&A回答テンプレ生成

---

## 11. Level 2 Deep Spec 判定
- Non-negotiable：あり
- Canonical Model/Contract：**参照のみ（vFinal.4固定）で本文不足**
- Behavior/Tests：**Gates/最小反例/DoDはあるが、テスト仕様本文は不足**
- 結論：**入力に「Non-negotiable + Canonical Model/Contract + Behavior/Tests」が揃っていないため、Level 2 Deep Spec は出力しない。**
