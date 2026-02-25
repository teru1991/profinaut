# Versioning Policy (SSOT) v1.0
Status: Canonical
Scope: How contracts/specs/policy evolve without breaking SSOT

## 0. Core Principle
- Contracts and Core Specs define meaning and compatibility.
- Policy changes tune values without changing meaning.
- Plans and Runbooks change freely but must not contradict contracts/specs.

---

## 1. Contracts (docs/contracts) Versioning
- JSON Schemas are versioned via:
  - `$id` stability
  - `schema_version` field (const integer)
- Rules:
  - Backward-compatible changes: add optional fields, broaden constraints safely → MINOR
  - Breaking changes: remove/rename fields, tighten constraints, change enums meaning → MAJOR
- If a breaking change is necessary, introduce a new schema_version.

---

## 2. Core Specs (docs/specs) SemVer
- MAJOR: meaning change of invariants, boundaries, or safety semantics
- MINOR: additive rules, new optional conventions, new interlock categories
- PATCH: clarifications, examples, editorial fixes

Core specs must not encode tunable constants. Those belong to Policy.

---

## 3. Policy (docs/policy) Versioning
Policy changes must not change meaning of Core spec.
Policy may change:
- thresholds, windows, caps, retention, alert sensitivity
Policy changes should be tracked with a simple changelog entry inside the policy doc(s).

---

## 4. Plans / Runbooks
- Plans and runbooks can change at any time.
- They must reference the canonical specs/contracts and must not redefine them.

---

## 5. Migration rule
If legacy docs contain unique knowledge:
- migrate the normative part into contracts/specs first,
- then keep only a reference in legacy.
