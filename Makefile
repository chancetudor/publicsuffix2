# Mirrors commands from .github/workflows/ci.yml

# Use bash for all recipes
SHELL := /bin/bash
CARGO := cargo

# Phony targets don't represent files
.PHONY: all ci test lint fmt clippy clean

# Default target: run all CI checks
all: ci

# Run all checks performed in CI
ci: test lint
	@echo "✅ All CI checks passed!"

# Run all tests with all features enabled.
test:
	@echo "--- Running tests ---"
	$(CARGO) test --all-features

# Run all lints (formatting and clippy).
lint: fmt clippy

# Check code formatting.
fmt:
	@echo "--- Checking formatting ---"
	$(CARGO) fmt --all -- --check

# Run clippy with strict warnings.
clippy:
	@echo "--- Running clippy ---"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Clean up build artifacts.
clean:
	@echo "--- Cleaning project ---"
	$(CARGO) clean