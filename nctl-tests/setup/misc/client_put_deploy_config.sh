#!/bin/bash

NETWORK_NAME=casper-net-1
NODE_1_RPC_PORT=11101
NODE_1_ADDRESS=http://localhost:$NODE_1_RPC_PORT
USER_1_SECRET_KEY=$NCTL/assets/net-1/users/user-1/secret_key.pem
USER_2_SECRET_KEY=$NCTL/assets/net-1/users/user-2/secret_key.pem
USER_3_SECRET_KEY=$NCTL/assets/net-1/users/user-3/secret_key.pem
USER_4_SECRET_KEY=$NCTL/assets/net-1/users/user-4/secret_key.pem
USER_5_SECRET_KEY=$NCTL/assets/net-1/users/user-5/secret_key.pem
GAS_LIMIT=1000000000000
#AUCTION_WASM=~/CasperLabs/casper-private-auction/target/wasm32-unknown-unknown/release/casper-private-auction-installer.wasm
#NFT_WASM=~/CasperLabs/CaskNFT/target/wasm32-unknown-unknown/release/cask-token.wasm
AUCTION_WASM=../tests/wasm/casper-private-auction-installer.wasm
KYC_WASM=setup/fixtures/contracts/civic-token.wasm
NFT_WASM=setup/fixtures/contracts/cask-token.wasm

# Make sure rust-script output is readable
export RUST_LOG=rust-script=error