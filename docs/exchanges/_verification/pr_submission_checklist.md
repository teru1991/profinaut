# PR Submission Checklist for Exchange Verification

このチェックリストは、`docs/exchanges/_verification` で作成した監査結果を
Pull Request に安全に出すための最小手順です。

## 1. 変更単位
- [ ] 1タスク（EXD-00x）ごとに1コミット
- [ ] 対象取引所は1つだけ（EXD-002〜008は取引所単位で直列）

## 2. 事前確認
- [ ] `docs/exchanges/_verification/reports/<exchange>.md` が更新済み
- [ ] `docs/exchanges/_verification/evidence/<exchange>/` に証拠がある
- [ ] `docs/exchanges/_verification/index.md` のステータス/Current Task を更新
- [ ] APIキー等の秘匿情報が含まれていない（必ずマスク）

## 3. 推奨コマンド
```bash
git status --short
git add <changed-files>
git commit -m "docs(exchanges): EXD-00x <exchange> <summary>"
```

## 4. PR本文に必ず含める内容
- 実施したEXDタスク番号
- 公式一次URL（確認日付き）
- 変更ファイル一覧
- スモークテスト結果（成功/失敗理由）
- 残課題（次タスク）

## 5. 完了条件
- [ ] コミット済み
- [ ] PR作成済み
- [ ] 作業ツリーが clean (`git status --short` が空)
