# Equity Corporate Action Policy

- Corporate actions include split/reverse-split/dividend/symbol-change/delist.
- Bars/quotes publish adjustment metadata (`raw` / `split_adjusted` / `split_dividend_adjusted`).
- Corporate actions in equity adapter are market-data adjustments, not IR/disclosure events.
- Overlapping actions are sorted and merged deterministically.
