# Access Review Report (Monthly)

## Summary
- allow rules: 3
- dangerous ops: 5
- controlled changes: 6
- registry items: 2

## Dangerous Ops Catalog
- failover_primary
- policy_update
- rotate_secret
- start_live
- withdraw_enable

## Controlled Changes (Extended Change Mgmt)
- ca_rotation
- cloudflare_access_change
- failover_primary
- policy_update
- rotate_secret
- withdraw_enable

## Allow Rules
- op=read_status actor=human modes=[dev, staging, prod] scopes=[*]
- op=start_live actor=human modes=[dev, staging] scopes=[bot:example:paper]
- op=rotate_secret actor=human modes=[dev, staging] scopes=[*]

## Asset Registry
- example.exchange.readonly: schemes=[fileenc, vault] scopes=[venue:example:readonly] max_ttl=30
- example.dev.env_only: schemes=[env] scopes=[venue:example:dev] max_ttl=10
