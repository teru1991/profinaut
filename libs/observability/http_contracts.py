from __future__ import annotations

from fastapi import Request

from contracts.observability.contract_constants import (
    SCHEMA_VERSION_CAPABILITIES,
    SCHEMA_VERSION_CORRELATION,
    SCHEMA_VERSION_HEALTHZ,
)
from libs.observability.contracts import (
    CapabilitiesResponse,
    CapabilityFeature,
    Correlation,
    HealthCheck,
    HealthStatus,
    HealthzResponse,
)
from libs.observability.correlation import make_correlation, now_utc_iso, set_correlation_response_headers


def _aggregate_health_status(checks: list[HealthCheck]) -> HealthStatus:
    if any(check.status == HealthStatus.FAILED for check in checks):
        return HealthStatus.FAILED
    if any(check.status == HealthStatus.DEGRADED for check in checks):
        return HealthStatus.DEGRADED
    if any(check.status == HealthStatus.OK for check in checks):
        return HealthStatus.OK
    return HealthStatus.UNKNOWN


def build_healthz_response(request: Request, checks: list[HealthCheck], op: str = "healthz") -> tuple[dict, dict]:
    if not checks:
        checks = [
            HealthCheck(
                name="self",
                status=HealthStatus.UNKNOWN,
                reason_code="UNKNOWN",
                summary="no checks emitted",
                observed_at=now_utc_iso(),
            )
        ]
    correlation_raw = make_correlation(op=op, request_headers=dict(request.headers))
    response = HealthzResponse(
        status=_aggregate_health_status(checks),
        checks=checks,
        correlation=Correlation(**correlation_raw),
    )
    headers: dict[str, str] = {}
    set_correlation_response_headers(headers, correlation_raw, [SCHEMA_VERSION_CORRELATION, SCHEMA_VERSION_HEALTHZ])
    return response.to_dict(), headers


def build_capabilities_response(
    request: Request, features: list[CapabilityFeature], op: str = "capabilities"
) -> tuple[dict, dict]:
    if not features:
        features = [
            CapabilityFeature(
                name="capabilities",
                state="NOT_IMPLEMENTED",
                reasons=[{"code": "NOT_IMPLEMENTED", "message": "no feature list provided"}],
            )
        ]
    correlation_raw = make_correlation(op=op, request_headers=dict(request.headers))
    response = CapabilitiesResponse(
        features=features,
        correlation=Correlation(**correlation_raw),
    )
    headers: dict[str, str] = {}
    set_correlation_response_headers(
        headers,
        correlation_raw,
        [SCHEMA_VERSION_CORRELATION, SCHEMA_VERSION_CAPABILITIES],
    )
    return response.to_dict(), headers
