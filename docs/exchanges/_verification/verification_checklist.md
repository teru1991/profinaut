# docs/exchanges Verification Checklist

確認日: <!-- YYYY-MM-DD -->
担当: <!-- name -->

## 0. 前提
- [ ] 対象取引所名を明記した
- [ ] 対象ドキュメントファイル（`docs/exchanges/<exchange>/...`）を列挙した
- [ ] 参照元は公式一次情報のみ（非公式は補助扱いで明示）
- [ ] 証拠ログに秘密情報（APIキー/署名元文字列の機密部分等）が含まれない

## 1. 公式一次情報の正当性（EXD-002）
- [ ] 公式サイトのトップから開発者ドキュメント導線を確認した
- [ ] 最終確定URLを記録した
- [ ] 確認日（YYYY-MM-DD）を記録した
- [ ] URLの配布形態（docsサブドメイン/GitBook/GitHub等）を明記した

## 2. カバレッジ（EXD-003）
### 2.1 REST/WS 領域
- [ ] Market Data（ticker/trades/orderbook/klines/instruments）
- [ ] Trading（new/cancel/amend/batch/status）
- [ ] Account（balances/positions/fees/leverage/margin）
- [ ] Transfer（deposit/withdraw/address/internal transfer）
- [ ] WS Public/Private（auth, reconnect, ping-pong）

### 2.2 運用要件
- [ ] Rate limit（weight/IP/key/ban/backoff）
- [ ] エラー体系（HTTP/独自コード/リトライ可否）
- [ ] 不足API・不要API・曖昧点を抽出した

## 3. 認証厳密性（EXD-004）
- [ ] 認証方式（HMAC/ED25519/JWT等）
- [ ] Required headers
- [ ] timestamp/nonceの単位と許容ズレ
- [ ] 署名対象文字列（method/path/query/body順序）
- [ ] body canonicalization（JSON整形影響）
- [ ] 公式サンプルとの差分を説明した（差分なし含む）

## 4. エンドポイント精査（EXD-005）
- [ ] P0（発注/取消/残高/板/ティッカー）を優先検証した
- [ ] 必須/任意・型・範囲・列挙値・単位・デフォルトを確認した
- [ ] 条件付き必須を明記した
- [ ] ページング仕様を明記した
- [ ] 返却フィールド意味（手数料通貨、数量単位、丸め）を確認した

## 5. スモークテスト（EXD-006）
- [ ] Public REST（最低 ticker/orderbook）
- [ ] Public WS（subscribe→受信→ping/pong）
- [ ] Private read-only（可能な範囲でbalance）
- [ ] 成功/失敗理由を evidence に保存した
- [ ] 秘匿情報をマスクした

## 6. 反映と追従（EXD-007/008）
- [ ] docs/exchanges の差分修正を反映した
- [ ] レポートに一次URL・確認日・スモーク結果を記録した
- [ ] index.md のステータスを更新した
- [ ] changelog/RSS/更新監視手順を記録した
