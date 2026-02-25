# On-chain Finality / Reorg Playbook v1.0

## 目的
オンチェーンはreorgが起きる前提で、finalityに沿って安全に扱う。
RPC不一致やfinality遅延を検知し、資金を守る。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/onchain_connector_spec.md`
- 固定仕様: `docs/specs/crosscut/safety_interlock_spec.md`（実行する場合）

---

## 0. 原則
- finalizedのみ確定扱い
- reorgは異常ではなく常態（ただし多発は危険）
- RPC不一致は重大（観測/実行ともに停止を検討）

---

## 1. 入口：事象タイプ
- [ ] finality_lag増大（確定が遅い）
- [ ] reorg_detected増大
- [ ] RPC不一致（同block_numberでhash違い等）
- [ ] txがpendingのまま（置換/失敗）

---

## 2. finality_lag増大
対応：
- [ ] RPCエンドポイントを切替（冗長化）
- [ ] 観測のみならSAFE_MODE相当（実行は止める）
- [ ] 実行があるなら HALT を検討（不確実が増大）

---

## 3. reorg_detected多発
対応：
- [ ] finality_statusの扱いを確認（pending/confirmed/finalized/reorged）
- [ ] supersedes関係が記録されていること
- [ ] 実行があるなら SAFE_MODE/HALT（状況により）

監査：
- reorgを重要イベントとして監査（将来type拡張可）

---

## 4. RPC不一致（最重大）
対応：
- [ ] 直ちに実行系をHALT（資金保護）
- [ ] RPCを変更/縮退（観測のみ）
- [ ] 不一致期間を監査とSupport Bundleへ残す

---

## 5. tx pending / 置換
対応：
- [ ] replace-by-fee前提で状態を再照会
- [ ] tx_hashの置換関係（もしあるなら）を記録
- [ ] 危険ならHALT

---

## 6. 収束判定
- [ ] finality_lagが許容内
- [ ] RPC一致
- [ ] reorg多発が収束
- [ ] Safety解除は強操作（理由/TTL/監査）
