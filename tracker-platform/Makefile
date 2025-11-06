# Makefile for Tracker Platform Testing

.PHONY: help test test-unit test-integration test-load test-all coverage clean

help: ## Show this help message
	@echo "Tracker Platform - Test Commands"
	@echo "=================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Rust Backend Tests
test: ## Run all Rust tests
	cargo test --all-features

test-unit: ## Run unit tests only
	cargo test --lib --all-features

test-integration: ## Run integration tests
	cargo test --test '*' --all-features

test-crate: ## Run tests for specific crate (use CRATE=name)
	cargo test -p $(CRATE)

test-watch: ## Run tests in watch mode
	cargo watch -x test

# Load Tests
test-load: ## Run all load tests
	k6 run tests/load/announce.js
	k6 run tests/load/api.js
	k6 run tests/load/search.js
	k6 run tests/load/graphql.js

test-load-announce: ## Run tracker announce load test
	k6 run --vus 1000 --duration 5m tests/load/announce.js

test-load-api: ## Run API load test
	k6 run --vus 100 --duration 2m tests/load/api.js

test-load-search: ## Run search load test
	k6 run --vus 75 --duration 2m tests/load/search.js

test-load-graphql: ## Run GraphQL load test
	k6 run --vus 75 --duration 2m tests/load/graphql.js

# Coverage
coverage: ## Generate coverage report (using llvm-cov)
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	cargo llvm-cov report --html

coverage-open: coverage ## Generate and open HTML coverage report
	open target/llvm-cov/html/index.html || xdg-open target/llvm-cov/html/index.html

# Alternative: using tarpaulin
coverage-tarpaulin: ## Generate coverage with tarpaulin
	cargo tarpaulin --all-features --workspace --out Html --out Lcov

# Frontend Tests
test-frontend: ## Run all frontend tests
	cd frontend && npm test

test-frontend-ui: ## Run frontend tests with UI
	cd frontend && npm run test:ui

test-frontend-coverage: ## Run frontend tests with coverage
	cd frontend && npm run test:coverage

test-e2e: ## Run E2E tests
	cd frontend && npm run test:e2e

test-e2e-ui: ## Run E2E tests with UI
	cd frontend && npm run test:e2e:ui

test-e2e-headed: ## Run E2E tests in headed mode
	cd frontend && npx playwright test --headed

# All Tests
test-all: test test-frontend test-e2e ## Run all tests (backend, frontend, E2E)

# Database
db-test-setup: ## Set up test database
	createdb tracker_test || true
	DATABASE_URL=postgresql://tracker:tracker_password@localhost:5432/tracker_test sqlx migrate run

db-test-reset: ## Reset test database
	dropdb tracker_test || true
	$(MAKE) db-test-setup

# Docker Tests
test-docker: ## Run tests in Docker
	docker-compose -f docker-compose.test.yml up --abort-on-container-exit

# Benchmarks
bench: ## Run benchmarks
	cargo bench --all-features

# Linting & Formatting
lint: ## Run linters
	cargo clippy --all-features -- -D warnings
	cd frontend && npm run lint

format: ## Format code
	cargo fmt --all
	cd frontend && npm run format

format-check: ## Check code formatting
	cargo fmt --all -- --check
	cd frontend && npm run format -- --check

# Clean
clean: ## Clean build artifacts and test data
	cargo clean
	rm -rf target/
	rm -rf frontend/node_modules
	rm -rf frontend/.svelte-kit
	rm -f lcov.info
	rm -rf htmlcov/
	rm -rf playwright-report/
	rm -rf test-results/

# CI/CD
ci: lint format-check test coverage ## Run CI pipeline locally

# Install Tools
install-tools: ## Install testing tools
	cargo install cargo-watch
	cargo install cargo-llvm-cov
	cargo install cargo-tarpaulin
	cargo install sqlx-cli
	npm install -g k6
	cd frontend && npm install
