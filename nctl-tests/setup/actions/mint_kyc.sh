#!/bin/bash

CWD_KYC=$(pwd)
parent_path_KYC=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path_KYC"

KYC_RECIPIENT_ARG=$1

# Note that this recreates all of the .json files every time
cd ../fixtures
./metacask-runtime-arg-builder $KYC_RECIPIENT_ARG $KYC_RECIPIENT_ARG "artist,$BUYER_5_KEY,100|broker,$BUYER_4_KEY,200"
cd "$parent_path_KYC"

KYC_MINT_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-hash $KYC_CONTRACT_HASH\
        --session-entry-point "mint"\
        --session-args-complex "../fixtures/arg-files/kyc_mint_args.json"\
        | jq .result.deploy_hash\
        | tr -d '"')

cd $CWD_KYC