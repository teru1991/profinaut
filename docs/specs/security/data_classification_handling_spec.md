# Data Classification & Handling Spec v1.0（固定）
Document ID: SEC-DATA-CLASS-SPEC
Status: Canonical / Fixed Contract
Scope: データ分類（public/internal/restricted）と取り扱い（ログ/監査/バンドル/エクスポート）を固定する

## 0. 目的（Non-negotiable）
事故の多くは「秘密/機微/個人情報の混入」と「誤共有」から起きる。
本仕様は、データを分類し、出力や共有で漏洩しない不変条件を固定する。

必達:
1) secret-free: secretsは絶対にログ/監査/バンドル/エクスポートへ出さない
2) least-privilege: 取り扱いは最小権限
3) evidence-friendly: 監査・再現のための証拠は残すが秘密は残さない
4) truthful: UNKNOWN/欠損を健康として表示しない

## 1. 分類（固定）
### 1.1 PUBLIC
- 公開しても問題ない情報
- 例: 一般統計（個別口座に紐づかない）、公開可能な仕様文書（secret無し）

### 1.2 INTERNAL
- 社内利用。外部公開は原則しない
- 例: 収集状況、SLO、内部メトリクス、個別取引に紐づかない集計

### 1.3 RESTRICTED
- 機微情報。漏洩が致命的
- 例:
  - APIキー/トークン/秘密鍵/署名素材（最重要）
  - 口座識別、注文詳細、約定詳細、残高、税務・申告データ
  - 個人情報（PII）になり得るもの
  - 支援バンドル内の環境情報（過度に詳細だと機微化）

固定ルール:
- RESTRICTED は “デフォルト非表示” で扱う
- RESTRICTED の export/共有は dangerous op になり得る（承認/監査必須）

## 2. 禁止情報（絶対に出してはいけない）
- secret類（api_key, secret, token, private_key, authorization, cookie 等）
- 署名鍵素材、復号可能な秘密
- 平文パスワード
固定ルール:
- これらは “redaction” の対象ではなく “出力禁止” とする（zero leakage）

## 3. ログ/監査/バンドル/エクスポート取り扱い（固定）
### 3.1 Logs
- 構造化ログ（JSON）を基本
- forbidden-key scan を通す（検知したら出力しない）
- RESTRICTED はログに出さない

### 3.2 Audit (audit_event)
- 重要イベントは残すが、値は最小化
- 例: order_id は外部IDでも機微になり得るため、必要最小限 + 参照（replay pointers）で辿れる設計
- “拒否” や “危険操作の確認” は必ず残す

### 3.3 Support Bundle
- 原則 INTERNAL 相当まで
- RESTRICTED が混入し得る場合は自動マスキング＋監査
- bundle生成は監査対象（誰が、何の範囲を、いつ）

### 3.4 Export / Reports
- exportは分類に従う
- RESTRICTED の export は admin/break-glass など制御し、監査必須
- 共有前に secret-free 検査を必須

## 4. PII（個人情報）扱い（固定）
- “PIIかもしれない” ものは RESTRICTED として扱う（安全側）
- 不明は UNKNOWN として扱い、外部共有は禁止

## 5. 例外（固定）
- 例外は break-glass のみ（期限/理由/監査必須）
- 例外であっても secret平文は出さない（例外なし）

## 6. DoD（検証）
最低限、以下を満たす:
1) forbidden-key scan が docs/ と bundle/export に適用される
2) secretが混入しない（ゼロ漏洩）
3) RESTRICTED は最小権限でのみ閲覧/出力できる
4) 重要操作は監査される（拒否も含む）

