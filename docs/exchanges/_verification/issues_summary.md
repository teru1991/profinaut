# Issues Summary

- Confirmed date: 2026-02-21
- Scope: 全20取引所の机上監査（EXD-B）

## Cross-Exchange Findings (P0/P1/P2)

| Exchange | Severity | Issue | Impact | Fix target files | Evidence URL |
|---|---|---|---|---|---|
| `binance` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/binance/rest-api.md`, `docs/exchanges/binance/websocket.md` | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/request-security |
| `binance` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/binance/websocket.md` | https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/general-api-information |
| `binance` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/binance/rest-api.md`, `docs/exchanges/binance/README.md` | https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits |
| `binance` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/binance/rest-api.md`, `docs/exchanges/binance/data.md` | https://developers.binance.com/docs/binance-spot-api-docs/rest-api |
| `binance` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/binance/README.md`, `docs/exchanges/binance/sources.md` | https://developers.binance.com/docs/binance-spot-api-docs/changelog |
| `binance-coinm` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/binance-coinm/rest-api.md`, `docs/exchanges/binance-coinm/websocket.md` | https://developers.binance.com/docs/derivatives/coin-margined-futures |
| `binance-coinm` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/binance-coinm/websocket.md` | https://developers.binance.com/docs/derivatives/coin-margined-futures/websocket-market-streams |
| `binance-coinm` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/binance-coinm/rest-api.md`, `docs/exchanges/binance-coinm/README.md` | https://developers.binance.com/docs/derivatives/coin-margined-futures |
| `binance-coinm` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/binance-coinm/rest-api.md`, `docs/exchanges/binance-coinm/data.md` | https://developers.binance.com/docs/derivatives/coin-margined-futures/rest-api |
| `binance-coinm` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/binance-coinm/README.md`, `docs/exchanges/binance-coinm/sources.md` | https://developers.binance.com/docs/derivatives/change-log |
| `binance-options` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/binance-options/rest-api.md`, `docs/exchanges/binance-options/websocket.md` | https://developers.binance.com/docs/derivatives/option |
| `binance-options` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/binance-options/websocket.md` | https://developers.binance.com/docs/derivatives/option/websocket-market-streams |
| `binance-options` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/binance-options/rest-api.md`, `docs/exchanges/binance-options/README.md` | https://developers.binance.com/docs/derivatives/option |
| `binance-options` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/binance-options/rest-api.md`, `docs/exchanges/binance-options/data.md` | https://developers.binance.com/docs/derivatives/option/market-data/rest-api |
| `binance-options` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/binance-options/README.md`, `docs/exchanges/binance-options/sources.md` | https://developers.binance.com/docs/derivatives/change-log |
| `binance-usdm` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/binance-usdm/rest-api.md`, `docs/exchanges/binance-usdm/websocket.md` | https://developers.binance.com/docs/derivatives/usds-margined-futures |
| `binance-usdm` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/binance-usdm/websocket.md` | https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams |
| `binance-usdm` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/binance-usdm/rest-api.md`, `docs/exchanges/binance-usdm/README.md` | https://developers.binance.com/docs/derivatives/usds-margined-futures |
| `binance-usdm` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/binance-usdm/rest-api.md`, `docs/exchanges/binance-usdm/data.md` | https://developers.binance.com/docs/derivatives/usds-margined-futures/rest-api |
| `binance-usdm` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/binance-usdm/README.md`, `docs/exchanges/binance-usdm/sources.md` | https://developers.binance.com/docs/derivatives/change-log |
| `bitbank` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bitbank/rest-api.md`, `docs/exchanges/bitbank/websocket.md` | https://github.com/bitbankinc/bitbank-api-docs |
| `bitbank` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bitbank/websocket.md` | https://github.com/bitbankinc/bitbank-api-docs |
| `bitbank` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bitbank/rest-api.md`, `docs/exchanges/bitbank/README.md` | https://github.com/bitbankinc/bitbank-api-docs |
| `bitbank` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bitbank/rest-api.md`, `docs/exchanges/bitbank/data.md` | https://github.com/bitbankinc/bitbank-api-docs/blob/master/rest-api.md |
| `bitbank` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bitbank/README.md`, `docs/exchanges/bitbank/sources.md` | https://github.com/bitbankinc/bitbank-api-docs/blob/master/CHANGELOG.md |
| `bitflyer` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bitflyer/rest-api.md`, `docs/exchanges/bitflyer/websocket.md` | https://bf-lightning-api.readme.io/docs/auth |
| `bitflyer` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bitflyer/websocket.md` | https://bitflyer.com/ja-jp/api |
| `bitflyer` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bitflyer/rest-api.md`, `docs/exchanges/bitflyer/README.md` | https://bf-lightning-api.readme.io/docs/rate-limit |
| `bitflyer` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bitflyer/rest-api.md`, `docs/exchanges/bitflyer/data.md` | https://bitflyer.com/ja-jp/api |
| `bitflyer` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bitflyer/README.md`, `docs/exchanges/bitflyer/sources.md` | https://bitflyer.com/ja-jp/api |
| `bitget` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bitget/rest-api.md`, `docs/exchanges/bitget/websocket.md` | https://www.bitget.com/api-doc/common/intro |
| `bitget` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bitget/websocket.md` | https://www.bitget.com/api-doc/common/websocket-intro |
| `bitget` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bitget/rest-api.md`, `docs/exchanges/bitget/README.md` | https://www.bitget.com/api-doc/common/intro |
| `bitget` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bitget/rest-api.md`, `docs/exchanges/bitget/data.md` | https://www.bitget.com/api-doc/common/intro |
| `bitget` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bitget/README.md`, `docs/exchanges/bitget/sources.md` | https://www.bitget.com/api-doc/common/changelog |
| `bithumb` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/websocket.md` | https://apidocs.bithumb.com/ |
| `bithumb` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bithumb/websocket.md` | https://apidocs.bithumb.com/reference/WEBSOCKET |
| `bithumb` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/README.md` | https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4 |
| `bithumb` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/data.md` | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| `bithumb` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bithumb/README.md`, `docs/exchanges/bithumb/sources.md` | https://apidocs.bithumb.com/changelog |
| `bitmex` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bitmex/rest-api.md`, `docs/exchanges/bitmex/websocket.md` | https://www.bitmex.com/app/apiKeys |
| `bitmex` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bitmex/websocket.md` | https://www.bitmex.com/app/wsAPI |
| `bitmex` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bitmex/rest-api.md`, `docs/exchanges/bitmex/README.md` | https://www.bitmex.com/api/explorer/ |
| `bitmex` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bitmex/rest-api.md`, `docs/exchanges/bitmex/data.md` | https://www.bitmex.com/api/explorer/ |
| `bitmex` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bitmex/README.md`, `docs/exchanges/bitmex/sources.md` | https://www.bitmex.com/app/apiChangelog |
| `bittrade` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/websocket.md` | https://www.bittrade.co.jp/ |
| `bittrade` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bittrade/websocket.md` | https://api-doc.bittrade.co.jp/#websocket-public |
| `bittrade` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/README.md` | https://www.bittrade.co.jp/ |
| `bittrade` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/data.md` | https://api-doc.bittrade.co.jp/#rest-api |
| `bittrade` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bittrade/README.md`, `docs/exchanges/bittrade/sources.md` | https://www.bittrade.co.jp/ |
| `bybit` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/bybit/rest-api.md`, `docs/exchanges/bybit/websocket.md` | https://bybit-exchange.github.io/docs/v5/websocket/wss-authentication |
| `bybit` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/bybit/websocket.md` | https://bybit-exchange.github.io/docs/v5/websocket/wss-authentication |
| `bybit` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/bybit/rest-api.md`, `docs/exchanges/bybit/README.md` | https://bybit-exchange.github.io/docs/v5/intro |
| `bybit` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/bybit/rest-api.md`, `docs/exchanges/bybit/data.md` | https://bybit-exchange.github.io/docs/api-explorer/v5/category/ |
| `bybit` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/bybit/README.md`, `docs/exchanges/bybit/sources.md` | https://bybit-exchange.github.io/docs/changelog/v5 |
| `coinbase` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/websocket.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `coinbase` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/coinbase/websocket.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket |
| `coinbase` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/README.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `coinbase` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/data.md` | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| `coinbase` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/coinbase/README.md`, `docs/exchanges/coinbase/sources.md` | https://docs.cdp.coinbase.com/coinbase-app/introduction/changelog |
| `coincheck` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/coincheck/rest-api.md`, `docs/exchanges/coincheck/websocket.md` | https://coincheck.com/ja/documents/exchange/api#auth |
| `coincheck` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/coincheck/websocket.md` | https://coincheck.com/ja/documents/exchange/api#websocket |
| `coincheck` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/coincheck/rest-api.md`, `docs/exchanges/coincheck/README.md` | https://coincheck.com/ja/ |
| `coincheck` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/coincheck/rest-api.md`, `docs/exchanges/coincheck/data.md` | https://coincheck.com/ja/ |
| `coincheck` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/coincheck/README.md`, `docs/exchanges/coincheck/sources.md` | https://coincheck.com/ja/ |
| `deribit` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/deribit/rest-api.md`, `docs/exchanges/deribit/websocket.md` | https://docs.deribit.com/api-reference/authentication/public-auth |
| `deribit` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/deribit/websocket.md` | https://docs.deribit.com/ |
| `deribit` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/deribit/rest-api.md`, `docs/exchanges/deribit/README.md` | https://docs.deribit.com/articles/rate-limits |
| `deribit` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/deribit/rest-api.md`, `docs/exchanges/deribit/data.md` | https://docs.deribit.com/api-reference |
| `deribit` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/deribit/README.md`, `docs/exchanges/deribit/sources.md` | https://docs.deribit.com/ |
| `gmocoin` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/gmocoin/rest-api.md`, `docs/exchanges/gmocoin/websocket.md` | https://api.coin.z.com/docs/ |
| `gmocoin` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/gmocoin/websocket.md` | https://api.coin.z.com/docs/#/ws |
| `gmocoin` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/gmocoin/rest-api.md`, `docs/exchanges/gmocoin/README.md` | https://api.coin.z.com/docs/ |
| `gmocoin` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/gmocoin/rest-api.md`, `docs/exchanges/gmocoin/data.md` | https://api.coin.z.com/docs/ |
| `gmocoin` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/gmocoin/README.md`, `docs/exchanges/gmocoin/sources.md` | https://api.coin.z.com/docs/ |
| `htx` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/htx/rest-api.md`, `docs/exchanges/htx/websocket.md` | https://www.htx.com/en-us/opend/newApiPages/ |
| `htx` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/htx/websocket.md` | https://www.htx.com/en-us/opend/newApiPages/ |
| `htx` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/htx/rest-api.md`, `docs/exchanges/htx/README.md` | https://www.htx.com/en-us/opend/newApiPages/ |
| `htx` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/htx/rest-api.md`, `docs/exchanges/htx/data.md` | https://www.htx.com/en-us/opend/newApiPages/ |
| `htx` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/htx/README.md`, `docs/exchanges/htx/sources.md` | https://www.htx.com/en-us/opend/newApiPages/ |
| `kraken` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/kraken/rest-api.md`, `docs/exchanges/kraken/websocket.md` | https://docs.kraken.com/api/docs/guides/spot-rest-auth/ |
| `kraken` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/kraken/websocket.md` | https://docs.kraken.com/api/docs/rest-api/get-websockets-token/ |
| `kraken` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/kraken/rest-api.md`, `docs/exchanges/kraken/README.md` | https://docs.kraken.com/api/docs/guides/spot-rest-ratelimits/ |
| `kraken` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/kraken/rest-api.md`, `docs/exchanges/kraken/data.md` | https://docs.kraken.com/api/docs/category/spot-rest-api |
| `kraken` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/kraken/README.md`, `docs/exchanges/kraken/sources.md` | https://docs.kraken.com/api/docs/change-log |
| `okx` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/okx/rest-api.md`, `docs/exchanges/okx/websocket.md` | https://www.okx.com/docs-v5/en/#overview-rest-authentication |
| `okx` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/okx/websocket.md` | https://www.okx.com/docs-v5/en/#overview-websocket-overview |
| `okx` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/okx/rest-api.md`, `docs/exchanges/okx/README.md` | https://www.okx.com/docs-v5/en/#overview-rate-limit |
| `okx` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/okx/rest-api.md`, `docs/exchanges/okx/data.md` | https://www.okx.com/docs-v5/en/#overview-rest-authentication |
| `okx` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/okx/README.md`, `docs/exchanges/okx/sources.md` | https://www.okx.com/docs-v5/en/ |
| `sbivc` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/sbivc/rest-api.md`, `docs/exchanges/sbivc/websocket.md` | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 |
| `sbivc` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/sbivc/websocket.md` | https://www.sbivc.co.jp/newsview/517ukdsg7_no |
| `sbivc` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/sbivc/rest-api.md`, `docs/exchanges/sbivc/README.md` | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 |
| `sbivc` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/sbivc/rest-api.md`, `docs/exchanges/sbivc/data.md` | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 |
| `sbivc` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/sbivc/README.md`, `docs/exchanges/sbivc/sources.md` | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 |
| `upbit` | P0 | 認証仕様（署名文字列/timestamp/header）未確定 | 認証失敗・誤実装リスク | `docs/exchanges/upbit/rest-api.md`, `docs/exchanges/upbit/websocket.md` | https://global-docs.upbit.com/reference/auth |
| `upbit` | P0 | WS keepalive/reconnect/sequence 復元要件の不足 | 板同期崩れ・切断復帰失敗 | `docs/exchanges/upbit/websocket.md` | https://global-docs.upbit.com/reference/websocket-guide |
| `upbit` | P1 | Rate limit/Retry指針の明示不足 | 過剰リトライ・BANリスク | `docs/exchanges/upbit/rest-api.md`, `docs/exchanges/upbit/README.md` | https://global-docs.upbit.com/reference/rate-limits |
| `upbit` | P1 | 数量・価格精度/丸め規則の明示不足 | 発注拒否・価格ズレ | `docs/exchanges/upbit/rest-api.md`, `docs/exchanges/upbit/data.md` | https://global-docs.upbit.com/reference |
| `upbit` | P2 | 版/非推奨区分と changelog 導線の改善余地 | 維持運用の追従漏れ | `docs/exchanges/upbit/README.md`, `docs/exchanges/upbit/sources.md` | https://global-docs.upbit.com/changelog |

## Summary by severity
- P0: 認証仕様確定、WS運用確定（全取引所）
- P1: レート制限、丸め/精度、パラメータ定義の不足（全取引所）
- P2: 版管理・changelog導線・補足説明の改善（全取引所）
