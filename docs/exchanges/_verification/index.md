# Exchange Docs Verification Ledger

最終更新日: 2026-02-21
目的: `docs/exchanges` 配下の取引所APIドキュメントを、公式一次情報と実通信で検証し、確定状態を管理する。

## ステータス定義
- 未着手: まだ検証を開始していない
- 調査中: EXD-002〜006 を実施中
- 確定: EXD-007 まで反映済み
- 要追従: 公式更新により再検証が必要

## EXD-001 インベントリ（固定）
- 詳細ファイル一覧: `docs/exchanges/_verification/inventory.md`
- ルール: `official_reference_url` は空欄禁止。EXD-002で公式導線から最終確定する。

| Exchange | Docs Path | Doc Types | Official Reference URL | Status | Current Task |
|---|---|---|---|---|---|
| binance | `docs/exchanges/binance` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://developers.binance.com/docs/binance-spot-api-docs/README | 調査中 | EXD-006待ち |
| binance-coinm | `docs/exchanges/binance-coinm` | catalog, changelog, diff, market-data, overview, reference-index, rest, schema-template, ws | https://developers.binance.com/docs/derivatives/coin-margined-futures | 未着手 | EXD-002待ち |
| binance-options | `docs/exchanges/binance-options` | catalog, changelog, diff, market-data, overview, reference-index, rest, schema-template, ws | https://developers.binance.com/docs/derivatives/option | 未着手 | EXD-002待ち |
| binance-usdm | `docs/exchanges/binance-usdm` | catalog, changelog, diff, market-data, overview, reference-index, rest, schema-template, ws | https://developers.binance.com/docs/derivatives/usds-margined-futures | 未着手 | EXD-002待ち |
| bitbank | `docs/exchanges/bitbank` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://github.com/bitbankinc/bitbank-api-docs | 未着手 | EXD-002待ち |
| bitflyer | `docs/exchanges/bitflyer` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://bitflyer.com/ja-jp/api | 未着手 | EXD-002待ち |
| bitget | `docs/exchanges/bitget` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://www.bitget.com/api-doc/common/intro | 未着手 | EXD-002待ち |
| bithumb | `docs/exchanges/bithumb` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://apidocs.bithumb.com/ | 未着手 | EXD-002待ち |
| bitmex | `docs/exchanges/bitmex` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://www.bitmex.com/api/explorer/ | 未着手 | EXD-002待ち |
| bittrade | `docs/exchanges/bittrade` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://www.bittrade.co.jp/ | 未着手 | EXD-002待ち |
| bybit | `docs/exchanges/bybit` | catalog, changelog, diff, market-data, overview, reference-index, rest, schema-template, ws | https://bybit-exchange.github.io/docs/v5/intro | 未着手 | EXD-002待ち |
| coinbase | `docs/exchanges/coinbase` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview | 未着手 | EXD-002待ち |
| coincheck | `docs/exchanges/coincheck` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://coincheck.com/ja/ | 未着手 | EXD-002待ち |
| deribit | `docs/exchanges/deribit` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://docs.deribit.com/ | 未着手 | EXD-002待ち |
| gmocoin | `docs/exchanges/gmocoin` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://api.coin.z.com/docs/ | 未着手 | EXD-002待ち |
| htx | `docs/exchanges/htx` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://www.htx.com/en-us/opend/newApiPages/ | 未着手 | EXD-002待ち |
| kraken | `docs/exchanges/kraken` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://docs.kraken.com/ | 未着手 | EXD-002待ち |
| okx | `docs/exchanges/okx` | catalog, changelog, diff, market-data, overview, reference-index, rest, schema-template, ws | https://www.okx.com/docs-v5/en/ | 未着手 | EXD-002待ち |
| sbivc | `docs/exchanges/sbivc` | catalog, changelog, market-data, overview, reference-index, rest, schema-template, ws | https://www.sbivc.co.jp/faqs/content/5c0nv5540jm3 | 未着手 | EXD-002待ち |
| upbit | `docs/exchanges/upbit` | catalog, changelog, diff, fix-log, market-data, overview, reference-index, rest, schema-template, ws | https://global-docs.upbit.com/ | 未着手 | EXD-002待ち |

## 生成済み共通成果物
- `verification_checklist.md`
- `exchange_report_template.md`
- `inventory.md`
- `pr_submission_checklist.md`
- `evidence/`
- `reports/`
