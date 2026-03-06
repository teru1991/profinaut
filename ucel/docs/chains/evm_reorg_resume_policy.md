# EVM Reorg Resume Policy

- Logs cursor stores block boundary and hash.
- Removed logs or hash mismatch indicate reorg.
- Replay range starts at rollback block and re-fetches deterministically.
- Resume path deduplicates logs by tx_hash + log_index.
