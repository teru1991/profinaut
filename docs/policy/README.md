# docs/policy（運用値 / 可変）

このディレクトリは **Policy（運用値）** を管理します。  
Core Spec（固定仕様）を変更せず、環境や運用で調整したい「値」だけをここに集約します。

## ルール
- Policyは **契約（docs/contracts）** と **固定仕様（docs/specs）** を破壊しない
- 値の変更は許容されるが、変更理由と影響（何が変わるか）を記録する
- Plan/RunbookはPolicyを参照して運用する

## 代表例
- 閾値（SLO、stale、gap、thin、WAL、disk、clock、obs欠損など）
- 禁止キー（redaction）
- 保持/圧縮/退避（retention）
- リモート閲覧（Cloudflare Access/Grafana）
