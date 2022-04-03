prepare:
	rustup default nightly
	rustup target add wasm32-unknown-unknown

rust-test-only:
	cargo test -p tests

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/*.wasm tests/wasm

test: build-contract copy-wasm-file-to-test rust-test-only

clippy:
	cargo clippy --all-targets --all -- -D warnings

check-lint: clippy
	cargo fmt --all -- --check

format:
	cargo fmt --all

lint: clippy format

build-contract:
	cargo build --release -p casper-private-auction-installer -p bid-purse --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/casper-private-auction-installer.wasm
	wasm-strip target/wasm32-unknown-unknown/release/bid-purse.wasm
	wasm-strip target/wasm32-unknown-unknown/release/extend-bid-purse.wasm
	wasm-strip target/wasm32-unknown-unknown/release/delta-bid-purse.wasm

clean:
	cargo clean


