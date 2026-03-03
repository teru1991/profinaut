use serde_json::Value;

#[derive(thiserror::Error, Debug)]
pub enum CoverageGateError {
    #[error("failed to read coverage file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing required field: {0}")]
    Missing(&'static str),
    #[error("invalid coverage for {exchange}: {reason}")]
    Invalid { exchange: String, reason: String },
}

fn bool_at(v: &Value, path: &[&str]) -> Option<bool> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_bool()
}

pub fn load_json(path: &std::path::Path) -> Result<Value, CoverageGateError> {
    let bytes = std::fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn assert_domestic_requirements(exchange: &str, v: &Value) -> Result<(), CoverageGateError> {
    let pr = bool_at(v, &["public", "rest"]).ok_or(CoverageGateError::Missing("public.rest"))?;
    let pw = bool_at(v, &["public", "ws"]).ok_or(CoverageGateError::Missing("public.ws"))?;
    let private_enabled =
        bool_at(v, &["private", "enabled"]).ok_or(CoverageGateError::Missing("private.enabled"))?;

    if !pr {
        return Err(CoverageGateError::Invalid {
            exchange: exchange.to_string(),
            reason: "domestic must support public REST".into(),
        });
    }
    if !pw {
        return Err(CoverageGateError::Invalid {
            exchange: exchange.to_string(),
            reason: "domestic must support public WS".into(),
        });
    }
    if !private_enabled {
        return Err(CoverageGateError::Invalid {
            exchange: exchange.to_string(),
            reason: "domestic must support private (enabled=true)".into(),
        });
    }
    Ok(())
}

pub fn assert_overseas_requirements(exchange: &str, v: &Value) -> Result<(), CoverageGateError> {
    let pr = bool_at(v, &["public", "rest"]).ok_or(CoverageGateError::Missing("public.rest"))?;
    let pw = bool_at(v, &["public", "ws"]).ok_or(CoverageGateError::Missing("public.ws"))?;
    if !pr {
        return Err(CoverageGateError::Invalid {
            exchange: exchange.to_string(),
            reason: "overseas must support public REST".into(),
        });
    }
    if !pw {
        return Err(CoverageGateError::Invalid {
            exchange: exchange.to_string(),
            reason: "overseas must support public WS".into(),
        });
    }
    Ok(())
}

pub fn ws_ops(v: &Value) -> Vec<String> {
    v.get("ws_ops")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub fn public_ws_enabled(v: &Value) -> Result<bool, CoverageGateError> {
    bool_at(v, &["public", "ws"]).ok_or(CoverageGateError::Missing("public.ws"))
}
