# KeePassEx — Development Makefile
# Usage: make <target>

.PHONY: help install build test lint fmt clean desktop mobile cli extension check

# ─── Colors ───────────────────────────────────────────────────────────────────
BOLD  := \033[1m
GREEN := \033[32m
CYAN  := \033[36m
RESET := \033[0m

help: ## Show this help
	@echo "$(BOLD)KeePassEx Development Commands$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'

# ─── Setup ────────────────────────────────────────────────────────────────────

install: ## Install all dependencies (JS + Rust)
	@echo "$(GREEN)Installing JS dependencies...$(RESET)"
	pnpm install
	@echo "$(GREEN)Building Rust core...$(RESET)"
	cargo build -p keepassex-core
	@echo "$(GREEN)✓ Setup complete$(RESET)"

# ─── Build ────────────────────────────────────────────────────────────────────

build: ## Build all packages
	cargo build --release --workspace
	pnpm build

build-core: ## Build Rust core only
	cargo build -p keepassex-core

build-desktop: ## Build desktop app (production)
	pnpm --filter @keepassex/desktop tauri:build

build-cli: ## Build CLI for current platform
	cargo build --release -p keepassex-cli

build-cli-all: ## Build CLI for all platforms
	cargo build --release -p keepassex-cli --target x86_64-unknown-linux-gnu
	cargo build --release -p keepassex-cli --target x86_64-apple-darwin
	cargo build --release -p keepassex-cli --target aarch64-apple-darwin

build-extension: ## Build browser extensions
	pnpm --filter @keepassex/browser-extension build:all

build-packages: ## Build all JS packages
	pnpm build --filter='./packages/*' --filter='./shared/*'

# ─── Development ──────────────────────────────────────────────────────────────

desktop: ## Run desktop app in dev mode
	pnpm --filter @keepassex/desktop tauri:dev

mobile: ## Start React Native metro bundler
	pnpm --filter @keepassex/mobile start

mobile-ios: ## Run mobile app on iOS simulator
	pnpm --filter @keepassex/mobile ios

mobile-android: ## Run mobile app on Android emulator
	pnpm --filter @keepassex/mobile android

cli: ## Run CLI (pass args with ARGS="--help")
	cargo run -p keepassex-cli -- $(ARGS)

tui: ## Run TUI (pass args with ARGS="--vault path")
	cargo run -p keepassex-tui -- $(ARGS)

build-tui: ## Build TUI for current platform
	cargo build --release -p keepassex-tui

build-server: ## Build self-hosted sync server
	cargo build --release -p keepassex-server

build-credprov: ## Build Windows Credential Provider DLL (Windows only)
	cargo build --release -p keepassex-credprov --target x86_64-pc-windows-msvc

server: ## Run self-hosted sync server (dev)
	cargo run -p keepassex-server -- --port 8080

server-docker: ## Run server with Docker Compose
	docker compose -f apps/server/docker-compose.yml up -d

extension-dev: ## Watch browser extension (Chrome)
	pnpm --filter @keepassex/browser-extension dev:chrome

# ─── Testing ──────────────────────────────────────────────────────────────────

test: ## Run all tests
	$(MAKE) test-rust
	$(MAKE) test-ts

test-rust: ## Run Rust tests
	cargo test --all --all-features

test-ts: ## Run TypeScript tests
	pnpm test:ts

test-coverage: ## Run tests with coverage
	pnpm test:ts -- --coverage
	cargo tarpaulin --all --out Html

test-watch: ## Run TypeScript tests in watch mode
	pnpm --filter '*' vitest

# ─── Code Quality ─────────────────────────────────────────────────────────────

lint: ## Run all linters
	cargo clippy --all-targets --all-features -- -D warnings
	pnpm lint

fmt: ## Format all code
	cargo fmt --all
	pnpm prettier --write "**/*.{ts,tsx,json,md}" --ignore-path .gitignore

fmt-check: ## Check formatting without modifying
	cargo fmt --all -- --check
	pnpm prettier --check "**/*.{ts,tsx,json,md}" --ignore-path .gitignore

typecheck: ## Run TypeScript type checking
	pnpm typecheck

check: ## Run all checks (lint + typecheck + fmt-check)
	$(MAKE) fmt-check
	$(MAKE) lint
	$(MAKE) typecheck

# ─── Security ─────────────────────────────────────────────────────────────────

audit: ## Run security audits
	cargo audit
	pnpm audit --audit-level=high

# ─── Cleanup ──────────────────────────────────────────────────────────────────

clean: ## Clean all build artifacts
	cargo clean
	pnpm clean
	rm -rf apps/browser-extension/dist
	rm -rf apps/desktop/dist

clean-deps: ## Remove all node_modules
	find . -name "node_modules" -type d -prune -exec rm -rf '{}' +

# ─── Utilities ────────────────────────────────────────────────────────────────

docs: ## Open documentation
	@echo "$(CYAN)Documentation:$(RESET)"
	@echo "  Architecture: docs/ARCHITECTURE.md"
	@echo "  API:          docs/API.md"
	@echo "  Build:        docs/BUILD.md"
	@echo "  Sync:         docs/SYNC.md"
	@echo "  Import/Export: docs/IMPORT_EXPORT.md"

version: ## Show version info
	@echo "$(BOLD)KeePassEx$(RESET)"
	@echo "  Rust:  $$(rustc --version)"
	@echo "  Node:  $$(node --version)"
	@echo "  pnpm:  $$(pnpm --version)"
	@cargo metadata --no-deps --format-version 1 | python3 -c "import json,sys; m=json.load(sys.stdin); print('  Core: v' + [p for p in m['packages'] if p['name']=='keepassex-core'][0]['version'])"

new-entry: ## Quick add entry via CLI (VAULT=path PASSWORD=pass TITLE=name)
	cargo run -p keepassex-cli -- \
		--vault "$(VAULT)" \
		--password "$(PASSWORD)" \
		add --title "$(TITLE)" --generate

steg-embed: ## Embed vault into image (CARRIER=img.png VAULT=v.kdbx OUTPUT=out.png STEG_PWD=pass)
	cargo run -p keepassex-cli -- \
		--vault "$(VAULT)" \
		--password "$(PASSWORD)" \
		steg embed --carrier "$(CARRIER)" --vault "$(VAULT)" --output "$(OUTPUT)" --steg-password "$(STEG_PWD)"

shard-split: ## Split vault key (VAULT=path PASSWORD=pass THRESHOLD=3 TOTAL=5)
	cargo run -p keepassex-cli -- \
		--vault "$(VAULT)" \
		--password "$(PASSWORD)" \
		shard split --threshold $(THRESHOLD) --total $(TOTAL) --output-dir ./shards/

nl-search: ## Natural language search (VAULT=path PASSWORD=pass QUERY="find weak passwords")
	cargo run -p keepassex-cli -- \
		--vault "$(VAULT)" \
		--password "$(PASSWORD)" \
		search "$(QUERY)"
