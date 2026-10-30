.PHONY: all build build-release install install-release clean test test-terminal check fmt lint run demo release measure measure-metrics measure-footprint

LEVEL ?= minor
TARGET_DIR = $(shell cargo metadata --no-deps --format-version 1 | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')
BINARY ?= $(TARGET_DIR)/release/toppy

# Default target
all: check build test

# Build debug version
build:
	cargo build

# Build release version
build-release:
	cargo build --release

# Install debug binary to ~/.cargo/bin
install:
	CARGO_INCREMENTAL=0 cargo install --path . --locked --bins --debug --force

# Install release binary to ~/.cargo/bin
install-release:
	CARGO_INCREMENTAL=0 cargo install --path . --locked --bins --force

# Clean build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# End-to-end refresh and terminal restoration tests (Linux/macOS, Python 3)
test-terminal: build-release
	python3 scripts/test_terminal.py "$(BINARY)" -v

# Repeatable release measurements; run before and after changing performance code
measure: measure-metrics measure-footprint

measure-metrics:
	cargo run --release --locked --example measure_metrics
	cargo test --release measure_expanded_snapshot_refresh -- --ignored --nocapture

measure-footprint: build-release
	python3 scripts/measure.py "$(BINARY)"

# Run clippy and check
check:
	cargo check
	cargo clippy --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt

# Lint (check formatting)
lint:
	cargo fmt -- --check
	cargo clippy --all-targets -- -D warnings

# Run with arguments (usage: make run ARGS="--refresh-rate 500")
run:
	cargo run -- $(ARGS)

# Quick demo
demo: install
	@echo "=== toppy demo ==="
	toppy --help

# Bump version, finalize CHANGELOG.md, tag, publish, and push (requires cargo-release)
release:
	cargo release $(LEVEL) --execute --no-confirm
