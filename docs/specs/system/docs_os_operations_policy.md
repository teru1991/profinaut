# Docs OS 運用規約 v1.0（運用憲法）
Document ID: SYS-DOCS-OS-OPS
Status: Canonical (Operations Rulebook)
Scope: Docs OS を迷わず運用し、SSOT分裂とファイル乱立を防ぐ

---

## 0. ゴール
- **正本（SSOT）が1つに決まっている**状態を維持する  
- **「どこに何を書くか」迷わない**  
- **追加しても壊れない**（リンク切れ・重複・古い資料の参照を防ぐ）

---

## 1. 最重要：書く場所のルール（これだけ覚えればOK）
### A) 固定仕様（Core / Spec）＝意味が変わらないもの
**書く場所：`docs/specs/**`**  
- 「こうあるべき（不変）」  
- 「安全上の不変条件」  
- 「意味が変わると互換が壊れる」  

**禁止**：閾値・保持期間・通知先・運用時間帯など“値”は書かない（Policyへ）

---

### B) 契約（Schema）＝機械可読で唯一の正
**書く場所：`docs/contracts/**`**  
- JSON Schema（audit_eventなど）
- READMEは最小でOK（説明の増殖禁止）

---

### C) 運用値（Policy）＝値だけ調整したいもの
**書く場所：`docs/policy/**`**  
例：しきい値、保持期間、上限、確認窓、通知フラグ

**ルール**
- Policyは **値だけ**（意味や設計はSpec）
- 根拠が必要なら短く「参照先Spec」をリンクする

---

### D) 計画（Plan）＝順番・進捗・フェーズ
**書く場所：`docs/plans/**`**  
- 進捗・順序・フェーズはPlanへ  
- “到達点（不変）”はSpec、“いつやるか”はPlan

---

### E) 手順（Runbook）＝どう直すか／どう運用するか
**書く場所：`docs/runbooks/**`**  
- 「観測→自動復旧→手動介入→証拠→再発防止」の型で揃える

---

### F) 決定（Decisions）＝方針のログ
**書く場所：`docs/decisions/decisions.md`**  
- 「なぜそうしたか」を短く残す（後で迷わない）

---

### G) リンクSSOT（Trace Index）＝参照の正本
**書く場所：`docs/status/trace-index.json`**  
- 「どれが正本か」のリンクをここに集約  
- READMEより機械的に追える

---

### H) 旧資料（Legacy）＝正として使わない隔離
**書く場所：`docs/legacy/**`**  
- 正本ではない  
- 必要なら stub（NOT CANONICAL）で正本へ誘導

---

### I) Context（補助）＝正本ではない
**書く場所：`docs/context/**`**

---

## 2. 新規ファイル作成のルール（乱立防止の中核）
### 2.1 追加前のチェック（必須）
新しいファイルを作る前に必ず：
1) `docs/specs/system/ssot_map.md` を見る（該当カテゴリがあるか）
2) `docs/status/trace-index.json` を見る（既に同目的の正本がないか）
3) docs全体を検索（同じテーマがないか）

同じテーマがあったら：
- **追記**が基本（増やさない）
- どうしても分割するなら “親README（Index）” を作ってそこに統合

---

### 2.2 命名ルール（迷子防止）
- 役割が分かる名前
- 1ファイル1テーマ
- spec: `*_spec.md`  
- policy: `*.toml`（推奨）  
- plan: `*.md`  
- runbook: `*_runbook.md` / `*_playbook.md`

---

### 2.3 正本の移動ルール（参照切れゼロ）
移動時は必ず：
- `git mv` を使う
- 旧パスに **NOT CANONICAL stub** を置く（正本リンクのみ）
- trace-index を更新する

stubテンプレ：
```md
# NOT CANONICAL
このファイルは正本ではありません。
✅ 正本: <path>


⸻

3. 更新ルール（どれを更新すると何が起きるか）
	•	Specを変える：影響大（意味が変わり得る）
	•	Policyを変える：値調整（SpecのSemVerは動かさない）
	•	Planを変える：順序や進捗（自由に更新OK）
	•	Runbookを変える：運用改善（自由に更新OK）

⸻

4. 迷ったときの判断フロー（最重要）
	•	“こうあるべき（不変）” → Spec
	•	“しきい値・保持・回数・上限など数値” → Policy
	•	“いつやる・どう進める” → Plan
	•	“どう直す・どう対応する” → Runbook
	•	“なぜそうした” → Decisions
	•	“古いが捨てられない” → Legacy
	•	“補助” → Context

⸻

5. ここまで作成した各ドキュメントの意図（役割）

5.1 Crosscut（横断固定仕様）
	•	safety_interlock_spec.md
安全状態（NORMAL/SAFE/EMERGENCY_STOP）と危険操作の確認導線を固定。事故時は安全側へ倒す最終防壁。
	•	audit_replay_spec.md
“証明できること” を固定。入力範囲・参照・再現の成立条件を定義。
	•	support_bundle_spec.md
障害時の証拠パッケージを固定。secret-freeで提出・調査できる。

5.2 System（運用の憲法）
	•	ssot_map.md
どこが正本か、どこに何を書くかの地図。迷子防止。
	•	docs_gate_spec.md
docs品質ゲートの仕様。リンク切れ・SSOT分裂・secret混入を防ぐ。
	•	environment_mode_matrix.md
dev/stage/prod と paper/shadow/live の意味を固定。live誤爆防止。

5.3 Security
	•	identity_access_spec.md
認可・権限・secret_refなどセキュリティ境界を固定。
	•	data_classification_handling_spec.md
public/internal/restricted を固定し、ログ/監査/バンドル/エクスポート事故を防ぐ。

5.4 UCEL
	•	docs/specs/ucel/**
profinautにおけるUCEL利用境界仕様（connector/spec）。
	•	docs/specs/ucel_golden/** + docs/policy/ucel_golden/** + docs/plans/ucel/**
UCEL Golden（思想/固定仕様/運用値/計画）の正本をこのリポ内で保持。
	•	compatibility_notes.md
“どっちが正本か” をケース別に固定し、SSOT分裂を防ぐ。

5.5 Policy / Plans / Runbooks
	•	docs/policy/**
運用値を調整する場所（Specの意味は変えない）。
	•	docs/plans/**
進め方・フェーズ・ロードマップ（変更前提）。
	•	docs/runbooks/**
障害時手順。テンプレ型で迷子を防ぐ。

⸻

6. 日常運用ルーチン（おすすめ）
	1.	新しい作業の前に ssot_map を見る
	2.	追加前に検索して既存がないか確認
	3.	迷ったら判断フローに従う
	4.	正本移動は git mv + stub + trace-index 更新
	5.	“増やす前に統合” を徹底する

⸻

End of document
