# Profinaut

**Investment platform monorepo** with FastAPI dashboard-api, worker, and PostgreSQL backend.

⚠️ **DEMO MODE ONLY - NO REAL TRADING** ⚠️

## Features

- **FastAPI Dashboard API**: RESTful API with /health and /metrics endpoints
- **Worker Service**: Background tasks for market data updates and bot monitoring
- **Bot Registry**: Pluggable bot lifecycle management with DummyBot
- **Market Data Provider**: Dummy data generation with caching
- **Contracts (SSOT)**: Shared Pydantic schemas across services
- **PostgreSQL + Alembic**: Database with migration support
- **JSON Logging**: Structured logging for all services
- **Docker Compose**: Complete local development environment
- **Kill Switch**: Safety mechanism (no real trading enabled)

## Project Structure

```
profinaut/
├── contracts/          # Single Source of Truth (SSOT) schemas
│   └── schemas.py      # Shared Pydantic models
├── dashboard_api/      # FastAPI dashboard API
│   ├── main.py         # API endpoints
│   ├── bot_registry.py # Bot lifecycle management
│   ├── market_data.py  # Market data provider
│   ├── models.py       # SQLAlchemy models
│   └── logging_config.py
├── worker/             # Background worker service
│   └── main.py         # Worker tasks
├── alembic/            # Database migrations
├── tests/              # Test suite
├── docker-compose.yml  # Docker services
└── pyproject.toml      # Project configuration
```

## Quick Start

### Local Development (with Docker)

1. **Start all services**:
   ```bash
   docker-compose up -d
   ```

2. **Access the API**:
   - Dashboard API: http://localhost:8000
   - API Docs: http://localhost:8000/docs
   - Health: http://localhost:8000/health
   - Metrics: http://localhost:8000/metrics

3. **Run migrations**:
   ```bash
   docker-compose exec dashboard-api alembic upgrade head
   ```

### Local Development (without Docker)

1. **Install dependencies**:
   ```bash
   pip install -e .[dev]
   ```

2. **Start PostgreSQL** (required):
   ```bash
   docker run -d \
     --name profinaut-postgres \
     -e POSTGRES_USER=profinaut \
     -e POSTGRES_PASSWORD=profinaut \
     -e POSTGRES_DB=profinaut \
     -p 5432:5432 \
     postgres:16-alpine
   ```

3. **Run migrations**:
   ```bash
   alembic upgrade head
   ```

4. **Start the API**:
   ```bash
   uvicorn dashboard_api.main:app --reload
   ```

5. **Start the worker** (in another terminal):
   ```bash
   python -m worker.main
   ```

## API Endpoints

### Health & Monitoring
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /kill-switch` - Trading kill switch status

### Bot Management
- `GET /bots` - List all bots
- `POST /bots` - Create a new bot
- `POST /bots/{bot_id}/start` - Start a bot
- `POST /bots/{bot_id}/stop` - Stop a bot

### Market Data
- `GET /market-data/symbols` - List available symbols
- `GET /market-data/{symbol}/price` - Get current price
- `GET /market-data/{symbol}/volume` - Get current volume
- `GET /market-data/{symbol}/orderbook` - Get orderbook

## Example Usage

### Create and start a bot:
```bash
# Create a bot
curl -X POST http://localhost:8000/bots \
  -H "Content-Type: application/json" \
  -d '{
    "bot_id": "my-bot",
    "bot_type": "dummy",
    "enabled": true,
    "config": {}
  }'

# Start the bot
curl -X POST http://localhost:8000/bots/my-bot/start

# Check bot status
curl http://localhost:8000/bots
```

### Get market data:
```bash
# Get BTC/USD price
curl http://localhost:8000/market-data/BTC%2FUSD/price

# List available symbols
curl http://localhost:8000/market-data/symbols
```

## Testing

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=. --cov-report=term-missing

# Run specific test file
pytest tests/test_api.py -v
```

## Linting

```bash
# Check code style
ruff check .

# Format code
ruff format .
```

## Database Migrations

```bash
# Create a new migration
alembic revision --autogenerate -m "description"

# Apply migrations
alembic upgrade head

# Rollback one migration
alembic downgrade -1
```

## Configuration

Environment variables:
- `DATABASE_URL` - PostgreSQL connection string (default: `postgresql://profinaut:profinaut@localhost:5432/profinaut`)
- `REDIS_URL` - Redis connection string (default: `redis://localhost:6379`)

## Architecture

### Contracts (SSOT)
All shared data models are defined in `contracts/schemas.py` using Pydantic. This ensures type safety and consistency across services.

### Bot Registry
The bot registry manages bot lifecycle:
- **Bot Types**: Pluggable bot implementations (DummyBot included)
- **Lifecycle**: Start, stop, and monitor bots
- **State Management**: Track bot status and errors

### Market Data Provider
Provides dummy market data with caching:
- **Price Data**: Simulated prices with random variation
- **Volume Data**: Simulated trading volumes
- **Orderbook**: Simulated bid/ask levels

### Worker Service
Background tasks:
- **Market Data Updates**: Periodic price updates for all symbols
- **Bot Monitoring**: Monitor and log bot status

### Kill Switch
The kill switch is **always enabled** to prevent real trading:
- All trading operations are blocked
- Bots run in demo mode only
- Safety mechanism for development/testing

## Security

- ⚠️ **No secrets in code**: All credentials via environment variables
- ⚠️ **No real trading**: Kill switch enforced
- ⚠️ **Demo mode only**: This is a scaffold for development

## CI/CD

GitHub Actions CI pipeline:
- **Linting**: Ruff code style checks
- **Testing**: Pytest with coverage
- **Docker**: Build verification

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## Support

For issues and questions, please open a GitHub issue.
