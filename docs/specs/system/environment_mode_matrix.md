# Environment & Mode Matrix v1.0（固定）
Document ID: SYS-ENV-MODE-MATRIX
Status: Canonical / Fixed Contract
Scope: dev/stage/prod と paper/shadow/live の意味・禁止事項・隔離を固定する

## 0. 目的（Non-negotiable）
環境とモードの混同は、誤発注・誤削除・監査破綻の原因になる。
本仕様は「何がどこで許されるか」を固定し、誤爆を構造的に防ぐ。

## 1. 環境（Environment）固定定義
- dev: 開発。破壊OKだが証拠は残す。外部共有禁止が基本
- stage: 検証。本番相当の設定でテスト。live相当は原則禁止
- prod: 本番。最小権限。危険操作は強制的に challenge/confirm

固定ルール:
- env間でデータを交差させない（DB/バケット/名前空間を分離）
- envは常時UI表示（誤認防止）

## 2. モード（Mode）固定定義
- paper: 仮想。実発注なし。外部取引所への注文送信は禁止
- shadow: 実データで判断するが実発注なし（intentのみ）
- live: 実発注あり（最も危険）

固定ルール:
- liveへは暗黙遷移しない（必ず明示操作）
- live操作は dangerous op（challenge/confirm）
- SAFE/EMERGENCY_STOP 下は live は禁止（実行ブロック）

## 3. 許可マトリクス（固定概念）
### 3.1 実発注（orders）
- paper/shadow: 禁止（例外なし）
- live: 許可（ただし gate + risk + killswitch に従う）

### 3.2 データ破壊（削除/強制圧縮/復旧）
- dev: 許可され得る（ただし監査）
- stage/prod: 原則dangerous op（challenge/confirm）

### 3.3 export（外部共有）
- dev: 原則禁止（例外はbreak-glass）
- stage/prod: 分類に従い制御。RESTRICTEDは厳格

## 4. 表示要件（固定）
UI/レポート/CLIは最低限を常時表示:
- env（dev/stage/prod）
- mode（paper/shadow/live）
- safety_mode
- killswitch
- gate/integrity（UNKNOWNを健康表示しない）
- data freshness（staleを最新扱いしない）

## 5. DoD（検証）
最低限:
1) env/mode が常時表示される
2) live誤爆がUI/API両方で止まる（challenge/confirm + gate）
3) env間データが交差しない（名前空間分離が前提）
4) SAFE/EMERGENCY_STOP で実行が止まる

