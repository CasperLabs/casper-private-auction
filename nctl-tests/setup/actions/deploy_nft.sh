#!/bin/bash

NFT_INSTALL_DEPLOY=$(casper-client put-deploy\
	--chain-name $NETWORK_NAME\
	--node-address $NODE_1_ADDRESS\
	--secret-key $USER_1_SECRET_KEY\
	--payment-amount $GAS_LIMIT\
	--session-path $NFT_WASM\
	--session-arg "token_name:string='Dragon'"\
	--session-arg "token_symbol:string='DRG'"\
	--session-arg "token_meta:string=''"\
	| jq .result.deploy_hash\
	| tr -d '"')