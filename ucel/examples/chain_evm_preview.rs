use ucel_chain_ethereum::{
    estimate_eip1559, get_native_balance, DeterministicTestSigner, EvmProviderInfo, EvmProviderSet,
    EvmReasonCode, FeePolicy, ProviderError, ProviderResponse,
};
use ucel_chain_ethereum::provider::{EvmHttpProvider, EvmWsProvider};
use ucel_core::{EvmAddress, EvmBlockRef, EvmChainId};

struct DemoHttp(EvmProviderInfo);
impl EvmHttpProvider for DemoHttp {
    fn info(&self) -> &EvmProviderInfo { &self.0 }
    fn rpc_call(&self, method: &str, _params: serde_json::Value) -> Result<ProviderResponse, ProviderError> {
        let v = match method {
            "eth_getBalance" => serde_json::json!("0x64"),
            "eth_chainId" => serde_json::json!("0x1"),
            _ => serde_json::json!(null),
        };
        Ok(ProviderResponse { provider: self.0.id.clone(), method: method.to_string(), value: v })
    }
}
struct DemoWs(EvmProviderInfo);
impl EvmWsProvider for DemoWs {
    fn info(&self) -> &EvmProviderInfo { &self.0 }
    fn subscribe(&self, _method: &str, _params: serde_json::Value) -> Result<String, ProviderError> { Ok("demo-sub".into()) }
    fn unsubscribe(&self, _id: &str) -> Result<(), ProviderError> { Ok(()) }
}

fn main() {
    let set = EvmProviderSet {
        primary_http: Box::new(DemoHttp(EvmProviderInfo { id: "demo".into(), chain_id: EvmChainId(1), priority: 0 })),
        fallback_http: vec![],
        primary_ws: Some(Box::new(DemoWs(EvmProviderInfo { id: "demo-ws".into(), chain_id: EvmChainId(1), priority: 0 }))),
        fallback_ws: vec![],
        policy: Default::default(),
    };
    let bal = get_native_balance(&set, EvmChainId(1), EvmAddress("0x1111111111111111111111111111111111111111".into()), EvmBlockRef::Latest).unwrap();
    let fee = estimate_eip1559(10, 2, 21000, FeePolicy::default()).unwrap();
    let _signer = DeterministicTestSigner { signer_id: "preview".into() };
    println!("balance_wei={} max_fee={:?} reason_sample={:?}", bal.wei, fee.max_fee_per_gas, EvmReasonCode::ProviderTimeout);
}
