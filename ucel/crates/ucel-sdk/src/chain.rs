use ucel_chain_ethereum::{
    build_transaction, estimate_eip1559, get_native_balance, sign_transaction, wait_for_receipt,
    DeterministicTestSigner, EvmProviderSet, FeePolicy, FinalityPolicy,
};
use ucel_core::{EvmAddress, EvmBlockRef, EvmChainId, UcelError};

pub struct ChainFacade {
    pub providers: EvmProviderSet,
    pub chain_id: EvmChainId,
}

impl ChainFacade {
    pub fn get_native_balance(&self, address: EvmAddress) -> Result<u128, UcelError> {
        Ok(get_native_balance(&self.providers, self.chain_id, address, EvmBlockRef::Latest)?.wei)
    }

    pub fn preview_capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "chain": "evm",
            "chain_id": self.chain_id.0,
            "surfaces": ["get_balance", "estimate_fees", "build_sign_send", "receipt", "logs"]
        })
    }

    pub fn tx_preview(
        &self,
        from: EvmAddress,
        to: EvmAddress,
        nonce: u64,
    ) -> Result<String, UcelError> {
        let fee = estimate_eip1559(10, 2, 21_000, FeePolicy::default())?;
        let tx = build_transaction(
            self.chain_id,
            from,
            Some(to),
            "0x".to_string(),
            0,
            21_000,
            fee,
            nonce,
        )?;
        let signer = DeterministicTestSigner {
            signer_id: "sdk-preview".into(),
        };
        let signed = sign_transaction(&signer, &tx)?;
        Ok(signed.tx_hash)
    }

    pub fn wait_receipt_preview(&self, tx_hash: &str) -> Result<bool, UcelError> {
        let receipt = wait_for_receipt(
            &self.providers,
            self.chain_id,
            tx_hash,
            100,
            FinalityPolicy::default(),
            1,
        )?;
        Ok(receipt.success)
    }
}
