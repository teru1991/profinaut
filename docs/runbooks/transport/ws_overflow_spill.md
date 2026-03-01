# Runbook: Outbound overflow / spill-to-disk

## Symptoms
- outq_dropped/outq_spilled が増える
- disk 使用量が増える（spill_dir）
- /healthz が Degraded（OUTQ_DROPPING / SPILL_ACTIVE）

## What to check
- metrics: outq_len, outq_dropped, outq_spilled
- events_tail: overflow outcome
- spill_dir のファイル数/容量

## Immediate actions
1) rules: overflow.mode を SlowDown 系へ（Drop→SlowDownへ）
2) public_rps を下げて負荷低減、private 優先が維持されることを確認
3) disk逼迫なら spill_dir ローテ/容量確保（運用手順）

## Recovery
- outq_len が cap 近傍から戻る
- drop/spill が落ち着く
