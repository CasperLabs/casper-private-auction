prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p casper-private-auction-installer --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/casper-private-auction-installer.wasm

test-only:
	cargo test -p tests

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/*.wasm tests/wasm

test: build-contract copy-wasm-file-to-test test-only

clean:
	cargo clean

clippy:
	cargo clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints

check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all