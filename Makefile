# ========================================
# 📦 Profinaut - Makefile（Vault統合・GitHub Actions対応・単一Compose構成）
# ========================================

# ========================================
# 📁 パス定義
# ========================================
ENV_FILE := docker/.env
DOCKER_COMPOSE := docker-compose.yml
VAULT_INIT_SCRIPT := docker/vault/scripts/init.sh
MERGE_ENV_SCRIPT := docker/vault/scripts/merge_env.sh

# ========================================
# 🔰 ヘルプ
# ========================================
.PHONY: help
help:
	@echo "🛠 Profinaut Makefile - Available Commands:"
	@grep -E '^[a-zA-Z_-]+:.*?##' Makefile | sed 's/:.*##/:/g' | column -t -s ':'

# ========================================
# 🔐 Vault関連
# ========================================
vault-init: ## Vaultポリシー/AppRoleを自動投入
	bash $(VAULT_INIT_SCRIPT)

vault-merge-env: ## .env.generated → .env へ安全にマージ
	bash $(MERGE_ENV_SCRIPT)

# ========================================
# 🐳 Docker関連
# ========================================
docker-up: ## Docker Compose 全サービス起動
	docker compose -f $(DOCKER_COMPOSE) up -d

docker-down: ## Docker Compose 停止
	docker compose -f $(DOCKER_COMPOSE) down

docker-pull: ## Dockerイメージをpull
	docker compose -f $(DOCKER_COMPOSE) pull

# ========================================
# 🧪 テスト（Rust/Python共通）
# ========================================
test: ## Python+Rustのテスト実行
	@echo "🧪 Running Python tests..."
	pip install -r requirements.txt
	pytest --disable-warnings
	@echo "🔧 Running Rust checks..."
	cargo check --all

# ========================================
# 🚀 デプロイ補助
# ========================================
release: docker-pull docker-up ## pullして再起動

# ========================================
# 🎉 初回セットアップ（Vault含む）
# ========================================
init: vault-init vault-merge-env docker-up ## Vault投入→envマージ→起動
# ========================================
# 🔐 Vault関連：.env.generated 自動生成
# ========================================
generate-env: ## Vaultから.env.generatedを生成（CI用）
	bash docker/vault/scripts/generate_env_ci.sh
