from __future__ import annotations

from dashboard_api.security_mw import SecurityMiddleware

# Integration point for API entry security.
SECURITY_MW = SecurityMiddleware()
