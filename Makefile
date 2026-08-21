.PHONY: build test test-memory fmt lint check clean new docs install uninstall

build:
	cargo build --workspace

test: test-memory
	cargo test --workspace

# Cross-session recall is Python, so `cargo test` cannot reach it.
# See the Session Recall section of ai/AGENTS.md.
test-memory:
	python3 ai/memory/test_recall.py

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check:
	cargo check --workspace --all-targets

clean:
	cargo clean

# Build and launch the interactive project bootstrapper.
new:
	cargo run -p ironroot --quiet

# Serve the docsify documentation site at http://localhost:3000.
docs:
	./docs/serve-docs.sh

# Install the ironroot bootstrapper into ~/.cargo/bin.
install:
	cargo install --path crates/new --locked --force

# Remove the ironroot bootstrapper from ~/.cargo/bin.
uninstall:
	cargo uninstall ironroot
