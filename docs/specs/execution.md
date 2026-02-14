# Execution Service Specification

## Overview

The execution service is a minimal paper execution service for order placement and tracking. It provides a safe-by-default implementation with symbol/exchange allowlists and comprehensive logging.

**Current version:** 0.1.0  
**Service name:** `execution`

## Features

- **Paper execution mode**: Simulates order execution without real trading
- **Idempotency**: Duplicate order prevention via idempotency keys
- **Safe defaults**: Unknown symbols and exchanges rejected by default
- **Observability**: Comprehensive logging with order lifecycle tracking

## API Endpoints

### GET /healthz

Health check endpoint.

**Response (200 OK):**
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### GET /capabilities

Service capabilities and features.

**Response (200 OK):**
```json
{
  "service": "execution",
  "version": "0.1.0",
  "status": "ok",
  "features": ["paper_execution"],
  "degraded_reason": null,
  "generated_at": "2024-01-15T10:30:00Z"
}
```

### POST /execution/order-intents

Submit an order intent for paper execution.

**Request Body:**
```json
{
  "idempotency_key": "unique-key-123",
  "exchange": "binance",
  "symbol": "BTC/USDT",
  "side": "BUY",
  "qty": 0.01,
  "type": "MARKET",
  "limit_price": null,
  "client_ts_utc": null
}
```

**Fields:**
- `idempotency_key` (string, required): Unique key for idempotent order submission
- `exchange` (string, required): Exchange identifier (must be in allowlist)
- `symbol` (string, required): Trading symbol (must be in allowlist)
- `side` (string, required): Order side, either "BUY" or "SELL"
- `qty` (number, required): Order quantity (must be > 0)
- `type` (string, required): Order type, either "MARKET" or "LIMIT"
- `limit_price` (number, optional): Limit price (required for LIMIT orders, must be > 0)
- `client_ts_utc` (string, optional): ISO 8601 timestamp when intent was created

**Response (201 Created):**
```json
{
  "order_id": "paper-a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "status": "NEW",
  "accepted_ts_utc": "2024-01-15T10:30:00Z",
  "exchange": "binance",
  "symbol": "BTC/USDT",
  "side": "BUY",
  "qty": 0.01,
  "filled_qty": 0.0
}
```

**Response (400 Bad Request):**
Symbol or exchange not in allowlist, or invalid order parameters.

**Response (409 Conflict):**
Duplicate `idempotency_key` - order was already submitted.

## Configuration

Configuration is managed via environment variables.

### Required Configuration

```bash
# Comma-separated list of allowed trading symbols
ALLOWED_SYMBOLS="BTC/USDT,ETH/USDT,SOL/USDT"

# Comma-separated list of allowed exchanges
ALLOWED_EXCHANGES="binance,coinbase"
```

### Optional Configuration

```bash
# Service identification
SERVICE_NAME="execution"
SERVICE_VERSION="0.1.0"
```

### Safe Defaults

- **Empty allowlists reject all**: If `ALLOWED_SYMBOLS` or `ALLOWED_EXCHANGES` is empty or not set, all orders are rejected
- **No secrets required**: Paper execution does not use real exchange credentials
- **No database**: Uses in-memory storage for simplicity

## Logging

All order operations are logged with structured fields:

- `idempotency_key`: Unique order intent identifier
- `order_id`: Generated order identifier (after creation)
- `exchange`: Exchange identifier
- `symbol`: Trading symbol
- `side`: Order side (BUY/SELL)
- `qty`: Order quantity
- `type`: Order type (MARKET/LIMIT)
- `status`: Order status

**Example log entries:**
```
2024-01-15 10:30:00 - execution - INFO - Received order intent {"idempotency_key": "test-1", "exchange": "binance", "symbol": "BTC/USDT", "side": "BUY", "qty": 0.01, "type": "MARKET"}
2024-01-15 10:30:00 - execution - INFO - Order created successfully {"idempotency_key": "test-1", "order_id": "paper-a1b2c3d4", "exchange": "binance", "symbol": "BTC/USDT", "side": "BUY", "qty": 0.01, "status": "NEW"}
```

## Running the Service

### Install Dependencies

```bash
cd services/execution
pip install -r requirements.txt
```

### Start the Service

```bash
# Set configuration
export ALLOWED_SYMBOLS="BTC/USDT,ETH/USDT"
export ALLOWED_EXCHANGES="binance,coinbase"

# Run with uvicorn
uvicorn app.main:app --host 0.0.0.0 --port 8001
```

### Run Tests

```bash
pip install -r requirements-dev.txt
PYTHONPATH=. pytest tests/
```

## Architecture

### Components

1. **main.py**: FastAPI application with endpoint handlers
2. **schemas.py**: Pydantic models for request/response validation
3. **storage.py**: Thread-safe in-memory order storage
4. **config.py**: Configuration management with Pydantic Settings

### Storage

Orders are stored in-memory with thread-safe operations:
- Orders indexed by `order_id`
- Idempotency map: `idempotency_key` → `order_id`
- Thread-safe operations using Python locks

### Order Lifecycle

1. Client submits OrderIntent with `idempotency_key`
2. Service validates symbol/exchange against allowlist
3. Service checks for duplicate `idempotency_key` (returns 409 if exists)
4. Service generates `order_id` and creates Order with status=NEW
5. Service returns Order to client (201)

## Future Enhancements

Optional endpoints that can be added:

- `GET /execution/orders` - List all orders
- `GET /execution/orders/{order_id}` - Get specific order
- `GET /execution/orders/{order_id}/fills` - Get order fills
- Order status updates (FILLED, CANCELED, etc.)
- Fill simulation based on market data
- Persistent storage (database)
- Live execution mode with real exchange integration

## Security

- **No secrets**: Paper mode does not use real credentials
- **Allowlist enforcement**: Unknown symbols/exchanges rejected
- **Input validation**: Pydantic models validate all inputs
- **Idempotency**: Prevents duplicate order submission
- **No database**: Eliminates SQL injection risks
- **Safe defaults**: Rejects everything unless explicitly allowed

## Observability

- Structured logging with JSON-compatible fields
- All order lifecycle events logged
- Request/response logging
- Error conditions logged with context
