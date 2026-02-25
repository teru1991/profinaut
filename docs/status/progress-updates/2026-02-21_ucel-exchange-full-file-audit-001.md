# UCEL Exchange Full File Audit 001

0. Audit Meta（監査メタ情報）
- Task ID: UCEL-EXCHANGE-FULL-FILE-AUDIT-001
- Repo: teru1991/profinaut
- Branch: copilot/create-audit-report-ucel-exchange
- Commit SHA: 8c3f9663ba415f7312862b64e134c542cc32d38b
- Audit timestamp (JST): 2026-02-21 14:09:21 +0900
- Auditor: Codex / AI
- Scope: ucel/** all files scanned
- Result (PASS/FAIL): FAIL
- Highest severity found: HIGH

0.1 Commands executed（再現用コマンド列）

```bash
# PRE-FLIGHT
cd /home/runner/work/profinaut/profinaut
sed -n '1,200p' docs/context/README_AI.md > /tmp/preflight_readme_ai.txt
cat docs/status/status.json > /tmp/preflight_status_json.txt
cat docs/handoff/HANDOFF.json > /tmp/preflight_handoff_json.txt
sed -n '1,200p' docs/decisions/decisions.md > /tmp/preflight_decisions_md.txt
cat docs/status/trace-index.json > /tmp/preflight_trace_index_json.txt
sed -n '1,200p' docs/runbooks/pr-preflight.md > /tmp/preflight_pr_preflight_md.txt

# BASELINE
(git rev-parse HEAD; git status --porcelain=v1; git log -1 --oneline) > /tmp/preflight_baseline.txt

# FILE ENUMERATION
find ucel -type f -print | sort > /tmp/ucel_files.txt
wc -l /tmp/ucel_files.txt > /tmp/ucel_files_count.txt

# COVERAGE DISCOVERY
(ls -la ucel/coverage || true) > /tmp/ucel_coverage_ls.txt
find ucel/coverage -maxdepth 1 -type f \( -name '*.yaml' -o -name '*.yml' -o -name '*.json' \) -print | sort > /tmp/ucel_coverage_manifests.txt
find ucel/coverage -maxdepth 1 -type f \( -name '*.yaml' -o -name '*.yml' -o -name '*.json' \) -printf '%f\n' | sed -E 's/\.(yaml|yml|json)$//' | sort -u > /tmp/exchanges_from_coverage.txt

# CODE SEARCH (BY EXCHANGE)
python (regex scan over ucel/** using /tmp/exchanges_from_coverage.txt) > /tmp/rg_hits_by_exchange.txt

# CATALOG PRESENCE
while read -r ex; do p='docs/exchanges/'"$ex"'/catalog.json'; [ -f "$p" ] || echo "$ex $p"; done < /tmp/exchanges_from_coverage.txt > /tmp/missing_catalog.txt

# SSOT ID SET EXTRACTION (per exchange)
python (catalog: rest_endpoints/ws_channels/data_feeds[].id, manifest: entries[].id) -> /tmp/audit_idset_summary.tsv and /tmp/audit_exchange_<ex>.txt

# POLICY SCANS
python regex scans over ucel/** -> /tmp/hardcoded_lists_1.txt /tmp/hardcoded_lists_2.txt /tmp/secret_patterns.txt /tmp/value_usage.txt /tmp/feature_usage.txt /tmp/msrv_signals.txt

# BUILD/TEST
cd /home/runner/work/profinaut/profinaut/ucel
cargo test -q
cargo test --doc -q
```

1. File Inventory（ucel/** 全ファイル棚卸し）

1.1 File count
- Total files under ucel/**: 211 （from wc -l /tmp/ucel_files.txt）

1.2 File list evidence
- Evidence file: /tmp/ucel_files.txt
- Notes: hiddenファイルも含む find -type f ベース。Cargo.toml/src/tests/coverage を含む。

2. Exchange Scope（取引所スコープ：自動検出）

2.1 Coverage-based discovery（唯一の正）
- Coverage dir exists: YES
- Coverage manifests count: 20
- Discovered exchanges count: 20

2.1.1 Discovered exchanges (sorted)
/tmp/exchanges_from_coverage.txt と一致
- BYBIT
- binance
- binance-coinm
- binance-options
- binance-usdm
- bitbank
- bitflyer
- bitget
- bitmex
- bittrade
- bybit
- coinbase
- coincheck
- deribit
- gmocoin
- htx
- kraken
- okx
- sbivc
- upbit

2.2 “Non-coverage exchange references” check（coverage外参照の検知）
- Any exchange-like tokens referenced but not in coverage: NO
- Evidence:
  - /tmp/non_coverage_refs.txt: bithumb, _verification は ucel/** 内ヒットなし
- Severity: MED

3. SSOT Alignment（coverage ↔ catalog ↔ 実装 ↔ テスト の整合）

3.1 Catalog presence（docs/exchanges/<exchange>/catalog.json）

| exchange | catalog.json exists | path | notes |
|---|---|---|---|
| BYBIT | NO | docs/exchanges/BYBIT/catalog.json | case mismatch (bybit exists) |
| binance | YES | docs/exchanges/binance/catalog.json |  |
| binance-coinm | YES | docs/exchanges/binance-coinm/catalog.json |  |
| binance-options | YES | docs/exchanges/binance-options/catalog.json |  |
| binance-usdm | YES | docs/exchanges/binance-usdm/catalog.json |  |
| bitbank | YES | docs/exchanges/bitbank/catalog.json |  |
| bitflyer | YES | docs/exchanges/bitflyer/catalog.json |  |
| bitget | YES | docs/exchanges/bitget/catalog.json |  |
| bitmex | YES | docs/exchanges/bitmex/catalog.json |  |
| bittrade | YES | docs/exchanges/bittrade/catalog.json |  |
| bybit | YES | docs/exchanges/bybit/catalog.json |  |
| coinbase | YES | docs/exchanges/coinbase/catalog.json |  |
| coincheck | YES | docs/exchanges/coincheck/catalog.json |  |
| deribit | YES | docs/exchanges/deribit/catalog.json |  |
| gmocoin | YES | docs/exchanges/gmocoin/catalog.json |  |
| htx | YES | docs/exchanges/htx/catalog.json |  |
| kraken | YES | docs/exchanges/kraken/catalog.json |  |
| okx | YES | docs/exchanges/okx/catalog.json |  |
| sbivc | YES | docs/exchanges/sbivc/catalog.json |  |
| upbit | YES | docs/exchanges/upbit/catalog.json |  |

- Missing catalogs summary:
  - /tmp/missing_catalog.txt: `BYBIT docs/exchanges/BYBIT/catalog.json`

3.2 Manifest presence（ucel/coverage/*）

| exchange | manifest exists | manifest file | ext | notes |
|---|---|---|---|---|
| BYBIT | YES | ucel/coverage/BYBIT.yaml | yaml | |
| binance | YES | ucel/coverage/binance.yaml | yaml | |
| binance-coinm | YES | ucel/coverage/binance-coinm.yaml | yaml | |
| binance-options | YES | ucel/coverage/binance-options.yaml | yaml | |
| binance-usdm | YES | ucel/coverage/binance-usdm.yaml | yaml | |
| bitbank | YES | ucel/coverage/bitbank.yaml | yaml | |
| bitflyer | YES | ucel/coverage/bitflyer.yaml | yaml | |
| bitget | YES | ucel/coverage/bitget.yaml | yaml | |
| bitmex | YES | ucel/coverage/bitmex.yaml | yaml | |
| bittrade | YES | ucel/coverage/bittrade.yaml | yaml | |
| bybit | YES | ucel/coverage/bybit.yaml | yaml | |
| coinbase | YES | ucel/coverage/coinbase.yaml | yaml | |
| coincheck | YES | ucel/coverage/coincheck.yaml | yaml | |
| deribit | YES | ucel/coverage/deribit.yaml | yaml | |
| gmocoin | YES | ucel/coverage/gmocoin.yaml | yaml | |
| htx | YES | ucel/coverage/htx.yaml | yaml | |
| kraken | YES | ucel/coverage/kraken.yaml | yaml | |
| okx | YES | ucel/coverage/okx.yaml | yaml | |
| sbivc | YES | ucel/coverage/sbivc.yaml | yaml | |
| upbit | YES | ucel/coverage/upbit.yaml | yaml | |

3.3 ID set equality（最重要：catalog_ids == manifest_ids）

| exchange | catalog REST ops | catalog WS ops | catalog total | manifest ops | set equality | missing_in_manifest | missing_in_catalog | extraction method |
|---|---:|---:|---:|---:|---|---|---|---|
| BYBIT | 77 | 19 | 99 | 96 | FAIL | bybit.data.account.ws.private,bybit.data.market.ws.public,bybit.data.trade.ws.trade | - | python(json/yaml parser) |
| binance | 9 | 5 | 19 | 14 | FAIL | data.account.realtime.json,data.market.realtime.json,data.market.realtime.sbe,data.market.snapshot.json,data.tradeevents.session.fix | - | python(json/yaml parser) |
| binance-coinm | 7 | 18 | 29 | 25 | FAIL | coinm.data.account.realtime.userdata,coinm.data.market.realtime.streams,coinm.data.market.snapshot.rest.depth,coinm.data.rpc.realtime.wsapi | - | python(json/yaml parser) |
| binance-options | 8 | 6 | 16 | 14 | FAIL | options.data.rest.market,options.data.ws.market | - | python(json/yaml parser) |
| binance-usdm | 6 | 10 | 20 | 16 | FAIL | usdm.data.account.realtime.userdata,usdm.data.market.realtime.streams,usdm.data.market.snapshot.rest.depth,usdm.data.rpc.realtime.wsapi | - | python(json/yaml parser) |
| bitbank | 28 | 16 | 44 | 44 | PASS | - | - | python(json/yaml parser) |
| bitflyer | 49 | 12 | 62 | 61 | FAIL | bitflyer.data.bulk.not_documented | - | python(json/yaml parser) |
| bitget | 1 | 1 | 3 | 2 | FAIL | data.other.unknown.unknown | - | python(json/yaml parser) |
| bitmex | 95 | 30 | 126 | 95 | FAIL | data.other.none.na,private.ws.affiliate.subscribe,private.ws.execution.subscribe,private.ws.margin.subscribe,private.ws.order.subscribe,private.ws.position.subscribe,private.ws.privatenotifications.subscribe,private.ws.transact.subscribe,private.ws.wallet.subscribe,public.ws.announcement.subscribe,public.ws.chat.subscribe,public.ws.connected.subscribe,public.ws.funding.subscribe,public.ws.instrument.subscribe,public.ws.insurance.subscribe,public.ws.liquidation.subscribe,public.ws.orderbook10.subscribe,public.ws.orderbookl2.subscribe,public.ws.orderbookl2_25.subscribe,public.ws.publicnotifications.subscribe,public.ws.quote.subscribe,public.ws.quotebin1d.subscribe,public.ws.quotebin1h.subscribe,public.ws.quotebin1m.subscribe,public.ws.quotebin5m.subscribe,public.ws.settlement.subscribe,public.ws.trade.subscribe,public.ws.tradebin1d.subscribe,public.ws.tradebin1h.subscribe,public.ws.tradebin1m.subscribe,public.ws.tradebin5m.subscribe | - | python(json/yaml parser) |
| bittrade | 27 | 7 | 35 | 34 | FAIL | data.none.official.bulk | - | python(json/yaml parser) |
| bybit | 77 | 19 | 99 | 77 | FAIL | bybit.data.account.ws.private,bybit.data.market.ws.public,bybit.data.trade.ws.trade,bybit.private.ws.private.dcp,bybit.private.ws.private.execution,bybit.private.ws.private.fast-execution,bybit.private.ws.private.greek,bybit.private.ws.private.order,bybit.private.ws.private.position,bybit.private.ws.private.wallet,bybit.private.ws.trade.guideline,bybit.public.ws.common.auth,bybit.public.ws.public.adl-alert,bybit.public.ws.public.insurance-pool,bybit.public.ws.public.kline,bybit.public.ws.public.liquidation,bybit.public.ws.public.order-price-limit,bybit.public.ws.public.orderbook,bybit.public.ws.public.orderbook-rpi,bybit.public.ws.public.ticker,bybit.public.ws.public.trade,bybit.public.ws.system.status | - | python(json/yaml parser) |
| coinbase | 7 | 8 | 16 | 15 | FAIL | data.other.unknown.other | - | python(json/yaml parser) |
| coincheck | 25 | 4 | 30 | 29 | FAIL | coincheck.data.none.official_bulk | - | python(json/yaml parser) |
| deribit | 0 | 0 | 1 | 28 | FAIL | data.other.unknown.other | jsonrpc.http.private.account.private_get_account_summary,jsonrpc.http.private.auth.public_auth,jsonrpc.http.private.trading.private_buy,jsonrpc.http.private.trading.private_cancel,jsonrpc.http.private.trading.private_sell,jsonrpc.http.public.market_data.public_get_instruments,jsonrpc.http.public.market_data.public_get_order_book,jsonrpc.http.public.market_data.public_get_tradingview_chart_data,jsonrpc.http.public.market_data.public_ticker,jsonrpc.ws.private.account.private_get_account_summary,jsonrpc.ws.private.trading.private_buy,jsonrpc.ws.private.trading.private_cancel,jsonrpc.ws.private.trading.private_sell,jsonrpc.ws.public.auth.public_auth,jsonrpc.ws.public.market_data.public_get_instruments,jsonrpc.ws.public.market_data.public_get_order_book,jsonrpc.ws.public.market_data.public_get_tradingview_chart_data,jsonrpc.ws.public.market_data.public_ticker,jsonrpc.ws.public.system.public_set_heartbeat,ws.private.sub.user.changes.instrument.raw,ws.private.sub.user.orders.instrument.raw,ws.private.sub.user.portfolio.currency,ws.private.sub.user.trades.instrument.interval,ws.public.sub.book.instrument.raw,ws.public.sub.chart.instrument.resolution,ws.public.sub.deribit_price_index.index,ws.public.sub.ticker.instrument.interval,ws.public.sub.trades.instrument.interval | python(json/yaml parser) |
| gmocoin | 30 | 12 | 43 | 42 | FAIL | data.snapshots.daily.csv.gz | - | python(json/yaml parser) |
| htx | 13 | 9 | 23 | 22 | FAIL | data.other.unknown.not_applicable | - | python(json/yaml parser) |
| kraken | 10 | 10 | 21 | 20 | FAIL | data.market-candles.minute.json | - | python(json/yaml parser) |
| okx | 4 | 3 | 9 | 7 | FAIL | okx.data.public-rest,okx.data.public-ws | - | python(json/yaml parser) |
| sbivc | 0 | 0 | 0 | 0 | PASS | - | - | python(json/yaml parser) |
| upbit | 22 | 7 | 30 | 29 | FAIL | data.other.not_applicable.json | - | python(json/yaml parser) |

3.3.1 Per-exchange detail blocks（取引所ごとに1ブロック）

Exchange: BYBIT
- Catalog path: docs/exchanges/BYBIT/catalog.json
- Manifest path: ucel/coverage/BYBIT.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: bybit.data.account.ws.private,bybit.data.market.ws.public,bybit.data.trade.ws.trade
  - Missing in catalog: -
- Verdict for BYBIT: FAIL
- Severity if FAIL: HIGH

Exchange: binance
- Catalog path: docs/exchanges/binance/catalog.json
- Manifest path: ucel/coverage/binance.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.account.realtime.json,data.market.realtime.json,data.market.realtime.sbe,data.market.snapshot.json,data.tradeevents.session.fix
  - Missing in catalog: -
- Verdict for binance: FAIL
- Severity if FAIL: HIGH

Exchange: binance-coinm
- Catalog path: docs/exchanges/binance-coinm/catalog.json
- Manifest path: ucel/coverage/binance-coinm.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: coinm.data.account.realtime.userdata,coinm.data.market.realtime.streams,coinm.data.market.snapshot.rest.depth,coinm.data.rpc.realtime.wsapi
  - Missing in catalog: -
- Verdict for binance-coinm: FAIL
- Severity if FAIL: HIGH

Exchange: binance-options
- Catalog path: docs/exchanges/binance-options/catalog.json
- Manifest path: ucel/coverage/binance-options.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: options.data.rest.market,options.data.ws.market
  - Missing in catalog: -
- Verdict for binance-options: FAIL
- Severity if FAIL: HIGH

Exchange: binance-usdm
- Catalog path: docs/exchanges/binance-usdm/catalog.json
- Manifest path: ucel/coverage/binance-usdm.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: usdm.data.account.realtime.userdata,usdm.data.market.realtime.streams,usdm.data.market.snapshot.rest.depth,usdm.data.rpc.realtime.wsapi
  - Missing in catalog: -
- Verdict for binance-usdm: FAIL
- Severity if FAIL: HIGH

Exchange: bitbank
- Catalog path: docs/exchanges/bitbank/catalog.json
- Manifest path: ucel/coverage/bitbank.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: -
  - Missing in catalog: -
- Verdict for bitbank: PASS
- Severity if FAIL: NONE

Exchange: bitflyer
- Catalog path: docs/exchanges/bitflyer/catalog.json
- Manifest path: ucel/coverage/bitflyer.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: bitflyer.data.bulk.not_documented
  - Missing in catalog: -
- Verdict for bitflyer: FAIL
- Severity if FAIL: HIGH

Exchange: bitget
- Catalog path: docs/exchanges/bitget/catalog.json
- Manifest path: ucel/coverage/bitget.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.unknown.unknown
  - Missing in catalog: -
- Verdict for bitget: FAIL
- Severity if FAIL: HIGH

Exchange: bitmex
- Catalog path: docs/exchanges/bitmex/catalog.json
- Manifest path: ucel/coverage/bitmex.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.none.na,private.ws.affiliate.subscribe,private.ws.execution.subscribe,private.ws.margin.subscribe,private.ws.order.subscribe,private.ws.position.subscribe,private.ws.privatenotifications.subscribe,private.ws.transact.subscribe,private.ws.wallet.subscribe,public.ws.announcement.subscribe,public.ws.chat.subscribe,public.ws.connected.subscribe,public.ws.funding.subscribe,public.ws.instrument.subscribe,public.ws.insurance.subscribe,public.ws.liquidation.subscribe,public.ws.orderbook10.subscribe,public.ws.orderbookl2.subscribe,public.ws.orderbookl2_25.subscribe,public.ws.publicnotifications.subscribe,public.ws.quote.subscribe,public.ws.quotebin1d.subscribe,public.ws.quotebin1h.subscribe,public.ws.quotebin1m.subscribe,public.ws.quotebin5m.subscribe,public.ws.settlement.subscribe,public.ws.trade.subscribe,public.ws.tradebin1d.subscribe,public.ws.tradebin1h.subscribe,public.ws.tradebin1m.subscribe,public.ws.tradebin5m.subscribe
  - Missing in catalog: -
- Verdict for bitmex: FAIL
- Severity if FAIL: HIGH

Exchange: bittrade
- Catalog path: docs/exchanges/bittrade/catalog.json
- Manifest path: ucel/coverage/bittrade.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.none.official.bulk
  - Missing in catalog: -
- Verdict for bittrade: FAIL
- Severity if FAIL: HIGH

Exchange: bybit
- Catalog path: docs/exchanges/bybit/catalog.json
- Manifest path: ucel/coverage/bybit.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: bybit.data.account.ws.private,bybit.data.market.ws.public,bybit.data.trade.ws.trade,bybit.private.ws.private.dcp,bybit.private.ws.private.execution,bybit.private.ws.private.fast-execution,bybit.private.ws.private.greek,bybit.private.ws.private.order,bybit.private.ws.private.position,bybit.private.ws.private.wallet,bybit.private.ws.trade.guideline,bybit.public.ws.common.auth,bybit.public.ws.public.adl-alert,bybit.public.ws.public.insurance-pool,bybit.public.ws.public.kline,bybit.public.ws.public.liquidation,bybit.public.ws.public.order-price-limit,bybit.public.ws.public.orderbook,bybit.public.ws.public.orderbook-rpi,bybit.public.ws.public.ticker,bybit.public.ws.public.trade,bybit.public.ws.system.status
  - Missing in catalog: -
- Verdict for bybit: FAIL
- Severity if FAIL: HIGH

Exchange: coinbase
- Catalog path: docs/exchanges/coinbase/catalog.json
- Manifest path: ucel/coverage/coinbase.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.unknown.other
  - Missing in catalog: -
- Verdict for coinbase: FAIL
- Severity if FAIL: HIGH

Exchange: coincheck
- Catalog path: docs/exchanges/coincheck/catalog.json
- Manifest path: ucel/coverage/coincheck.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: coincheck.data.none.official_bulk
  - Missing in catalog: -
- Verdict for coincheck: FAIL
- Severity if FAIL: HIGH

Exchange: deribit
- Catalog path: docs/exchanges/deribit/catalog.json
- Manifest path: ucel/coverage/deribit.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.unknown.other
  - Missing in catalog: jsonrpc.http.private.account.private_get_account_summary,jsonrpc.http.private.auth.public_auth,jsonrpc.http.private.trading.private_buy,jsonrpc.http.private.trading.private_cancel,jsonrpc.http.private.trading.private_sell,jsonrpc.http.public.market_data.public_get_instruments,jsonrpc.http.public.market_data.public_get_order_book,jsonrpc.http.public.market_data.public_get_tradingview_chart_data,jsonrpc.http.public.market_data.public_ticker,jsonrpc.ws.private.account.private_get_account_summary,jsonrpc.ws.private.trading.private_buy,jsonrpc.ws.private.trading.private_cancel,jsonrpc.ws.private.trading.private_sell,jsonrpc.ws.public.auth.public_auth,jsonrpc.ws.public.market_data.public_get_instruments,jsonrpc.ws.public.market_data.public_get_order_book,jsonrpc.ws.public.market_data.public_get_tradingview_chart_data,jsonrpc.ws.public.market_data.public_ticker,jsonrpc.ws.public.system.public_set_heartbeat,ws.private.sub.user.changes.instrument.raw,ws.private.sub.user.orders.instrument.raw,ws.private.sub.user.portfolio.currency,ws.private.sub.user.trades.instrument.interval,ws.public.sub.book.instrument.raw,ws.public.sub.chart.instrument.resolution,ws.public.sub.deribit_price_index.index,ws.public.sub.ticker.instrument.interval,ws.public.sub.trades.instrument.interval
- Verdict for deribit: FAIL
- Severity if FAIL: HIGH

Exchange: gmocoin
- Catalog path: docs/exchanges/gmocoin/catalog.json
- Manifest path: ucel/coverage/gmocoin.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.snapshots.daily.csv.gz
  - Missing in catalog: -
- Verdict for gmocoin: FAIL
- Severity if FAIL: HIGH

Exchange: htx
- Catalog path: docs/exchanges/htx/catalog.json
- Manifest path: ucel/coverage/htx.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.unknown.not_applicable
  - Missing in catalog: -
- Verdict for htx: FAIL
- Severity if FAIL: HIGH

Exchange: kraken
- Catalog path: docs/exchanges/kraken/catalog.json
- Manifest path: ucel/coverage/kraken.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.market-candles.minute.json
  - Missing in catalog: -
- Verdict for kraken: FAIL
- Severity if FAIL: HIGH

Exchange: okx
- Catalog path: docs/exchanges/okx/catalog.json
- Manifest path: ucel/coverage/okx.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: okx.data.public-rest,okx.data.public-ws
  - Missing in catalog: -
- Verdict for okx: FAIL
- Severity if FAIL: HIGH

Exchange: sbivc
- Catalog path: docs/exchanges/sbivc/catalog.json
- Manifest path: ucel/coverage/sbivc.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: -
  - Missing in catalog: -
- Verdict for sbivc: PASS
- Severity if FAIL: NONE

Exchange: upbit
- Catalog path: docs/exchanges/upbit/catalog.json
- Manifest path: ucel/coverage/upbit.yaml
- Extraction:
  - Catalog IDs extraction: Python json parser (`rest_endpoints/ws_channels/data_feeds[].id`)
  - Manifest IDs extraction: Python yaml parser (`entries[].id`)
- Diff evidence:
  - Missing in manifest: data.other.not_applicable.json
  - Missing in catalog: -
- Verdict for upbit: FAIL
- Severity if FAIL: HIGH

4. Implementation Touchpoints（ucel/** 内の実装参照・所有関係）

4.1 Exchange hits in code（取引所名の言及箇所）

| exchange | hits count | top files (max 10) | notes |
|---|---:|---|---|
| BYBIT | 216 | ucel/coverage/BYBIT.yaml(97); ucel/coverage/bybit.yaml(78); ucel/crates/ucel-cex-bybit/src/lib.rs(39); ucel/Cargo.toml(1); ucel/crates/ucel-cex-bybit/Cargo.toml(1) | registry/adapter candidate |
| binance | 205 | ucel/crates/ucel-cex-binance-coinm/src/lib.rs(67); ucel/crates/ucel-cex-binance-usdm/src/lib.rs(48); ucel/crates/ucel-cex-binance/src/lib.rs(40); ucel/crates/ucel-cex-binance-options/src/lib.rs(34); ucel/Cargo.lock(4); ucel/Cargo.toml(4); ucel/coverage/binance-options.yaml(1); ucel/coverage/binance-usdm.yaml(1); ucel/coverage/binance-coinm.yaml(1); ucel/coverage/binance.yaml(1) | registry/adapter candidate |
| binance-coinm | 21 | ucel/crates/ucel-cex-binance-coinm/src/lib.rs(17); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/binance-coinm.yaml(1); ucel/crates/ucel-cex-binance-coinm/Cargo.toml(1) | registry/adapter candidate |
| binance-options | 14 | ucel/crates/ucel-cex-binance-options/src/lib.rs(10); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/binance-options.yaml(1); ucel/crates/ucel-cex-binance-options/Cargo.toml(1) | registry/adapter candidate |
| binance-usdm | 15 | ucel/crates/ucel-cex-binance-usdm/src/lib.rs(11); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/binance-usdm.yaml(1); ucel/crates/ucel-cex-binance-usdm/Cargo.toml(1) | registry/adapter candidate |
| bitbank | 16 | ucel/crates/ucel-cex-bitbank/src/lib.rs(12); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/bitbank.yaml(1); ucel/crates/ucel-cex-bitbank/Cargo.toml(1) | registry/adapter candidate |
| bitflyer | 59 | ucel/crates/ucel-cex-bitflyer/src/lib.rs(55); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/bitflyer.yaml(1); ucel/crates/ucel-cex-bitflyer/Cargo.toml(1) | registry/adapter candidate |
| bitget | 33 | ucel/crates/ucel-cex-bitget/src/lib.rs(30); ucel/Cargo.lock(1); ucel/coverage/bitget.yaml(1); ucel/crates/ucel-cex-bitget/Cargo.toml(1) | registry/adapter candidate |
| bitmex | 44 | ucel/crates/ucel-cex-bitmex/src/lib.rs(40); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/bitmex.yaml(1); ucel/crates/ucel-cex-bitmex/Cargo.toml(1) | registry/adapter candidate |
| bittrade | 33 | ucel/crates/ucel-cex-bittrade/src/lib.rs(20); ucel/crates/ucel-cex-bittrade/tests/rest_contract.rs(9); ucel/crates/ucel-cex-bybit/src/lib.rs(2); ucel/coverage/bittrade.yaml(1); ucel/crates/ucel-cex-bittrade/Cargo.toml(1) | registry/adapter candidate |
| bybit | 216 | ucel/coverage/BYBIT.yaml(97); ucel/coverage/bybit.yaml(78); ucel/crates/ucel-cex-bybit/src/lib.rs(39); ucel/Cargo.toml(1); ucel/crates/ucel-cex-bybit/Cargo.toml(1) | registry/adapter candidate |
| coinbase | 29 | ucel/crates/ucel-cex-coinbase/src/lib.rs(19); ucel/crates/ucel-cex-coinbase/tests/rest_contract.rs(6); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/coinbase.yaml(1); ucel/crates/ucel-cex-coinbase/Cargo.toml(1) | registry/adapter candidate |
| coincheck | 71 | ucel/crates/ucel-cex-coincheck/src/lib.rs(34); ucel/coverage/coincheck.yaml(30); ucel/crates/ucel-cex-coincheck/tests/rest_contract.rs(4); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/crates/ucel-cex-coincheck/Cargo.toml(1) | registry/adapter candidate |
| deribit | 111 | ucel/crates/ucel-cex-deribit/src/lib.rs(69); ucel/crates/ucel-registry/src/deribit.rs(19); ucel/crates/ucel-cex-deribit/tests/contract_rest.rs(16); ucel/crates/ucel-registry/src/lib.rs(3); ucel/coverage/deribit.yaml(2); ucel/crates/ucel-cex-deribit/Cargo.lock(1); ucel/crates/ucel-cex-deribit/Cargo.toml(1) | registry/adapter candidate |
| gmocoin | 22 | ucel/crates/ucel-cex-gmocoin/src/lib.rs(18); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/gmocoin.yaml(1); ucel/crates/ucel-cex-gmocoin/Cargo.toml(1) | registry/adapter candidate |
| htx | 56 | ucel/crates/ucel-cex-htx/src/lib.rs(31); ucel/crates/ucel-cex-htx/tests/rest_contract.rs(22); ucel/Cargo.lock(1); ucel/coverage/htx.yaml(1); ucel/crates/ucel-cex-htx/Cargo.toml(1) | registry/adapter candidate |
| kraken | 62 | ucel/crates/ucel-cex-kraken/src/lib.rs(58); ucel/Cargo.lock(1); ucel/Cargo.toml(1); ucel/coverage/kraken.yaml(1); ucel/crates/ucel-cex-kraken/Cargo.toml(1) | registry/adapter candidate |
| okx | 146 | ucel/crates/ucel-cex-okx/src/lib.rs(79); ucel/crates/ucel-registry/src/okx.rs(33); ucel/crates/ucel-cex-okx/tests/rest_contract.rs(16); ucel/coverage/okx.yaml(8); ucel/crates/ucel-testkit/src/okx.rs(6); ucel/Cargo.lock(1); ucel/crates/ucel-cex-okx/Cargo.toml(1); ucel/crates/ucel-testkit/src/lib.rs(1); ucel/crates/ucel-registry/src/lib.rs(1) | registry/adapter candidate |
| sbivc | 19 | ucel/crates/ucel-cex-sbivc/src/lib.rs(17); ucel/coverage/sbivc.yaml(1); ucel/crates/ucel-cex-sbivc/Cargo.toml(1) | registry/adapter candidate |
| upbit | 52 | ucel/crates/ucel-cex-upbit/src/lib.rs(26); ucel/crates/ucel-cex-upbit/tests/rest_contract.rs(22); ucel/Cargo.toml(1); ucel/coverage/upbit.yaml(1); ucel/crates/ucel-cex-upbit/Cargo.toml(1); ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.travelrule.vasps.json(1) | registry/adapter candidate |

- Evidence file: /tmp/rg_hits_by_exchange.txt（要約は /tmp/exchange_hits_summary.tsv）

4.2 Ownership mapping（どのcrate/moduleが担当しているか）

| exchange | likely owner crate | modules | evidence |
|---|---|---|---|
| BYBIT | ucel-cex-bybit | src/lib.rs; tests/ | ucel/crates/ucel-cex-bybit/src/lib.rs |
| binance | ucel-cex-binance | src/lib.rs; tests/ | ucel/crates/ucel-cex-binance/src/lib.rs |
| binance-coinm | ucel-cex-binance-coinm | src/lib.rs; tests/ | ucel/crates/ucel-cex-binance-coinm/src/lib.rs |
| binance-options | ucel-cex-binance-options | src/lib.rs; tests/ | ucel/crates/ucel-cex-binance-options/src/lib.rs |
| binance-usdm | ucel-cex-binance-usdm | src/lib.rs; tests/ | ucel/crates/ucel-cex-binance-usdm/src/lib.rs |
| bitbank | ucel-cex-bitbank | src/lib.rs; tests/ | ucel/crates/ucel-cex-bitbank/src/lib.rs |
| bitflyer | ucel-cex-bitflyer | src/lib.rs; tests/ | ucel/crates/ucel-cex-bitflyer/src/lib.rs |
| bitget | ucel-cex-bitget | src/lib.rs; tests/ | ucel/crates/ucel-cex-bitget/src/lib.rs |
| bitmex | ucel-cex-bitmex | src/lib.rs; tests/ | ucel/crates/ucel-cex-bitmex/src/lib.rs |
| bittrade | ucel-cex-bittrade | src/lib.rs; tests/ | ucel/crates/ucel-cex-bittrade/src/lib.rs |
| bybit | ucel-cex-bybit | src/lib.rs; tests/ | ucel/crates/ucel-cex-bybit/src/lib.rs |
| coinbase | ucel-cex-coinbase | src/lib.rs; tests/ | ucel/crates/ucel-cex-coinbase/src/lib.rs |
| coincheck | ucel-cex-coincheck | src/lib.rs; tests/ | ucel/crates/ucel-cex-coincheck/src/lib.rs |
| deribit | ucel-cex-deribit | src/lib.rs; tests/ | ucel/crates/ucel-cex-deribit/src/lib.rs |
| gmocoin | ucel-cex-gmocoin | src/lib.rs; tests/ | ucel/crates/ucel-cex-gmocoin/src/lib.rs |
| htx | ucel-cex-htx | src/lib.rs; tests/ | ucel/crates/ucel-cex-htx/src/lib.rs |
| kraken | ucel-cex-kraken | src/lib.rs; tests/ | ucel/crates/ucel-cex-kraken/src/lib.rs |
| okx | ucel-cex-okx | src/lib.rs; tests/ | ucel/crates/ucel-cex-okx/src/lib.rs |
| sbivc | ucel-cex-sbivc | src/lib.rs; tests/ | ucel/crates/ucel-cex-sbivc/src/lib.rs |
| upbit | ucel-cex-upbit | src/lib.rs; tests/ | ucel/crates/ucel-cex-upbit/src/lib.rs |

5. Test Coverage Signals（テスト存在の監査）

5.1 Tests inventory（ucel/**/tests, src/** mod tests）
- tests/ directories found:
  - ucel/crates/ucel-cex-binance-coinm/tests
  - ucel/crates/ucel-cex-binance-options/tests
  - ucel/crates/ucel-cex-bittrade/tests
  - ucel/crates/ucel-cex-coinbase/tests
  - ucel/crates/ucel-cex-coincheck/tests
  - ucel/crates/ucel-cex-deribit/tests
  - ucel/crates/ucel-cex-htx/tests
  - ucel/crates/ucel-cex-kraken/tests
  - ucel/crates/ucel-cex-okx/tests
  - ucel/crates/ucel-cex-upbit/tests
- “exchange-specific tests” detected:
  - BYBIT: ucel/crates/ucel-cex-bybit/src/lib.rs
  - binance: ucel/crates/ucel-cex-binance-coinm/src/lib.rs;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.account.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.listenkey.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.trade.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.ws.userdata.events.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.common.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.errors.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.general.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.market.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.aggtrade.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.bookticker.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.composite-index.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.continuous-kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.contract-info.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.depth.diff.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.depth.partial.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.index-kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.liquidation.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.markprice.json
  - binance-coinm: ucel/crates/ucel-cex-binance-coinm/src/lib.rs;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.account.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.listenkey.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.rest.trade.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.private.ws.userdata.events.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.common.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.errors.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.general.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.rest.market.ref.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.aggtrade.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.bookticker.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.composite-index.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.continuous-kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.contract-info.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.depth.diff.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.depth.partial.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.index-kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.kline.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.liquidation.json;ucel/crates/ucel-cex-binance-coinm/tests/fixtures/coinm.public.ws.market.markprice.json
  - binance-options: ucel/crates/ucel-cex-binance-options/src/lib.rs;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.private.rest.account.ref.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.private.rest.listenkey.delete.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.private.rest.listenkey.post.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.private.rest.listenkey.put.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.private.rest.trade.ref.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.public.rest.errors.ref.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.public.rest.general.ref.json;ucel/crates/ucel-cex-binance-options/tests/fixtures/options.public.rest.market.ref.json
  - binance-usdm: ucel/crates/ucel-cex-binance-usdm/src/lib.rs
  - bitbank: ucel/crates/ucel-cex-bitbank/src/lib.rs
  - bitflyer: ucel/crates/ucel-cex-bitflyer/src/lib.rs
  - bitget: ucel/crates/ucel-cex-bitget/src/lib.rs
  - bitmex: ucel/crates/ucel-cex-bitmex/src/lib.rs
  - bittrade: ucel/crates/ucel-cex-bittrade/src/lib.rs;ucel/crates/ucel-cex-bittrade/tests/fixtures/other.rest.host.info.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.account.accounts.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.account.balance.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.batchcancel.open.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.batchcancel.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.cancel.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.list.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.matchresults.byorder.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.matchresults.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.open.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.order.place.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.retail.maintain.time.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.retail.order.list.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.retail.order.place.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.wallet.depositwithdraw.get.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.wallet.withdraw.cancel.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/private.rest.wallet.withdraw.create.post.json;ucel/crates/ucel-cex-bittrade/tests/fixtures/public.rest.common.currencys.get.json
  - bybit: ucel/crates/ucel-cex-bybit/src/lib.rs
  - coinbase: ucel/crates/ucel-cex-coinbase/src/lib.rs;ucel/crates/ucel-cex-coinbase/tests/fixtures/advanced.crypto.private.rest.reference.introduction.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/advanced.crypto.public.rest.reference.introduction.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/exchange.crypto.private.rest.reference.introduction.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/exchange.crypto.public.rest.reference.introduction.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/intx.crypto.private.rest.reference.welcome.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/intx.crypto.public.rest.reference.welcome.json;ucel/crates/ucel-cex-coinbase/tests/fixtures/other.other.public.rest.docs.root.json;ucel/crates/ucel-cex-coinbase/tests/rest_contract.rs
  - coincheck: ucel/crates/ucel-cex-coincheck/src/lib.rs;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.other.auth.headers.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.accounts.balance.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.accounts.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.bank_accounts.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.bank_accounts.id.delete.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.bank_accounts.post.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.deposit_money.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.cancel_status.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.id.delete.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.id.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.opens.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.post.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.transactions.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.exchange.orders.transactions_pagination.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.send_money.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.send_money.post.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.withdraws.get.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.withdraws.id.delete.json;ucel/crates/ucel-cex-coincheck/tests/fixtures/coincheck.rest.private.withdraws.post.json
  - deribit: ucel/crates/ucel-cex-deribit/src/lib.rs;ucel/crates/ucel-cex-deribit/tests/contract_rest.rs;ucel/crates/ucel-registry/src/deribit.rs;ucel/crates/ucel-registry/src/lib.rs
  - gmocoin: ucel/crates/ucel-cex-gmocoin/src/lib.rs
  - htx: ucel/crates/ucel-cex-htx/src/lib.rs;ucel/crates/ucel-cex-htx/tests/fixtures/rest.success.json;ucel/crates/ucel-cex-htx/tests/rest_contract.rs
  - kraken: ucel/crates/ucel-cex-kraken/src/lib.rs;ucel/crates/ucel-cex-kraken/tests/fixtures/futures.private.rest.accounts.get.json;ucel/crates/ucel-cex-kraken/tests/fixtures/futures.private.rest.order.send.json;ucel/crates/ucel-cex-kraken/tests/fixtures/futures.public.rest.instruments.list.json;ucel/crates/ucel-cex-kraken/tests/fixtures/futures.public.rest.tickers.list.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.private.rest.balance.get.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.private.rest.order.add.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.private.rest.token.ws.get.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.public.rest.asset-pairs.list.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.public.rest.assets.list.json;ucel/crates/ucel-cex-kraken/tests/fixtures/spot.public.rest.ticker.get.json
  - okx: ucel/crates/ucel-cex-okx/src/lib.rs;ucel/crates/ucel-cex-okx/tests/fixtures/okx.rest.auth.json;ucel/crates/ucel-cex-okx/tests/fixtures/okx.rest.overview.json;ucel/crates/ucel-cex-okx/tests/fixtures/okx.rest.private.json;ucel/crates/ucel-cex-okx/tests/fixtures/okx.rest.public.json;ucel/crates/ucel-cex-okx/tests/rest_contract.rs;ucel/crates/ucel-registry/src/lib.rs;ucel/crates/ucel-registry/src/okx.rs;ucel/crates/ucel-testkit/src/lib.rs;ucel/crates/ucel-testkit/src/okx.rs
  - sbivc: ucel/crates/ucel-cex-sbivc/src/lib.rs
  - upbit: ucel/crates/ucel-cex-upbit/src/lib.rs;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.accounts.list.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.deposits.address.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.deposits.list.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.keys.list.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.orders.cancel.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.orders.chance.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.orders.closed.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.orders.create.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.orders.open.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.service.walletstatus.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.travelrule.vasps.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.withdraws.coin.json;ucel/crates/ucel-cex-upbit/tests/fixtures/exchange.private.rest.withdraws.list.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.candles.days.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.candles.minutes.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.candles.months.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.candles.weeks.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.candles.years.json;ucel/crates/ucel-cex-upbit/tests/fixtures/quotation.public.rest.markets.list.json

5.2 “tested=true” credibility（現状の信頼度）

| exchange | manifest tested=true ratio | test files found | mapping confidence | verdict | severity |
|---|---|---|---|---|---|
| BYBIT | 96/96 (100%) | YES | HIGH | PASS | NONE |
| binance | 14/14 (100%) | YES | HIGH | PASS | NONE |
| binance-coinm | 25/25 (100%) | YES | HIGH | PASS | NONE |
| binance-options | 14/14 (100%) | YES | HIGH | PASS | NONE |
| binance-usdm | 16/16 (100%) | YES | HIGH | PASS | NONE |
| bitbank | 44/44 (100%) | YES | HIGH | PASS | NONE |
| bitflyer | 61/61 (100%) | YES | HIGH | PASS | NONE |
| bitget | 2/2 (100%) | YES | HIGH | PASS | NONE |
| bitmex | 95/95 (100%) | YES | HIGH | PASS | NONE |
| bittrade | 34/34 (100%) | YES | HIGH | PASS | NONE |
| bybit | 77/77 (100%) | YES | HIGH | PASS | NONE |
| coinbase | 15/15 (100%) | YES | HIGH | PASS | NONE |
| coincheck | 29/29 (100%) | YES | HIGH | PASS | NONE |
| deribit | 28/28 (100%) | YES | HIGH | PASS | NONE |
| gmocoin | 42/42 (100%) | YES | HIGH | PASS | NONE |
| htx | 22/22 (100%) | YES | HIGH | PASS | NONE |
| kraken | 20/20 (100%) | YES | HIGH | PASS | NONE |
| okx | 7/7 (100%) | YES | HIGH | PASS | NONE |
| sbivc | 0/0 | YES | HIGH | PASS | NONE |
| upbit | 29/29 (100%) | YES | HIGH | PASS | NONE |

6. Library-grade Policy Scans（ucel/** 全体のポリシー監査）

6.1 Hardcoded exchange lists（手書き取引所列挙）
- Found: YES
- Evidence:
  - /tmp/hardcoded_lists_1.txt: 23 lines（主に `docs/exchanges/.../catalog.json` 参照）
  - /tmp/hardcoded_lists_2.txt: 36 lines（主に Cargo.toml の features 配列ヒット）
- Severity: MED（ランタイム固定列挙の直接証拠は限定的）

6.2 Secrets exposure risk（秘密漏洩の危険）
- Found patterns: YES
- Evidence (dangerous occurrences only):
  - 明確な secret 生値出力の確証は確認できず（多くは redaction テスト/マスク実装）
- Classification:
  - SAFE (field name only): 110
  - DANGEROUS (format!/println!/tracing!/error Display/Debug): 0（監査判断）
- Severity: LOW

6.3 Typed parsing compliance（serde_json::Value 逃げ）
- Value usage found: YES
- Evidence:
  - /tmp/value_usage.txt: 18 lines
  - ucel/crates/ucel-cex-binance-usdm/src/lib.rs:345:pub base_rules: serde_json::Value,
  - ucel/crates/ucel-cex-binance-usdm/src/lib.rs:539:if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
  - ucel/crates/ucel-cex-binance-coinm/src/lib.rs:1047:let catalog: serde_json::Value = serde_json::from_str(&raw).unwrap();
  - ucel/crates/ucel-cex-htx/tests/rest_contract.rs:393:let catalog: serde_json::Value = serde_json::from_str(&raw_catalog).unwrap();
  - ucel/crates/ucel-cex-sbivc/src/lib.rs:774:let manifest: serde_yaml::Value = serde_yaml::from_str(
- Classification:
  - Quarantined + documented: UNKNOWN
  - Used in exchange parsing path: YES
- Severity: MED

6.4 Feature gates impacting exchanges（featureで取引所挙動が変わる）
- Feature usage found: YES
- Summary:
  - Declared features (workspace): Cargo dependency feature flags across multiple crates
  - Where used: /tmp/feature_usage.txt (80 lines)
- Risk:
  - feature combination may break exchange support: UNKNOWN

6.5 MSRV signals（最低Rustバージョン）
- rust-toolchain present: NO path N/A
- Cargo rust-version set: NO value N/A (ucel/** scope)
- Conflicts found: NO
- Severity: LOW

6.6 Docs / Examples signals（利用者導線）
- Crate-level docs presence: YES (evidence ucel/README.md)
- examples present: NO (ucel/**/examples none)
- doc tests referenced: YES (`cargo test --doc -q` executed)
- Severity: MED

7. Build/Test Sanity（監査の信頼性のための実行結果）

7.1 cargo test
- Command: cd ucel && cargo test -q
- Result: FAIL
- If FAIL: error summary (first 30 lines max):

```text
error: duplicate key
  --> crates/ucel-cex-binance-usdm/Cargo.toml:13:1
   |
13 | reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
   | ^^^^^^^
error: failed to load manifest for workspace member `/home/runner/work/profinaut/profinaut/ucel/crates/ucel-cex-binance-usdm`
referenced by workspace at `/home/runner/work/profinaut/profinaut/ucel/Cargo.toml`
```

7.2 cargo test --doc
- Command: cd ucel && cargo test --doc -q
- Result: FAIL
- If FAIL: error summary (first 30 lines max):

```text
error: duplicate key
  --> crates/ucel-cex-binance-usdm/Cargo.toml:13:1
   |
13 | reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
   | ^^^^^^^
error: failed to load manifest for workspace member `/home/runner/work/profinaut/profinaut/ucel/crates/ucel-cex-binance-usdm`
referenced by workspace at `/home/runner/work/profinaut/profinaut/ucel/Cargo.toml`
```

8. Findings（指摘事項：優先度順）

8.1 CRITICAL
- 該当なし

8.2 HIGH
- [HIGH-001] coverage exchange `BYBIT` の catalog.json が exact path 不一致
  - Why: coverage検出名と docs/exchanges パス表記が不一致（BYBIT vs bybit）
  - Evidence: /tmp/missing_catalog.txt: `BYBIT docs/exchanges/BYBIT/catalog.json`
  - Impact: 自動監査/整合チェックで欠落扱い
  - Minimal fix: coverage ファイル名または catalog ディレクトリ名を一意な小文字に統一
- [HIGH-002] catalog_ids と manifest_ids が多数の取引所で不一致
  - Why: ID set equality が FAIL（18/20）
  - Evidence: /tmp/audit_idset_summary.tsv
  - Impact: coverage strict 判定時に誤検知/漏れを招く
  - Minimal fix: exchangeごとに manifest entries を catalog ID 全件へ同期

8.3 MED
- [MED-001] serde_json::Value が交換所パース経路で複数使用
  - Why: Typed schema 回避の可能性
  - Evidence: /tmp/value_usage.txt
  - Impact: 契約破壊変更の検知が遅れる
  - Minimal fix: 重要応答から順に型定義へ置換し、Value 使用を隔離

8.4 LOW
- [LOW-001] examples 不在（ucel/**/examples なし）
  - Why: 利用者導線が弱い
  - Evidence: glob `ucel/**/examples/**` no matches
  - Impact: 導入時の誤用リスク増
  - Minimal fix: 最低1つの公開APIサンプルを examples に追加

9. Verdict（最終判定）
- Verdict: FAIL
- Rationale:
  - HIGH finding（catalog欠落相当/ID集合不一致）が存在
  - Build/Test sanity も既存 manifest エラーで FAIL
- Blocking items:
  - HIGH-001, HIGH-002

10. Remediation Plan（最小修正計画：優先度順）
1. coverage/case naming 正規化
   - Scope: ucel/coverage/BYBIT.yaml, docs/exchanges/bybit/catalog.json, 参照箇所
   - Goal: exchange key の一意化（lowercase）
   - Verification: `find ucel/coverage ...`, `test -f docs/exchanges/bybit/catalog.json`, ID再比較
2. manifest ID 同期
   - Scope: ucel/coverage/*.yaml（FAIL exchange）
   - Goal: catalog_ids == manifest_ids
   - Verification: `/tmp/audit_idset_summary.tsv` 再生成で全PASS
3. Value隔離/型付け段階移行
   - Scope: /tmp/value_usage.txt 記載箇所
   - Goal: パース経路の Value を削減
   - Verification: `rg "serde_json::Value|:\s*Value\b" ucel` で減少確認

11. Appendix（付録：監査証跡の参照）
- /tmp/ucel_files.txt（ucel全ファイル一覧）
- /tmp/ucel_coverage_manifests.txt（coverageファイル一覧）
- /tmp/exchanges_from_coverage.txt（coverage由来の取引所一覧）
- /tmp/rg_hits_by_exchange.txt（コード中の取引所参照）
- /tmp/missing_catalog.txt（catalog欠落）
- /tmp/hardcoded_lists_*.txt（手書き列挙の検出）
- /tmp/secret_patterns.txt（secret関連パターン）
- /tmp/value_usage.txt（serde_json::Value使用）
- /tmp/feature_usage.txt（feature使用）
- /tmp/msrv_signals.txt（MSRVシグナル）
- /tmp/audit_exchange_<ex>.txt（取引所別の抽出/差分）
