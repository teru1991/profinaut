from fastapi import Header, HTTPException, status

from .config import get_settings


def require_admin_actor(x_admin_token: str | None = Header(default=None, alias="X-Admin-Token")) -> str:
    settings = get_settings()
    if not x_admin_token or x_admin_token != settings.admin_token:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Unauthorized")
    return "admin-token-user"
