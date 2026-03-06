# UCEL Equity Data Surface v1

## Canonical operations
- get_quote
- get_last_trade (optional by vendor)
- get_bars
- list_symbols
- get_market_calendar
- get_corporate_actions
- resolve_symbol

## Canonical models
- EquitySymbol
- EquityQuote
- EquityBar
- EquityExchangeCode
- EquityMarketCalendar
- EquitySessionWindow
- EquityCorporateAction / EquitySplit / EquityDividend
- EquityVendorLatencyClass

## Core policies
- vendor capability registry (quotes/bars/calendar/corporate-actions/realtime/delayed)
- symbol mapping must fail on ambiguity
- timezone/session are required for market-calendar semantics
- delayed must never be reported as realtime
