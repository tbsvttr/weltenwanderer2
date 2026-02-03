.PHONY: setup check fmt lint test doc deny fix clean

## Setup — run once after cloning
setup:
	git config core.hooksPath .githooks
	@echo "Git hooks installed (.githooks/)"
	@command -v cargo-deny >/dev/null 2>&1 || { echo "Installing cargo-deny..."; cargo install cargo-deny --locked; }
	@echo "Setup complete."

## Run all checks (same as pre-commit hook)
check: fmt lint test doc deny
	@echo "All checks passed."

## Formatting
fmt:
	cargo fmt --check

## Linting
lint:
	cargo clippy --workspace --all-targets -- -D warnings

## Tests
test:
	cargo test --workspace

## Documentation (warnings = errors)
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

## Dependency audit
deny:
	cargo deny check

## Auto-fix formatting and lint suggestions
fix:
	cargo fmt
	cargo clippy --workspace --all-targets --fix --allow-dirty

## Clean build artifacts
clean:
	cargo clean
