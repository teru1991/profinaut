use super::{ChannelKey, ExchangeId, OperationKey};
use crate::{EndpointSpec, ExchangeCatalog, WsChannelSpec};
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use super::errors::HubError;

#[derive(Debug, Clone, Copy)]
pub struct ExchangeRegistration {
    pub exchange_id: ExchangeId,
    pub canonical_name: &'static str,
    pub aliases: &'static [&'static str],
    pub catalog_json: &'static str,
    pub crate_family: &'static str,
    pub notes: &'static str,
}

const REGISTRATIONS: &[ExchangeRegistration] = &[
    ExchangeRegistration {
        exchange_id: ExchangeId::Binance,
        canonical_name: "binance",
        aliases: &["binance-spot"],
        catalog_json: include_str!("../../../../../docs/exchanges/binance/catalog.json"),
        crate_family: "ucel-cex-binance",
        notes: "spot",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::BinanceUsdm,
        canonical_name: "binance-usdm",
        aliases: &["binance-futures-usdm"],
        catalog_json: include_str!("../../../../../docs/exchanges/binance-usdm/catalog.json"),
        crate_family: "ucel-cex-binance-usdm",
        notes: "family split",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::BinanceCoinm,
        canonical_name: "binance-coinm",
        aliases: &["binance-futures-coinm"],
        catalog_json: include_str!("../../../../../docs/exchanges/binance-coinm/catalog.json"),
        crate_family: "ucel-cex-binance-coinm",
        notes: "family split",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::BinanceOptions,
        canonical_name: "binance-options",
        aliases: &["binance-option"],
        catalog_json: include_str!("../../../../../docs/exchanges/binance-options/catalog.json"),
        crate_family: "ucel-cex-binance-options",
        notes: "family split",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bitbank,
        canonical_name: "bitbank",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bitbank/catalog.json"),
        crate_family: "ucel-cex-bitbank",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bitflyer,
        canonical_name: "bitflyer",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bitflyer/catalog.json"),
        crate_family: "ucel-cex-bitflyer",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bitget,
        canonical_name: "bitget",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bitget/catalog.json"),
        crate_family: "ucel-cex-bitget",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bithumb,
        canonical_name: "bithumb",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bithumb/catalog.json"),
        crate_family: "ucel-cex-bithumb",
        notes: "crate exists but may be out of workspace members",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bitmex,
        canonical_name: "bitmex",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bitmex/catalog.json"),
        crate_family: "ucel-cex-bitmex",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bittrade,
        canonical_name: "bittrade",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bittrade/catalog.json"),
        crate_family: "ucel-cex-bittrade",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Bybit,
        canonical_name: "bybit",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/bybit/catalog.json"),
        crate_family: "ucel-cex-bybit",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Coinbase,
        canonical_name: "coinbase",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/coinbase/catalog.json"),
        crate_family: "ucel-cex-coinbase",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Coincheck,
        canonical_name: "coincheck",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/coincheck/catalog.json"),
        crate_family: "ucel-cex-coincheck",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Deribit,
        canonical_name: "deribit",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/deribit/catalog.json"),
        crate_family: "ucel-cex-deribit",
        notes: "catalog-empty: jsonrpc coverage staged",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Gmocoin,
        canonical_name: "gmocoin",
        aliases: &["gmo-coin"],
        catalog_json: include_str!("../../../../../docs/exchanges/gmocoin/catalog.json"),
        crate_family: "ucel-cex-gmocoin",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Htx,
        canonical_name: "htx",
        aliases: &["huobi"],
        catalog_json: include_str!("../../../../../docs/exchanges/htx/catalog.json"),
        crate_family: "ucel-cex-htx",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Kraken,
        canonical_name: "kraken",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/kraken/catalog.json"),
        crate_family: "ucel-cex-kraken",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Okx,
        canonical_name: "okx",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/okx/catalog.json"),
        crate_family: "ucel-cex-okx",
        notes: "",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Sbivc,
        canonical_name: "sbivc",
        aliases: &["sbi-vc"],
        catalog_json: include_str!("../../../../../docs/exchanges/sbivc/catalog.json"),
        crate_family: "ucel-cex-sbivc",
        notes: "public-only policy exception",
    },
    ExchangeRegistration {
        exchange_id: ExchangeId::Upbit,
        canonical_name: "upbit",
        aliases: &[],
        catalog_json: include_str!("../../../../../docs/exchanges/upbit/catalog.json"),
        crate_family: "ucel-cex-upbit",
        notes: "",
    },
];

pub fn exchange_registrations() -> &'static [ExchangeRegistration] {
    REGISTRATIONS
}

pub fn list_registered_exchanges() -> Vec<ExchangeId> {
    REGISTRATIONS.iter().map(|r| r.exchange_id).collect()
}

pub fn list_registered_exchange_ids() -> Vec<ExchangeId> {
    REGISTRATIONS.iter().map(|r| r.exchange_id).collect()
}

pub fn list_registered_exchange_names() -> Vec<&'static str> {
    REGISTRATIONS.iter().map(|r| r.canonical_name).collect()
}

pub fn list_registered_catalog_keys() -> Vec<&'static str> {
    REGISTRATIONS.iter().map(|r| r.canonical_name).collect()
}

pub fn find_registration(exchange: &str) -> Option<&'static ExchangeRegistration> {
    let input = exchange.to_ascii_lowercase();
    REGISTRATIONS.iter().find(|r| {
        r.canonical_name == input
            || r.aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&input))
    })
}

pub fn catalog_for(exchange: ExchangeId) -> Result<ExchangeCatalog, HubError> {
    let reg = REGISTRATIONS
        .iter()
        .find(|r| r.exchange_id == exchange)
        .ok_or_else(|| HubError::UnknownExchange(exchange.as_str().to_string()))?;
    serde_json::from_str::<ExchangeCatalog>(reg.catalog_json).map_err(HubError::Json)
}

fn is_public_catalog_entry(id: &str, visibility: &str) -> bool {
    let vis = visibility.trim().to_ascii_lowercase();
    if vis == "public" {
        return true;
    }
    if vis == "private" {
        return false;
    }
    (id.contains(".public.") || id.starts_with("public.")) && !id.contains(".private.")
}

pub fn list_public_rest_entries(exchange: ExchangeId) -> Result<Vec<EndpointSpec>, HubError> {
    let catalog = catalog_for(exchange)?;
    Ok(catalog
        .rest_endpoints
        .into_iter()
        .filter(|entry| is_public_catalog_entry(&entry.id, &entry.visibility))
        .collect())
}

pub fn list_public_ws_entries(exchange: ExchangeId) -> Result<Vec<WsChannelSpec>, HubError> {
    let catalog = catalog_for(exchange)?;
    Ok(catalog
        .ws_channels
        .into_iter()
        .filter(|entry| is_public_catalog_entry(&entry.id, &entry.visibility))
        .collect())
}

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

        for registration in REGISTRATIONS {
            let exchange = registration.exchange_id;
            let catalog = catalog_for(exchange)?;
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
            rest_keys.entry(exchange).or_default();
            ws_keys.entry(exchange).or_default();
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

    pub fn catalog_entries_counts(&self, exchange: ExchangeId) -> Result<(usize, usize), HubError> {
        let rest = self
            .rest_keys
            .get(&exchange)
            .map(|x| x.len())
            .ok_or_else(|| HubError::UnknownExchange(exchange.as_str().to_string()))?;
        let ws = self
            .ws_keys
            .get(&exchange)
            .map(|x| x.len())
            .ok_or_else(|| HubError::UnknownExchange(exchange.as_str().to_string()))?;
        Ok((rest, ws))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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

    #[test]
    fn exchange_roundtrip_and_alias() {
        for id in ExchangeId::all() {
            let canonical = id.as_str();
            let parsed = ExchangeId::from_str(canonical).unwrap();
            assert_eq!(*id, parsed);
        }
        assert_eq!(
            ExchangeId::from_str("binance-spot").unwrap(),
            ExchangeId::Binance
        );
    }

    #[test]
    fn registrations_align_with_exchange_id_all() {
        assert_eq!(exchange_registrations().len(), ExchangeId::all().len());
    }
}
