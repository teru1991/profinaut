from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import json
import math
import re
from typing import Any

# NOTE:
# - このモジュールは “秘密が出力面へ出ない” を最優先のSSOTとして扱う。
# - redact(): 出力用（マスク）
# - scan_*(): 出力前の検査用（Fail-closedの根拠）
#
# 重要: ここにあるルールは “広くマスク” で安全側に倒す。
#       後続タスク（B-STEP2..）で policy により一部緩和/例外を設ける場合でも、
#       この基礎層自体は “秘密を見逃さない” 方向で固定する。

# Key-based detection (common)
_SECRET_KEY_RE = re.compile(r"(?i)\b(token|secret|private[_-]?key|api[_-]?key|password|passphrase|authorization)\b")

# Content-based detection
# JWT-like: header.payload.signature (base64url parts)
_JWT_RE = re.compile(r"\beyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\b")

# PEM private key blocks
_PEM_PRIVATE_RE = re.compile(r"-----BEGIN (?:EC |RSA |OPENSSH |)PRIVATE KEY-----[\s\S]+?-----END (?:EC |RSA |OPENSSH |)PRIVATE KEY-----")

# Authorization header patterns (Bearer/Basic)
_AUTH_BEARER_RE = re.compile(r"(?i)\bBearer\s+([A-Za-z0-9\-\._~\+/]+=*)")
_AUTH_BASIC_RE = re.compile(r"(?i)\bBasic\s+([A-Za-z0-9\+/]+=*)")

# Long hex / hashes
_LONG_HEX_RE = re.compile(r"\b[a-fA-F0-9]{32,}\b")

# Base64-ish long blobs (avoid small strings)
_BASE64_LONG_RE = re.compile(r"\b[A-Za-z0-9\+/]{40,}={0,2}\b")

# “Address-like” (very loose; still masked)
_ADDRESS_LIKE_RE = re.compile(r"\b(?:0x)?[A-Za-z0-9]{20,}\b")

# Query param style secrets (token=..., api_key=..., etc)
_QUERY_SECRET_RE = re.compile(r"(?i)\b(token|secret|api[_-]?key|password|passphrase)\s*=\s*([^&\s]{6,})")

# Some known token shapes (best-effort; keep minimal here)
_SLACK_TOKEN_RE = re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b")
_GITHUB_TOKEN_RE = re.compile(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b")
_AWS_ACCESS_KEY_RE = re.compile(r"\bAKIA[0-9A-Z]{16}\b")
_AWS_SECRET_KEY_RE = re.compile(r"\b[0-9a-zA-Z\/+=]{40}\b")

# near-secret IDs (examples; treat as near-secret for redaction)
_ACCOUNT_ID_RE = re.compile(r"(?i)\b(account|acct)[\s_\-]?id[:=]?\s*([A-Za-z0-9_\-]{6,})")
_WHITELIST_RE = re.compile(r"(?i)\bwhitelist\b")


def _mask(text: str) -> str:
    if not text:
        return "***"
    if len(text) <= 6:
        return "***"
    return f"{text[:3]}***{text[-3:]}"


def _shannon_entropy(s: str) -> float:
    # conservative entropy estimation
    if not s:
        return 0.0
    freq = {}
    for ch in s:
        freq[ch] = freq.get(ch, 0) + 1
    n = len(s)
    ent = 0.0
    for c in freq.values():
        p = c / n
        ent -= p * math.log2(p)
    return ent


@dataclass(frozen=True, slots=True)
class RedactionFinding:
    kind: str
    match: str
    context_key: str | None
    severity: str  # "secret" | "near_secret"


def scan_text(text: str, *, context_key: str | None = None) -> list[RedactionFinding]:
    findings: list[RedactionFinding] = []
    if not isinstance(text, str) or not text:
        return findings

    def add(kind: str, m: str, severity: str) -> None:
        findings.append(RedactionFinding(kind=kind, match=_mask(m), context_key=context_key, severity=severity))

    # Secrets
    for m in _PEM_PRIVATE_RE.finditer(text):
        add("pem_private_key", m.group(0)[:32], "secret")
    for m in _JWT_RE.finditer(text):
        add("jwt", m.group(0), "secret")
    for m in _AUTH_BEARER_RE.finditer(text):
        add("authorization_bearer", m.group(1), "secret")
    for m in _AUTH_BASIC_RE.finditer(text):
        add("authorization_basic", m.group(1), "secret")
    for m in _QUERY_SECRET_RE.finditer(text):
        add(f"query_{m.group(1).lower()}", m.group(2), "secret")
    for m in _SLACK_TOKEN_RE.finditer(text):
        add("slack_token", m.group(0), "secret")
    for m in _GITHUB_TOKEN_RE.finditer(text):
        add("github_token", m.group(0), "secret")
    for m in _AWS_ACCESS_KEY_RE.finditer(text):
        add("aws_access_key_id", m.group(0), "secret")

    # Near-secrets / suspicious blobs
    # Long hex or base64 can be hashes or IDs; treat as near-secret unless proven otherwise.
    for m in _LONG_HEX_RE.finditer(text):
        add("long_hex", m.group(0), "near_secret")
    for m in _BASE64_LONG_RE.finditer(text):
        blob = m.group(0)
        # entropy gate: mask only if looks random-ish
        if len(blob) >= 48 and _shannon_entropy(blob) >= 4.0:
            add("base64_blob", blob, "near_secret")

    # Address-like (very broad)
    for m in _ADDRESS_LIKE_RE.finditer(text):
        # Avoid double-reporting if it was already covered by long_hex/base64/jwt
        token = m.group(0)
        if len(token) >= 24:
            add("address_like", token, "near_secret")

    # Account/whitelist markers (near-secret)
    for m in _ACCOUNT_ID_RE.finditer(text):
        add("account_id", m.group(2), "near_secret")
    if _WHITELIST_RE.search(text):
        findings.append(RedactionFinding(kind="whitelist_marker", match="whitelist", context_key=context_key, severity="near_secret"))

    return findings


def redact_text(text: str) -> str:
    if not isinstance(text, str) or not text:
        return text

    # PEM blocks => hard mask
    if _PEM_PRIVATE_RE.search(text):
        return "***REDACTED_PEM_PRIVATE_KEY***"

    # Mask known patterns in-string (keep structure)
    text = _AUTH_BEARER_RE.sub(lambda m: "Bearer " + _mask(m.group(1)), text)
    text = _AUTH_BASIC_RE.sub(lambda m: "Basic " + _mask(m.group(1)), text)
    text = _JWT_RE.sub(lambda m: _mask(m.group(0)), text)
    text = _SLACK_TOKEN_RE.sub(lambda m: _mask(m.group(0)), text)
    text = _GITHUB_TOKEN_RE.sub(lambda m: _mask(m.group(0)), text)
    text = _AWS_ACCESS_KEY_RE.sub(lambda m: _mask(m.group(0)), text)

    # token=... style
    text = _QUERY_SECRET_RE.sub(lambda m: f"{m.group(1)}={_mask(m.group(2))}", text)

    # Long blobs
    text = _LONG_HEX_RE.sub(lambda m: _mask(m.group(0)), text)

    def _mask_base64(m: re.Match[str]) -> str:
        blob = m.group(0)
        if len(blob) >= 48 and _shannon_entropy(blob) >= 4.0:
            return _mask(blob)
        return blob

    text = _BASE64_LONG_RE.sub(_mask_base64, text)
    text = _ADDRESS_LIKE_RE.sub(lambda m: _mask(m.group(0)) if len(m.group(0)) >= 24 else m.group(0), text)
    return text


def scan_obj(value: Any, *, context_key: str | None = None) -> list[RedactionFinding]:
    findings: list[RedactionFinding] = []
    if value is None:
        return findings

    if isinstance(value, Mapping):
        for k, v in value.items():
            ks = str(k)
            # key name itself indicates secret
            if _SECRET_KEY_RE.search(ks):
                findings.append(RedactionFinding(kind="secret_key_name", match=ks, context_key=context_key or ks, severity="secret"))
                # still scan value too (it may contain additional info)
            findings.extend(scan_obj(v, context_key=ks))
        return findings

    if isinstance(value, (list, tuple, set)):
        for v in value:
            findings.extend(scan_obj(v, context_key=context_key))
        return findings

    if isinstance(value, str):
        return scan_text(value, context_key=context_key)

    # numbers/bools => no scan
    return findings


def redact(value: Any) -> Any:
    # Backward compatible: recursive redaction for dict/list/tuple/str
    if isinstance(value, Mapping):
        out: dict[str, Any] = {}
        for k, v in value.items():
            ks = str(k)
            if _SECRET_KEY_RE.search(ks):
                out[ks] = "***REDACTED***"
            else:
                out[ks] = redact(v)
        return out
    if isinstance(value, list):
        return [redact(v) for v in value]
    if isinstance(value, tuple):
        return tuple(redact(v) for v in value)
    if isinstance(value, str):
        return redact_text(value)
    return value


def safe_str(value: Any) -> str:
    # For logging / exception message
    try:
        if isinstance(value, str):
            return redact_text(value)
        return json.dumps(redact(value), ensure_ascii=False, default=str)
    except Exception:
        # last-resort: do not leak
        return "***REDACTED_UNSERIALIZABLE***"


def safe_json(value: Any) -> str:
    # For payloads that will be emitted externally
    try:
        return json.dumps(redact(value), ensure_ascii=False, separators=(",", ":"), default=str)
    except Exception:
        return json.dumps({"error": "***REDACTED_UNSERIALIZABLE***"}, ensure_ascii=False)
