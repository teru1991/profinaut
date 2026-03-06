# UCEL Chain EVM Surface v1

## Canonical operations
- get_chain_id
- get_block_number
- get_native_balance
- get_erc20_balance
- estimate_gas
- estimate_fees
- get_nonce
- build_transaction
- sign_transaction
- send_raw_transaction
- wait_for_receipt
- get_receipt
- get_logs
- subscribe_logs
- subscribe_new_heads

## Canonical models
- EvmChainId / EvmAddress / EvmBlockRef
- EvmNativeBalance / EvmTokenBalance
- EvmTransactionRequest / EvmSignedTransaction / EvmTransactionReceipt
- EvmLogEvent / EvmLogCursor / EvmReorgEvent
- EvmFinalityState

## Safety policy anchors
- provider failover with chain id verification
- nonce reservation + refresh
- EIP-1559 and legacy fee policy with ceiling
- reorg-safe log cursor + replay range
