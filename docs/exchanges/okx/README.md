# OKX Official Docs Catalog (API v5)

このディレクトリは、OKX公式ドキュメント（docs-v5）を一次ソースとして API v5（REST/WS）のカタログ化を行うための成果物です。

## Scope
- API v5 REST / WebSocket
- Public / Private
- instType: SPOT / SWAP / FUTURES / OPTION
- Changelog（log / upcoming changes）から API/WS 影響差分抽出

## Files
- `sources.md`: 参照した公式URLと取得可否
- `rest-api.md`: RESTカタログ（001F形式）
- `websocket.md`: WSカタログ（001F形式）
- `data.md`: マーケットデータ配信整理
- `diffs.md`: Changelog差分（根拠URL付き）
- `catalog.json`: 機械可読カタログ
- `templates.md`: 001F行テンプレート
- `CHANGELOG.md`: このディレクトリ更新履歴

## Note
この実行環境では `okx.com` へのHTTPS接続がプロキシ制約（HTTP CONNECT 403）で遮断され、公式本文の自動取得ができませんでした。従って本成果物は「公式URLの列挙」と「取得不能の明示」を中心とした初期化状態です。
