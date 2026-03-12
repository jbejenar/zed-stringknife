.PHONY: all build test lint fmt check clean setup doctor watch bench

all: check test lint fmt

# Build all crates
build:
	cargo build --manifest-path stringknife-core/Cargo.toml
	cargo build --manifest-path stringknife-lsp/Cargo.toml

# Run all tests
test:
	cargo test --manifest-path stringknife-core/Cargo.toml
	cargo test --manifest-path stringknife-lsp/Cargo.toml

# Run clippy lints
lint:
	cargo clippy --manifest-path stringknife-core/Cargo.toml -- -D warnings
	cargo clippy --manifest-path stringknife-lsp/Cargo.toml -- -D warnings

# Check formatting
fmt:
	cargo fmt --manifest-path stringknife-core/Cargo.toml --check
	cargo fmt --manifest-path stringknife-lsp/Cargo.toml --check

# Format in-place
fmt-fix:
	cargo fmt --manifest-path stringknife-core/Cargo.toml
	cargo fmt --manifest-path stringknife-lsp/Cargo.toml

# Type check without building
check:
	cargo check --manifest-path stringknife-core/Cargo.toml
	cargo check --manifest-path stringknife-lsp/Cargo.toml

# Run performance benchmarks (T-410)
bench:
	cargo bench --manifest-path stringknife-core/Cargo.toml

# Check WASM extension compiles
check-wasm:
	RUSTUP_TOOLCHAIN=stable-aarch64-apple-darwin RUSTC=$$(rustup which rustc) cargo check --target wasm32-wasip1

# Watch mode for tests (requires cargo-watch)
watch:
	cargo watch -w stringknife-core/src -w stringknife-lsp/src -x 'test --manifest-path stringknife-core/Cargo.toml' -x 'test --manifest-path stringknife-lsp/Cargo.toml'

# Dependency audit
audit:
	cargo deny check --manifest-path stringknife-lsp/Cargo.toml

# Clean build artifacts
clean:
	cargo clean --manifest-path Cargo.toml
	cargo clean --manifest-path stringknife-core/Cargo.toml
	cargo clean --manifest-path stringknife-lsp/Cargo.toml

# First-time setup
setup:
	rustup target add wasm32-wasip1
	@command -v cargo-watch >/dev/null 2>&1 || echo "Optional: cargo install cargo-watch (for 'make watch')"
	@command -v cargo-deny >/dev/null 2>&1 || echo "Optional: cargo install cargo-deny (for 'make audit')"
	@echo "Setup complete. Run 'make test' to verify."

# Health check
doctor:
	@echo "Checking development environment..."
	@command -v rustc >/dev/null 2>&1 && echo "  rustc: $$(rustc --version)" || echo "  rustc: MISSING"
	@command -v cargo >/dev/null 2>&1 && echo "  cargo: $$(cargo --version)" || echo "  cargo: MISSING"
	@rustup target list --installed 2>/dev/null | grep -q wasm32-wasip1 && echo "  wasm32-wasip1: installed" || echo "  wasm32-wasip1: MISSING (run 'make setup')"
	@command -v cargo-deny >/dev/null 2>&1 && echo "  cargo-deny: installed" || echo "  cargo-deny: not installed (optional)"
	@command -v cargo-watch >/dev/null 2>&1 && echo "  cargo-watch: installed" || echo "  cargo-watch: not installed (optional)"
	@echo "Done."
