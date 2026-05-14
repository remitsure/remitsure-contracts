.PHONY: build test fmt lint check deploy-testnet clean

build:
	stellar contract build
	@ls -l target/wasm32v1-none/release/*.wasm

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings

check: lint test build

deploy-testnet: build
	./scripts/deploy-testnet.sh

clean:
	cargo clean
