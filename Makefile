prepare:
	rustup default nightly-2021-06-17-x86_64-unknown-linux-gnu
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p casper-private-auction-installer --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/casper-private-auction-installer.wasm

clean:
	cargo clean
