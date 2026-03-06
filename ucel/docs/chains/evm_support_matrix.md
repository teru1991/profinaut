# EVM Support Matrix

| surface | status | notes |
|---|---|---|
| read (chain/balance/call) | supported | with hex/address validation |
| tx build/sign/send | supported | signer trait + normalized errors |
| receipt/finality | supported | policy depth + timeout handling |
| logs/get_logs/subscribe | supported | cursor + dedup helpers |
| reorg/resume/replay | supported | rollback + replay range helpers |
