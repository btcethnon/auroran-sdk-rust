.PHONY: help e2e test fmt clippy check

help:
	@echo "Targets:"
	@echo "  make e2e     Run e2e_flow (requires AURORAN_PRIVATE_KEY)"
	@echo "  make test    Run all tests"
	@echo "  make fmt     Format code"
	@echo "  make clippy  Run clippy"
	@echo "  make check   fmt + clippy + test"

e2e:
	cargo run --example e2e_flow

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

check: fmt clippy test
