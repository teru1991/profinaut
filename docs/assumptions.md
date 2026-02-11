# Assumptions

## Step 0 defaults
1. Development starts on Windows 11 with Docker Desktop installed and WSL2 backend enabled.
2. Team members on macOS/Linux can use npm scripts that proxy to Docker Compose commands.
3. Step 0 creates runnable infrastructure placeholders only; application runtime implementation starts in Step 1 and Step 2.
4. PostgreSQL is the sole control-plane/state database for MVP.
5. Time values across backend storage will be UTC ISO-8601 once API contracts are introduced in Step 1.
6. Admin authentication in MVP uses `X-Admin-Token` sourced from `.env` (implemented in Step 2).
7. Discord support before Step 7 is outbound webhook notifications only.
