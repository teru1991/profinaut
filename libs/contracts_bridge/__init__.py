"""
contracts_bridge – legacy → new SSOT conversion layer.

Public API:
    map_legacy_safe_mode_to_safety_state  – legacy safe_mode → SafetyState dict
    map_legacy_command_to_safety_state    – legacy strong command → SafetyState dict
    map_legacy_auditlog_to_audit_event    – legacy AuditLog → AuditEvent dict
    redact_obj_fail_closed                – fail-closed redaction for any object
"""

from .audit_bridge import map_legacy_auditlog_to_audit_event
from .redaction import (
    DEFAULT_FORBIDDEN_KEYS,
    DEFAULT_FORBIDDEN_REGEX,
    redact_obj_fail_closed,
)
from .safety_bridge import (
    map_legacy_command_to_safety_state,
    map_legacy_safe_mode_to_safety_state,
)

__all__ = [
    "map_legacy_safe_mode_to_safety_state",
    "map_legacy_command_to_safety_state",
    "map_legacy_auditlog_to_audit_event",
    "redact_obj_fail_closed",
    "DEFAULT_FORBIDDEN_KEYS",
    "DEFAULT_FORBIDDEN_REGEX",
]
