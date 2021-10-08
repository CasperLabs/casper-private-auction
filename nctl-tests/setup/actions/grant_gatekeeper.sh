#!/bin/bash

GATEKEEPER_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address $NODE_1_ADDRESS\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-hash $KYC_CONTRACT_HASH\
        --session-entry-point "grant_gatekeeper"\
        --session-arg "gatekeeper:key='$SELLER_KEY'"\
        | jq .result.deploy_hash\
        | tr -d '"')