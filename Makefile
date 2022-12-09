prepare:
	rustup default nightly
	rustup target add wasm32-unknown-unknown

rust-test-only:
	cargo test -p tests

copy-wasm-files-to-tests:
	cp target/wasm32-unknown-unknown/release/*.wasm tests/wasm
	cp nctl-tests/setup/fixtures/contracts/cask-token.wasm tests/wasm/nft-contract.wasm
	cp nctl-tests/setup/fixtures/contracts/civic-token.wasm tests/wasm/kyc-contract.wasm
	cp tests/wasm/*.wasm example/casper-private-auction-tests/wasm/

test: build-contract copy-wasm-files-to-tests rust-test-only

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


