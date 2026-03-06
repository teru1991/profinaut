# EVM Nonce and Fee Policy

- Nonce scope = chain_id + account.
- Nonce manager reserves locally and can rollback/refresh.
- Fee estimation supports EIP-1559 and legacy gasPrice.
- Fee ceiling/spike guard enforces fail-closed behavior.
