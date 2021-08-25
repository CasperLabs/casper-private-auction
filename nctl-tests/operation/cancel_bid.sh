#!/bin/bash

USER_SECRET_KEY_ARG=$1
AUCTION_HASH_ARG=$2

CANCEL_BID_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_SECRET_KEY_ARG\
  --payment-amount $GAS_LIMIT\
  --session-hash $AUCTION_HASH_ARG\
  --session-entry-point "cancel_bid"\
  | jq .result.deploy_hash\
  | tr -d '"')

sleep 90

STATE=$(casper-client get-state-root-hash\
  --node-address $NODE_1_ADDRESS\
  | jq .result.state_root_hash\
  | tr -d '"')