#!/bin/bash

CWD=$(pwd)
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

NFT_MINT_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-hash $TOKEN_CONTRACT_HASH\
        --session-entry-point "mint"\
        --session-args-complex "../fixtures/arg-files/nft_mint_args.json"\
        | jq .result.deploy_hash\
        | tr -d '"')

cd $CWD