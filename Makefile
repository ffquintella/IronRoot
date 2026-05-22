.PHONY: build test fmt lint check clean new docs install uninstall

build:
	cargo build --workspace

test:
	cargo test --workspace

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
	./docs/serve_docs.sh

# Install the ironroot bootstrapper into ~/.cargo/bin.
install:
	cargo install --path crates/new --locked --force

# Remove the ironroot bootstrapper from ~/.cargo/bin.
uninstall:
	cargo uninstall ironroot
