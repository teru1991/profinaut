import os

from fastapi import Header, HTTPException, status


def require_execution_token(x_execution_token: str | None = Header(default=None, alias="X-Execution-Token")) -> str:
    """Require X-Execution-Token header for sensitive execution endpoints."""
    expected_token = os.getenv("EXECUTION_API_TOKEN", "")
    if not expected_token:
        # If no token is configured, this is a configuration error
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, 
            detail="Execution API token not configured"
        )
    if not x_execution_token or x_execution_token != expected_token:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, 
            detail="Unauthorized"
        )
    return "execution-token-user"
