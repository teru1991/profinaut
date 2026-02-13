use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct HealthzResponse {
    status: &'static str,
    timestamp_utc: String,
}

#[derive(Serialize)]
struct CapabilitiesResponse {
    service: &'static str,
    version: &'static str,
    features: Vec<&'static str>,
    status: &'static str,
    degraded_reason: Option<String>,
    generated_at: String,
}

#[derive(Deserialize)]
struct TickerQuery {
    exchange: String,
    symbol: String,
}

#[derive(Serialize)]
struct TickerResponse {
    exchange: String,
    symbol: String,
    price: f64,
    bid: f64,
    ask: f64,
    status: &'static str,
    degraded_reason: Option<String>,
    timestamp_utc: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/healthz", get(get_healthz))
        .route("/capabilities", get(get_capabilities))
        .route("/ticker/latest", get(get_ticker_latest));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("failed to bind 0.0.0.0:8081");

    println!("marketdata-rs listening on http://0.0.0.0:8081");
    axum::serve(listener, app)
        .await
        .expect("server error");
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
        features: vec!["ticker"],
        status: "ok",
        degraded_reason: None,
        generated_at: Utc::now().to_rfc3339(),
    })
}

async fn get_ticker_latest(Query(query): Query<TickerQuery>) -> impl IntoResponse {
    if query.exchange.as_str() != "gmo" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_exchange",
                message: format!(
                    "unsupported exchange '{}'; supported exchanges: [gmo]",
                    query.exchange
                ),
            }),
        )
            .into_response();
    }

    if query.symbol.as_str() != "BTC_JPY" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_symbol",
                message: format!(
                    "unsupported symbol '{}'; supported symbols for gmo: [BTC_JPY]",
                    query.symbol
                ),
            }),
        )
            .into_response();
    }

    let payload = TickerResponse {
        exchange: query.exchange,
        symbol: query.symbol,
        price: 10_000_000.0,
        bid: 9_999_900.0,
        ask: 10_000_100.0,
        status: "ok",
        degraded_reason: None,
        timestamp_utc: Utc::now().to_rfc3339(),
    };

    (StatusCode::OK, Json(payload)).into_response()
}
