.PHONY: setup check fmt lint test doc deny readme readme-check fix clean

## Setup — run once after cloning
setup:
	git config core.hooksPath .githooks
	@echo "Git hooks installed (.githooks/)"
	@command -v cargo-deny >/dev/null 2>&1 || { echo "Installing cargo-deny..."; cargo install cargo-deny --locked; }
	@echo "Setup complete."

## Run all checks (same as pre-commit hook)
check: fmt lint test doc deny readme-check
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

## Generate README.md from code + template
readme:
	./scripts/generate-readme.sh

## Verify README.md is up to date
readme-check: readme
	@git diff --quiet README.md || { echo "error: README.md is out of date. Run 'make readme' and commit the result."; exit 1; }

## Auto-fix formatting and lint suggestions
fix:
	cargo fmt
	cargo clippy --workspace --all-targets --fix --allow-dirty

## Clean build artifacts
clean:
	cargo clean
