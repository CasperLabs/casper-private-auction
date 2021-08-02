#!/bin/bash

. client_put_deploy_config.sh

NFT_WASM=~/CasperLabs/casper-nft-cep47/target/wasm32-unknown-unknown/release/dragons-nft.wasm

NFT_INSTALL_DEPLOY=$(casper-client put-deploy\
	--chain-name $NETWORK_NAME\
	--node-address http://localhost:$NODE_1_RPC_PORT\
	--secret-key $USER_1_SECRET_KEY\
	--payment-amount $GAS_LIMIT\
	--session-path $NFT_WASM\
	--session-arg "token_name:string='Dragon'"\
	--session-arg "token_symbol:string='DRG'"\
	--session-arg "token_meta:string=''"\
	| jq .result.deploy_hash\
	| tr -d '"')