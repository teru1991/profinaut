from __future__ import annotations

from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from contracts.observability.contract_constants import (
    SCHEMA_VERSION_CAPABILITIES,
    SCHEMA_VERSION_CORRELATION,
    SCHEMA_VERSION_HEALTHZ,
)


class HealthStatus(str, Enum):
    OK = "OK"
    DEGRADED = "DEGRADED"
    FAILED = "FAILED"
    UNKNOWN = "UNKNOWN"


class FeatureState(str, Enum):
    ENABLED = "ENABLED"
    DISABLED = "DISABLED"
    DEGRADED = "DEGRADED"
    NOT_IMPLEMENTED = "NOT_IMPLEMENTED"


class ContractBaseModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    def to_dict(self) -> dict[str, Any]:
        return self.model_dump(by_alias=True, exclude_none=True)


class Correlation(ContractBaseModel):
    schema_version: str = Field(default=SCHEMA_VERSION_CORRELATION)
    run_id: str
    instance_id: str
    trace_id: str | None = None
    event_uid: str | None = None
    op: str
    emitted_at: str
    build: dict[str, str] | None = None
    env: dict[str, str] | None = None


class HealthCheck(ContractBaseModel):
    name: str
    status: HealthStatus
    reason_code: str
    summary: str
    observed_at: str
    details: dict[str, Any] | None = None


class HealthzResponse(ContractBaseModel):
    schema_version: str = Field(default=SCHEMA_VERSION_HEALTHZ)
    status: HealthStatus
    checks: list[HealthCheck]
    correlation: Correlation


class CapabilityReason(ContractBaseModel):
    code: str
    message: str
    since: str | None = None


class CapabilityFeature(ContractBaseModel):
    name: str
    state: FeatureState
    reasons: list[CapabilityReason] = Field(default_factory=list)


class CapabilitiesResponse(ContractBaseModel):
    schema_version: str = Field(default=SCHEMA_VERSION_CAPABILITIES)
    features: list[CapabilityFeature]
    correlation: Correlation


class ErrorContext(ContractBaseModel):
    component: str
    request_id: str | None = None
    trace_id: str | None = None
    run_id: str | None = None
    path: str | None = None
    method: str | None = None
    status_code: int | None = None
    upstream: str | None = None
    retry_after_ms: int | None = None


class StandardError(ContractBaseModel):
    code: str
    reason_code: str
    kind: str
    severity: str
    retryable: bool
    source: str
    context: ErrorContext
    message: str | None = None
    details: dict[str, Any] | None = None
    schema_version: str | None = None
    contract_version: str | None = None


class ErrorEnvelope(ContractBaseModel):
    error: StandardError
