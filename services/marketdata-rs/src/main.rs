use std::{collections::HashMap, net::SocketAddr};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct HealthzResponse {
    status: &'static str,
    timestamp_utc: String,
}

#[derive(Serialize)]
struct CapabilitiesResponse {
    service: &'static str,
    version: &'static str,
    status: &'static str,
    generated_at: String,
    features: Vec<&'static str>,
}

#[derive(Serialize)]
struct TickerLatestResponse {
    ts_utc: String,
    exchange: &'static str,
    symbol: &'static str,
    bid: f64,
    ask: f64,
    last: f64,
    degraded: bool,
    degraded_reason: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("server error: {err}");
    }
}

async fn run() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/healthz", get(get_healthz))
        .route("/capabilities", get(get_capabilities))
        .route("/ticker/latest", get(get_ticker_latest));

    let addr: SocketAddr = "0.0.0.0:8081".parse().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid bind address")
    })?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("marketdata-rs listening on http://{addr}");
    axum::serve(listener, app).await
}

async fn get_healthz() -> Json<HealthzResponse> {
    Json(HealthzResponse {
        status: "ok",
        timestamp_utc: Utc::now().to_rfc3339(),
    })
}

async fn get_capabilities() -> Json<CapabilitiesResponse> {
    Json(CapabilitiesResponse {
        service: "marketdata",
        version: "0.1.0",
        status: "ok",
        generated_at: Utc::now().to_rfc3339(),
        features: vec!["ticker"],
    })
}

async fn get_ticker_latest(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let exchange = match params.get("exchange") {
        Some(v) if !v.is_empty() => v,
        _ => {
            return bad_request(
                "missing_exchange",
                "query param 'exchange' is required".to_string(),
            )
        }
    };

    let symbol = match params.get("symbol") {
        Some(v) if !v.is_empty() => v,
        _ => {
            return bad_request(
                "missing_symbol",
                "query param 'symbol' is required".to_string(),
            )
        }
    };

    if exchange != "gmo" {
        return bad_request(
            "invalid_exchange",
            format!("unsupported exchange '{exchange}'; supported exchanges: [gmo]"),
        );
    }

    if symbol != "BTC_JPY" {
        return bad_request(
            "invalid_symbol",
            format!("unsupported symbol '{symbol}'; supported symbols for gmo: [BTC_JPY]"),
        );
    }

    let payload = TickerLatestResponse {
        ts_utc: Utc::now().to_rfc3339(),
        exchange: "gmo",
        symbol: "BTC_JPY",
        bid: 9_999_900.0,
        ask: 10_000_100.0,
        last: 10_000_000.0,
        degraded: false,
        degraded_reason: None,
    };

    (StatusCode::OK, Json(payload)).into_response()
}

fn bad_request(error: &'static str, message: String) -> axum::response::Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse { error, message }),
    )
        .into_response()
}
