use super::{ChannelKey, ExchangeId, OperationKey};
use crate::{EndpointSpec, ExchangeCatalog, WsChannelSpec};
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use super::errors::HubError;

#[derive(Debug)]
pub struct SpecRegistry {
    rest: HashMap<(ExchangeId, OperationKey), EndpointSpec>,
    ws: HashMap<(ExchangeId, ChannelKey), WsChannelSpec>,
    rest_keys: HashMap<ExchangeId, BTreeSet<OperationKey>>,
    ws_keys: HashMap<ExchangeId, BTreeSet<ChannelKey>>,
}

impl SpecRegistry {
    pub fn global() -> Result<&'static Self, HubError> {
        static REG: OnceLock<Result<SpecRegistry, HubError>> = OnceLock::new();
        REG.get_or_init(Self::build)
            .as_ref()
            .map_err(|e| HubError::RegistryValidation(e.to_string()))
    }

    fn build() -> Result<SpecRegistry, HubError> {
        let mut rest = HashMap::new();
        let mut ws = HashMap::new();
        let mut rest_keys: HashMap<ExchangeId, BTreeSet<OperationKey>> = HashMap::new();
        let mut ws_keys: HashMap<ExchangeId, BTreeSet<ChannelKey>> = HashMap::new();

        for (exchange, catalog) in exchange_catalogs()? {
            for spec in catalog.rest_endpoints {
                let key = spec.id.clone();
                let idx = (exchange, key.clone());
                if rest.insert(idx, spec).is_some() {
                    return Err(HubError::RegistryValidation(format!(
                        "duplicate rest key for {exchange:?}:{key}"
                    )));
                }
                rest_keys.entry(exchange).or_default().insert(key);
            }
            for spec in catalog.ws_channels {
                let key = spec.id.clone();
                let idx = (exchange, key.clone());
                if ws.insert(idx, spec).is_some() {
                    return Err(HubError::RegistryValidation(format!(
                        "duplicate ws key for {exchange:?}:{key}"
                    )));
                }
                ws_keys.entry(exchange).or_default().insert(key);
            }
        }

        Ok(Self {
            rest,
            ws,
            rest_keys,
            ws_keys,
        })
    }

    pub fn resolve_rest(&self, exchange: ExchangeId, key: &str) -> Result<&EndpointSpec, HubError> {
        self.rest
            .get(&(exchange, key.to_string()))
            .ok_or_else(|| HubError::UnknownOperation {
                exchange: exchange.as_str().to_string(),
                key: key.to_string(),
            })
    }

    pub fn resolve_ws(&self, exchange: ExchangeId, key: &str) -> Result<&WsChannelSpec, HubError> {
        self.ws
            .get(&(exchange, key.to_string()))
            .ok_or_else(|| HubError::UnknownChannel {
                exchange: exchange.as_str().to_string(),
                key: key.to_string(),
            })
    }

    pub fn list_operations(&self, exchange: ExchangeId) -> Vec<OperationKey> {
        self.rest_keys
            .get(&exchange)
            .map(|x| x.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_channels(&self, exchange: ExchangeId) -> Vec<ChannelKey> {
        self.ws_keys
            .get(&exchange)
            .map(|x| x.iter().cloned().collect())
            .unwrap_or_default()
    }
}

fn exchange_catalogs() -> Result<Vec<(ExchangeId, ExchangeCatalog)>, HubError> {
    const CATALOGS: &[(ExchangeId, &str)] = &[
        (
            ExchangeId::Binance,
            include_str!("../../../../../docs/exchanges/binance/catalog.json"),
        ),
        (
            ExchangeId::Bybit,
            include_str!("../../../../../docs/exchanges/bybit/catalog.json"),
        ),
        (
            ExchangeId::Coinbase,
            include_str!("../../../../../docs/exchanges/coinbase/catalog.json"),
        ),
        (
            ExchangeId::Coincheck,
            include_str!("../../../../../docs/exchanges/coincheck/catalog.json"),
        ),
        (
            ExchangeId::Deribit,
            include_str!("../../../../../docs/exchanges/deribit/catalog.json"),
        ),
        (
            ExchangeId::Gmocoin,
            include_str!("../../../../../docs/exchanges/gmocoin/catalog.json"),
        ),
        (
            ExchangeId::Kraken,
            include_str!("../../../../../docs/exchanges/kraken/catalog.json"),
        ),
        (
            ExchangeId::Okx,
            include_str!("../../../../../docs/exchanges/okx/catalog.json"),
        ),
        (
            ExchangeId::Upbit,
            include_str!("../../../../../docs/exchanges/upbit/catalog.json"),
        ),
    ];

    CATALOGS
        .iter()
        .map(|(exchange, raw)| {
            serde_json::from_str::<ExchangeCatalog>(raw)
                .map(|catalog| (*exchange, catalog))
                .map_err(HubError::Json)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_resolves_and_lists_keys() {
        let registry = SpecRegistry::global().unwrap();
        let ops = registry.list_operations(ExchangeId::Binance);
        assert!(!ops.is_empty());
        let key = &ops[0];
        assert_eq!(
            registry.resolve_rest(ExchangeId::Binance, key).unwrap().id,
            *key
        );
    }

    #[test]
    fn unknown_key_returns_error() {
        let registry = SpecRegistry::global().unwrap();
        let err = registry
            .resolve_ws(ExchangeId::Binance, "missing.key")
            .unwrap_err();
        assert!(matches!(err, HubError::UnknownChannel { .. }));
    }
}
