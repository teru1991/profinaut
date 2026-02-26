分類理由: 配布/パッケージ/運用設計に関する機能要求のSSOTであり、保存先指針の system に該当するため。
# Level 1 SSOT Outline — Domain X（Productization / Distribution）
Source: “X. Productization / Distribution — Feature Catalog v1.3 (SSOT)” :contentReference[oaicite:0]{index=0}

## 0. SSOT Metadata
- Domain: X（Productization / Distribution）
- Document Type: Feature Catalog（機能要求の網羅SSOT）
- Status: CANONICAL（SSOT） :contentReference[oaicite:1]{index=1}
- Version: 1.3.0 :contentReference[oaicite:2]{index=2}
- Last-Updated: 2026-02-26 :contentReference[oaicite:3]{index=3}

## 0.1 Purpose（目的）
- ドメインX（製品化/配布）が提供すべき **機能要求（Feature Requirements）** を最初から最後まで網羅して列挙するSSOT。 :contentReference[oaicite:4]{index=4}
- 対象は運用ルールではなく、ユーザー/システムが利用する機能。 :contentReference[oaicite:5]{index=5}
- セキュリティ/堅牢性の原則は別SSOTで規定され、本書は機能の見取り図として独立して読める。 :contentReference[oaicite:6]{index=6}
- 必須/推奨の優先度は実装計画で決める（本書は存在すべき機能の列挙）。 :contentReference[oaicite:7]{index=7}

## 0.2 Non-negotiables
TODO: 本入力には「Non-negotiable」章が明示されていないため、別SSOT（X Core Spec / Policy 等）から抽出して追記する。 :contentReference[oaicite:8]{index=8}

## 0.3 Canonical Model / Contract
TODO: 本入力にはモデル/契約（Reason Codes / CLI Contract / Schema 等）の本文が無いため、関連SSOT（X Schemas / Reason Codes / CLI Contract）を参照して追記する。 :contentReference[oaicite:9]{index=9}

## 0.4 Behavior / Tests
TODO: 本入力にはテスト/検証（Behavior/Tests）章が無いため、別途の検証仕様（例: rehearse/audit の期待結果、成功条件、失敗分類）を収集して追記する。 :contentReference[oaicite:10]{index=10}

---

## 1. Capability Index（ID保持）
- 既存の番号/ID（A-xxx / Sxx / T-xx / F-xx / Y-xx 等）は本入力中に明示がない。 :contentReference[oaicite:11]{index=11}
- TODO: 既存IDが別文書に存在する場合、それらを回収して本Indexへ追加する（推測で新規IDは発行しない）。

---

## 2. Capabilities（機能要求の整理）
> 見出し（章/節）は 1.x のカテゴリとして保持し、各項目は “提供すべき機能” を列挙する。 :contentReference[oaicite:12]{index=12}

### 2.1 インストール／更新の多形態サポート
#### 2.1.1 配布形態
- ZIP（ポータブル）配布
- MSI（Windows Installer）配布
- EXE（セットアップ）配布
- 形態が異なっても Update Manager / License / Diagnostics は同一の中核を利用する
- （上級）アップデータ/インストーラ自身の自己更新（22章） :contentReference[oaicite:13]{index=13}

#### 2.1.2 無人導入・無人更新
- サイレントインストール（無人導入）
- サイレント更新（無人更新）
- スケジュール更新（夜間/アイドル時など）
- “承認が必要な更新”は無人では適用されない（承認フローと連携） :contentReference[oaicite:14]{index=14}

#### 2.1.3 システム要件チェック
- OS/arch/権限/ディスク/ネットワーク基本チェック
- 競合プロセス（実行中の更新対象）検知
- セキュリティ製品（AV等）によるアクセス拒否の検知とガイド
- 必須依存（ランタイム等）がある場合の検出 :contentReference[oaicite:15]{index=15}

#### 2.1.4 アンインストール
- 設定保持アンインストール
- 完全削除アンインストール（危険操作として保護）
- 残骸（versions/stage/state/logs 等）の安全な掃除導線
- “復旧だけしたい”場合のアンインストール抑止（ガイド） :contentReference[oaicite:16]{index=16}

---

### 2.2 Distribution Sources（更新入力ソース）
#### 2.2.1 取得元の多様化
- インターネット配布（通常）
- ローカルファイル更新（USB/共有フォルダ）
- LAN内ミラー更新（NAS/社内HTTP等）
- 固定URL更新（pin用途）
- オフライン更新リポジトリ（USB/NAS用のミニindex）
- 署名検証済みキャッシュ（検証済み成果物の安全再利用） :contentReference[oaicite:17]{index=17}

#### 2.2.2 ミラー機能
- 複数ミラー対応（フェイルオーバ）
- 自動最適化（成功率/速度に基づく選択）
- ミラー健全性の計測（ランキング/履歴）
- ミラーの事前検証（ヘルスチェック） :contentReference[oaicite:18]{index=18}

#### 2.2.3 ネットワーク環境対応
- 帯域制限（スロットリング）
- 時間帯制限（schedulerと統合）
- プロキシ対応（企業環境）
- 証明書/CAの取り扱い導線（企業CA等）
- ネットワーク不安定前提の堅牢再試行（分類された失敗理由＋フェイルオーバ） :contentReference[oaicite:19]{index=19}

#### 2.2.4 成果物の検疫（Quarantine：上級）
- ダウンロード済み成果物を “隔離” し、手動承認まで適用しない
- 隔離中の再検証/再スキャン
- 隔離解除の監査（操作ログと連携） :contentReference[oaicite:20]{index=20}

---

### 2.3 軽量更新・分割更新
#### 2.3.1 差分更新（delta/patch）
- 差分更新導入
- 失敗時の full フォールバック
- 適用後整合性チェック（差分適用の不整合検出）
- 差分の署名・整合性保証（fullと同等の信頼） :contentReference[oaicite:21]{index=21}

#### 2.3.2 コンポーネント単位更新
- UIのみ/collectorのみ等の部分更新
- 依存順序と互換マトリクスに従う
- 部分更新の成功/失敗が履歴と診断に反映される :contentReference[oaicite:22]{index=22}

#### 2.3.3 優先度付き更新
- security fixes 優先適用
- 機能改善・UI更新は後回し
- “適用の優先度ルール”をユーザーが選べる :contentReference[oaicite:23]{index=23}

#### 2.3.4 段階ダウンロード／プリフェッチ
- 検証に必要な最小データを先に取得し早期reject
- 更新候補の“ダウンロードだけ先行”（プリフェッチ）
- プリフェッチ時点で署名検証まで完了できる :contentReference[oaicite:24]{index=24}

---

### 2.4 移行支援（Migration Assistant）
#### 2.4.1 推奨順序の提示
- update plan に「推奨更新順序」「前提条件」「互換性判定」を出す
- DB migration や contract update が必要なら順序を自動提示 :contentReference[oaicite:25]{index=25}

#### 2.4.2 設定自動変換（Adapter）
- config schema の自動変換
- 変換後検証（失敗時は旧設定で起動可能 or 更新拒否）
- 変換の“何がどう変わったか”を差分として提示 :contentReference[oaicite:26]{index=26}

#### 2.4.3 “戻せない更新”支援
- not-rollbackable の強い警告
- 事前チェックリスト生成（バックアップ/停止/容量/影響など）
- 手動適用の導線（危険操作として保護）
- 適用後の検証項目（post-check）提示 :contentReference[oaicite:27]{index=27}

#### 2.4.4 互換性エラーの自動案内（Migration Advisor）
- compat NG のとき「先に何を上げるべきか」「この順序でやれ」を案内
- plan/what-if に反映 :contentReference[oaicite:28]{index=28}

---

### 2.5 Preflight / Self-check / Audit（事前確認と自己診断）
#### 2.5.1 常設監査（Audit）
- `audit install`（常設）
- UI/CLIから到達可能
- fail/warn/pass と修正手順の提示 :contentReference[oaicite:29]{index=29}

#### 2.5.2 事前ベンチ・健康診断
- ディスクIO簡易測定（閾値未満でwarn/blocked）
- ネットワーク簡易測定（遅延・失敗率でwarn）
- プロキシ/CA設定の整合点検 :contentReference[oaicite:30]{index=30}

#### 2.5.3 外部依存プローブ（Deps Probes）
- deps gate のプローブを手動実行できる
- 依存サービスの“自動検出”（設定からDB/MinIO/Kafka等を推定）※任意
- プローブ仕様の統一（timeout/成功条件/赤塗りルール） :contentReference[oaicite:31]{index=31}

#### 2.5.4 Policy Explain（拒否理由の説明）
- “なぜ拒否されたか”の説明
- “どのルールが発火したか”の提示
- “例外を使う場合に必要な条件”の提示
- スヌーズ/hold/deny/freeze 等の“ブロック理由”の可視化 :contentReference[oaicite:32]{index=32}

#### 2.5.5 事前チェックリスト自動生成
- リリースごとに適用前のチェックリストを生成
- plan に埋め込み／印刷用・共有用に出力 :contentReference[oaicite:33]{index=33}

---

### 2.6 状態表示・UX（操作性）
#### 2.6.1 進捗ストリーミング
- download/verify/stage/commit の進捗を連続表示
- 機械可読JSONを正とし、UIはその表現 :contentReference[oaicite:34]{index=34}

#### 2.6.2 plan/status の見やすさ
- `update plan`（dry-run）の詳細表示
- `update status` の機械可読出力
- What-if（適用せずに総合評価：互換/移行/依存/ポリシーまで） :contentReference[oaicite:35]{index=35}

#### 2.6.3 対話型ガイド／オンデバイスHelp
- 理由コード→推奨手順のガイド（安全側の提示）
- よくある理由コードのオンデバイスFAQ
- オンデバイスhelp（復旧手順内蔵）
- チュートリアルモード（初回だけ案内） :contentReference[oaicite:36]{index=36}

#### 2.6.4 復旧ウィザード
- safe/maintenance からの復旧ウィザード
- “次にやるべきこと”を段階で提示 :contentReference[oaicite:37]{index=37}

#### 2.6.5 ローカライズ／アクセシビリティ（任意）
- 日本語/英語切替
- エラー・ガイドの可読性向上 :contentReference[oaicite:38]{index=38}

#### 2.6.6 状態タイムライン（上級）
- 更新/拒否/承認/スヌーズ/deny/freeze/hold を時系列で表示
- 各イベントに report（audit/diag/rehearse/migration）へのリンク :contentReference[oaicite:39]{index=39}

---

### 2.7 ロールバック／復旧（拡張）
#### 2.7.1 小スナップショット（最小復旧点）
- 更新直前の最小バックアップ（設定/メタデータ等）
- バックアップ目録（何を保存したか） :contentReference[oaicite:40]{index=40}

#### 2.7.2 セーフバックアップ（製品機能化）
- 復元ウィザード
- 復元後検証
- スナップショット世代管理（設定全体のスナップショット：秘密除外） :contentReference[oaicite:41]{index=41}

#### 2.7.3 復旧パッケージ
- 必要最小ファイルをまとめた復旧パッケージ生成
- “復旧用ブートストラップ”パッケージ（USBで持ち運べる復旧ツール） :contentReference[oaicite:42]{index=42}

#### 2.7.4 緊急起動
- 前バージョンで “起動だけ” できる導線
- 操作不能状態からの脱出を最優先 :contentReference[oaicite:43]{index=43}

#### 2.7.5 anti-repeat hold
- ロールバック後に同じ更新を繰り返さない抑止
- hold の理由と解除条件の可視化 :contentReference[oaicite:44]{index=44}

#### 2.7.6 ロールバックの粒度向上
- コンポーネント単位ロールバック（UIだけ戻す等）
- 段階ロールバック（依存順序に従う）
- データ互換ガード付きロールバック（危険なら拒否＋代替案提示） :contentReference[oaicite:45]{index=45}

#### 2.7.7 訓練機能
- `rollback rehearse`（実際には戻さず、整合性と手順だけ確認） :contentReference[oaicite:46]{index=46}

#### 2.7.8 ロールバック後の自動診断生成（上級）
- ロールバック発生時に自動で以下をまとめて生成：
  - support bundle
  - rehearsal report（可能なら）
  - before/after diff :contentReference[oaicite:47]{index=47}

---

### 2.8 セキュリティ運用機能（配布機能として）
#### 2.8.1 鍵・ポリシー状態の可視化
- 鍵セット状態（ローテ/失効）
- trust root 状態（pinning/更新状態）
- 署名検証の状態・結果の表示
- （上級）署名検証の二重化状態の可視化 :contentReference[oaicite:48]{index=48}

#### 2.8.2 deny/containment/freeze/hold/snooze の可視化
- deny list 適用状況
- containment（封鎖）状態
- freeze 状態
- hold / snooze 状態 :contentReference[oaicite:49]{index=49}

#### 2.8.3 tamper / integrity
- インストール改ざん検知（auditに統合）
- versions/current/stage/state の整合点検
- 不整合の自動復旧（安全側：掃除/隔離/戻し）※任意 :contentReference[oaicite:50]{index=50}

#### 2.8.4 セキュリティ要約通知
- ゲート失敗、deny一致、safe-mode突入等の通知（ローカル通知から） :contentReference[oaicite:51]{index=51}

#### 2.8.5 “壊れた更新”回避機能
- bad release deny list による自動回避
- deny一致時の代替候補提示（なければblocked） :contentReference[oaicite:52]{index=52}

#### 2.8.6 成果物検証ツール単体配布（上級）
- オフライン検証や別PC検証用に `verify-artifact` ツールを単体配布 :contentReference[oaicite:53]{index=53}

---

### 2.9 診断・サポート出力
#### 2.9.1 rehearsal（切替なし検証）
- stage/verify/deps-probe まで実行
- 切替はしない
- rehearsal report を出力 :contentReference[oaicite:54]{index=54}

#### 2.9.2 before/after diff
- 更新前後の差分（config/capabilities/主要メトリクス要約）
- 設定差分の正規化（同じ設定なら同じdiff） :contentReference[oaicite:55]{index=55}

#### 2.9.3 support ticket code
- support ticket code 自動発行
- instance_id / target_version と紐付け
- 履歴・診断・レポートに必ず追跡可能にする :contentReference[oaicite:56]{index=56}

#### 2.9.4 エクスポート／アーカイブ
- update history / audit / rehearsal / migration report のまとめ出力（秘密なし）
- 証跡保存・別PC移行の導線 :contentReference[oaicite:57]{index=57}

#### 2.9.5 ワンクリック統合
- 「診断→rehearse→plan」を一連で実行するショートカット :contentReference[oaicite:58]{index=58}

#### 2.9.6 Capabilities/Entitlements の診断連携（上級）
- capabilities一覧と無効理由（ライセンス/環境/縮退）を診断へ含める :contentReference[oaicite:59]{index=59}

---

### 2.10 複数インスタンス／複数環境（Fleet）
#### 2.10.1 インスタンス一覧
- instance_id/バージョン/状態/環境 の一覧 :contentReference[oaicite:60]{index=60}

#### 2.10.2 グルーピング
- primary/backup/dev などグループ化 :contentReference[oaicite:61]{index=61}

#### 2.10.3 一括操作
- 一括 plan / 一括 apply（グループ単位）
- 依存順序と互換を尊重 :contentReference[oaicite:62]{index=62}

#### 2.10.4 fleet pin / 揃え込み
- 全台を特定バージョンに統一（fleet pin）
- 解除も監査可能に :contentReference[oaicite:63]{index=63}

#### 2.10.5 差分検知
- どれがどの版か
- 設定差分（秘密除外）も検知可能 :contentReference[oaicite:64]{index=64}

#### 2.10.6 二重起動支援
- backup→観察→primary の導線
- 同一ソースで同一成果物を取得する補助（ミラー固定等） :contentReference[oaicite:65]{index=65}

#### 2.10.7 Primary/Backup 整合性チェッカー（上級）
- primary と backup の版差・設定差（秘密除外）・互換（DB等）を一括評価
- “今切り替えて大丈夫か”判定を出す :contentReference[oaicite:66]{index=66}

---

### 2.11 オフライン/隔離環境（Air-gap / Offline）
#### 2.11.1 ローカル更新repo生成
- USB/NAS用に “ミニindex + manifest群 + 署名” を生成 :contentReference[oaicite:67]{index=67}

#### 2.11.2 検証済みキャッシュ
- 一度検証した成果物を安全に再利用
- 改ざん検知つき :contentReference[oaicite:68]{index=68}

#### 2.11.3 検証レポート搬出
- 隔離環境から外へ出せる “最小証跡” を出力 :contentReference[oaicite:69]{index=69}

---

### 2.12 リリース管理UX（比較・検索・ロールアウト）
#### 2.12.1 リリース比較
- current vs candidate の差分表示（互換/影響/推定時間） :contentReference[oaicite:70]{index=70}

#### 2.12.2 ロールアウト状況可視化
- 段階配布の進捗・成功率 :contentReference[oaicite:71]{index=71}

#### 2.12.3 注釈（運用メモ）
- ローカル注釈保存（自分用） :contentReference[oaicite:72]{index=72}

#### 2.12.4 変更点検索/フィルタ
- breaking/security/ops-impact/known-issues で検索・絞り込み :contentReference[oaicite:73]{index=73}

#### 2.12.5 リリース制約の可視化
- “prod不可/手動のみ/戻せない”等の制約表示 :contentReference[oaicite:74]{index=74}

#### 2.12.6 互換マトリクス可視化（上級）
- core/collector/dashboard/plugins の互換マトリクスをUIで可視化
- plan で理由付き説明 :contentReference[oaicite:75]{index=75}

---

### 2.13 更新の選択制（承認・スヌーズ・優先度）
#### 2.13.1 スヌーズ
- 一定期間の更新抑止
- 期限・理由・解除条件を持つ :contentReference[oaicite:76]{index=76}

#### 2.13.2 承認フロー
- plan生成→approve→apply
- 承認の監査ログを残す :contentReference[oaicite:77]{index=77}

#### 2.13.3 優先度ポリシー
- securityのみ自動、機能は手動 等 :contentReference[oaicite:78]{index=78}

#### 2.13.4 バックグラウンド検証
- 定期rehearse（適用は後） :contentReference[oaicite:79]{index=79}

#### 2.13.5 アップデート・カレンダー
- scheduler窓、スヌーズ期限、保留中承認、次のLTS候補等を表示 :contentReference[oaicite:80]{index=80}

---

### 2.14 データ移行の見える化
#### 2.14.1 移行進捗
- DB migration等の段階・推定表示 :contentReference[oaicite:81]{index=81}

#### 2.14.2 移行プレビュー
- 影響範囲/ロック時間/容量 :contentReference[oaicite:82]{index=82}

#### 2.14.3 移行後検証
- 整合性チェック（サンプル検証等） :contentReference[oaicite:83]{index=83}

#### 2.14.4 移行レポート
- 解析用レポート出力（support bundle同梱可） :contentReference[oaicite:84]{index=84}

---

### 2.15 プラグイン/拡張の配布（将来拡張）
#### 2.15.1 プラグイン配布チャネル
- 本体と別manifest/index :contentReference[oaicite:85]{index=85}

#### 2.15.2 互換性ゲート
- ABI/契約バージョンで判定 :contentReference[oaicite:86]{index=86}

#### 2.15.3 署名検証
- 本体同等の署名必須 :contentReference[oaicite:87]{index=87}

#### 2.15.4 無効化/隔離
- プラグインの安全な無効化/隔離（safe-mode連携） :contentReference[oaicite:88]{index=88}

---

### 2.16 タイムライン・影響推定・通知・訓練（追加機能）
#### 2.16.1 変更影響の自動推定
- 推定ダウンタイム、推定ディスク消費、推定負荷（目安）を plan に載せる :contentReference[oaicite:89]{index=89}

#### 2.16.2 What-if（適用せずに総合評価）
- 互換性、移行、依存、ポリシーまで含めて “適用せず評価” する :contentReference[oaicite:90]{index=90}

#### 2.16.3 ヘルス基準の推奨
- commit health criteria の推奨値（提案のみ） :contentReference[oaicite:91]{index=91}

#### 2.16.4 エスカレーション通知
- safe-mode突入、deny一致、連続失敗、承認待ち等の通知（まずローカル） :contentReference[oaicite:92]{index=92}

#### 2.16.5 Canary Simulation（上級）
- 実適用前に重要シナリオだけ自動チェックし、疑似カナリア判定を出す :contentReference[oaicite:93]{index=93}

---

### 2.17 パッケージ最適化（高速化/安定化）
#### 2.17.1 圧縮・解凍の最適化
- 更新時間を短縮する最適化（圧縮形式/分割など） :contentReference[oaicite:94]{index=94}

#### 2.17.2 セキュリティ製品との相性改善
- 典型的に引っかかりやすい構成を避ける（ビルド時最適化） :contentReference[oaicite:95]{index=95}

---

### 2.18 設定テンプレ／設定管理の高度化
#### 2.18.1 テンプレのバージョニング
- テンプレ自体にversion付与
- 適用履歴、差分、ロールバック :contentReference[oaicite:96]{index=96}

#### 2.18.2 テンプレ配布
- dev/stg/prod テンプレ配布
- fleet向けテンプレ適用（グループ単位） :contentReference[oaicite:97]{index=97}

#### 2.18.3 実行中タスクの安全停止（任意）
- 更新前に稼働中タスク（重い処理等）があれば 一時停止→更新→再開の導線 :contentReference[oaicite:98]{index=98}

---

### 2.19 リリース自動評価（安全度スコア）
- Gate結果、過去成功率、deny/rollback履歴等から “安全度スコア” を算出
- plan/比較画面に表示（判断材料） :contentReference[oaicite:99]{index=99}

---

### 2.20 Capabilities / Entitlements 可視化（上級）
- 現在の capabilities（有効機能）一覧表示
- 無効理由の表示（ライセンス/環境/安全縮退）
- （任意）安全縮退の一部を手動で解除/再適用する導線（監査ログ必須） :contentReference[oaicite:100]{index=100}

---

### 2.21 操作ログ（上級）
- approve / apply / rollback / snooze / pin / fleet pin / quarantine解除 等の操作を“操作ログ”として可視化
- 履歴と別に “人が何をしたか” を追える :contentReference[oaicite:101]{index=101}

---

### 2.22 自己更新（上級）
- Update Manager / Installer 自体を安全に更新する
- 二段階で入れ替え、失敗時は旧アップデータへ戻す
- 自己更新の plan / rehearse / rollback が成立する :contentReference[oaicite:102]{index=102}

---

### 2.23 Appendix: Ultra-Advanced Features（超上級ニッチ機能）
#### 2.23.1 影響範囲の自動検出（Change Surface Detection）
- 新旧成果物差分から、変更されたバイナリ/設定/スキーマ/影響コンポーネントを自動推定して plan に表示 :contentReference[oaicite:103]{index=103}

#### 2.23.2 安全なA/B（Parallel Run / Shadow Traffic）
- 同一マシンで新旧を並行起動し、read-only/shadowで比較する（資源が許す範囲） :contentReference[oaicite:104]{index=104}

#### 2.23.3 継続監査（Continuous Audit）
- audit install を定期実行し、drift/権限変更/改ざん/allowlist逸脱などを検知して通知 :contentReference[oaicite:105]{index=105}

#### 2.23.4 ロールバック自動提案（Auto-rollback Advisor）
- エラー率/失敗傾向からロールバック推奨や優先チェックを提案（自動実行はしない） :contentReference[oaicite:106]{index=106}

#### 2.23.5 復旧USB作成（Recovery Media Builder）
- 復旧用ブートストラップをUSBイメージとして生成（検証ツールや最小証跡も同梱可能） :contentReference[oaicite:107]{index=107}

#### 2.23.6 配布前検証（Pre-pack Verify）
- 配布前にレイアウト/署名/SBOM整合/危険パターン検出を自動で実行し、事故リリースを減らす :contentReference[oaicite:108]{index=108}

#### 2.23.7 軽量インベントリ（Inventory Export）
- 各インスタンスの version/capabilities/最終更新日/健全性 をCSV/JSONでエクスポート :contentReference[oaicite:109]{index=109}

#### 2.23.8 最小停止最適化（Zero-downtime-ish）
- UI無停止・collector短停止など、コンポーネント単位で停止時間を最小化するモード :contentReference[oaicite:110]{index=110}

---

## 3. Related SSOT / References（関連SSOT）
- X Core Spec / Policy / Plan（安全・堅牢の原則）
- X Addendum（Time Robustness / Deny List / Integrity Audit / Privilege Model）
- X Schemas / Reason Codes / CLI Contract（別途用意する場合） :contentReference[oaicite:111]{index=111}

---

## 4. TODO / Gaps（不足の明示）
- TODO: 本ドメインXの Non-negotiables を別SSOTから抽出し、本書 0.2 に確定記載する。 :contentReference[oaicite:112]{index=112}
- TODO: Reason Codes / CLI Contract / Schema（契約面）を回収し、0.3 と各カテゴリに紐付ける。 :contentReference[oaicite:113]{index=113}
- TODO: Behavior/Tests（受入条件・期待結果・失敗分類のテスト観点）を回収し、0.4 に整理する。 :contentReference[oaicite:114]{index=114}
- TODO: 本入力外に存在する可能性のあるID（A-xxx等）を探索し、Capability Index に追記する（推測で発行しない）。 :contentReference[oaicite:115]{index=115}
