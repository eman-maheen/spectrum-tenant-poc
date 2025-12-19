.PHONY: help dev-backend dev-desktop clean test

help:
	@echo "Available commands:"
	@echo "  make dev-backend   - Run backend server"
	@echo "  make dev-desktop   - Run desktop app"
	@echo "  make clean         - Clean all build artifacts"
	@echo "  make test          - Run all tests"

dev-backend:
	cd backend && cargo run

dev-desktop:
	cd desktop && cargo tauri dev

clean:
	cargo clean
	cd frontend && rm -rf node_modules build .svelte-kit

test:
	cargo test --all
