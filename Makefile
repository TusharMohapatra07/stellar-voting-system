.PHONY: help install build test clean

help:
	@echo "Stellar Voting DApp - Development Commands"
	@echo ""
	@echo "Available targets:"
	@echo "  install-contracts  - Install Rust dependencies for contracts"
	@echo "  install-frontend   - Install Node.js dependencies for frontend"
	@echo "  install-backend    - Install Node.js dependencies for backend"
	@echo "  install-all        - Install all dependencies"
	@echo ""
	@echo "  build-contracts    - Build Soroban smart contracts"
	@echo "  build-frontend     - Build frontend application"
	@echo "  build-backend      - Build backend API"
	@echo "  build-all          - Build all components"
	@echo ""
	@echo "  test-contracts     - Run smart contract tests"
	@echo "  test-frontend      - Run frontend tests"
	@echo "  test-backend       - Run backend tests"
	@echo "  test-all           - Run all tests"
	@echo ""
	@echo "  dev-frontend       - Start frontend development server"
	@echo "  dev-backend        - Start backend development server"
	@echo ""
	@echo "  clean              - Remove build artifacts"

install-contracts:
	cd contracts && cargo fetch

install-frontend:
	cd frontend && npm install

install-backend:
	cd backend && npm install

install-all: install-contracts install-frontend install-backend

build-contracts:
	cd contracts && cargo build --target wasm32-unknown-unknown --release

build-frontend:
	cd frontend && npm run build

build-backend:
	cd backend && npm run build

build-all: build-contracts build-frontend build-backend

test-contracts:
	cd contracts && cargo test

test-frontend:
	cd frontend && npm test

test-backend:
	cd backend && npm test

test-all: test-contracts test-frontend test-backend

dev-frontend:
	cd frontend && npm run dev

dev-backend:
	cd backend && npm run dev

clean:
	cd contracts && cargo clean
	cd frontend && rm -rf dist node_modules
	cd backend && rm -rf dist node_modules
