#!/bin/bash

KYC_RECIPIENT_ARG=$1

KYC_MINT_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-hash $KYC_CONTRACT_HASH\
        --session-entry-point "mint"\
        --session-arg "recipient:key='$KYC_RECIPIENT_ARG'"\
        --session-arg "token_id:opt_string=null"\
        --session-arg "token_meta:string=''"\
        | jq .result.deploy_hash\
        | tr -d '"')