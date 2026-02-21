# Exchange Documentation Verification Index

- Last updated: 2026-02-21
- Scope: docs/exchanges 配下の全20取引所（EXD-A/B/C）

## Progress Ledger

| Exchange | Official sources (primary) | Confirmed date | Status | Smoke test |
|---|---|---|---|---|
| `binance` | https://developers.binance.com/docs/binance-spot-api-docs/README | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `binance-coinm` | https://developers.binance.com/docs/derivatives/coin-margined-futures | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `binance-options` | https://developers.binance.com/docs/derivatives/option | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `binance-usdm` | https://developers.binance.com/docs/derivatives/usds-margined-futures | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bitbank` | https://github.com/bitbankinc/bitbank-api-docs | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bitflyer` | https://bitflyer.com/ja-jp/api | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bitget` | https://www.bitget.com/api-doc/common/intro | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bithumb` | https://apidocs.bithumb.com/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bitmex` | https://www.bitmex.com/api/explorer/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `bittrade` | https://www.bittrade.co.jp/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未実施(endpoint不明)/WS未実施 |
| `bybit` | https://bybit-exchange.github.io/docs/v5/intro | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `coinbase` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `coincheck` | https://coincheck.com/ja/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `deribit` | https://docs.deribit.com/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `gmocoin` | https://api.coin.z.com/docs/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `htx` | https://www.htx.com/en-us/opend/newApiPages/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `kraken` | https://docs.kraken.com/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `okx` | https://www.okx.com/docs-v5/en/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |
| `sbivc` | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 | 2026-02-21 | A完了 / B完了 / C例外 | REST未実施(endpoint不明)/WS未実施 |
| `upbit` | https://global-docs.upbit.com/ | 2026-02-21 | A完了 / B完了 / C例外 | REST未達(ネットワーク制約)/WS未実施 |

## Notes
- EXD-B: 全取引所で Coverage/Auth/P0/Rate/Error の Yes/No/NA 評価を完了。
- EXD-C: 多くの取引所で外部ネットワーク制約により Public REST スモーク未達。bittrade/sbivc は公開REST endpoint不明により未実施。WSは未実施（ログ保存済み）。
- C例外解除条件: 外部疎通可能環境で smoke_rest/smoke_ws を再実行し最終確定。
