#!/bin/bash

. client_put_deploy_config.sh

SELLER_ACCOUNT_ARG=$1
TOKEN_CONTRACT_HASH_ARG=$2
TOKEN_ID_ARG=$3
FORMAT=$4
RESERVE_PRICE=$5
STARTING_PRICE=$6
# current milliseconds + 1.5 minutes
START_TIME=`expr $(date "+%s%3N") + 90000`
# plus 10 minutes
CANCEL_TIME=`expr $START_TIME + 600000`
# plus 20 minutes
END_TIME=`expr $START_TIME + 1200000`

AUCTION_INSTALL_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-path $AUCTION_WASM\
  --session-arg "beneficiary_account:key='$SELLER_ACCOUNT_ARG'"\
  --session-arg "token_contract_hash:key='$TOKEN_CONTRACT_HASH_ARG'"\
  --session-arg "format:string='$FORMAT'"\
  --session-arg "starting_price:opt_u512=$STARTING_PRICE"\
  --session-arg "reserve_price:u512='$RESERVE_PRICE'"\
  --session-arg "token_id:string='$TOKEN_ID_ARG'"\
  --session-arg "start_time:u64='$START_TIME'"\
  --session-arg "cancellation_time:u64='$CANCEL_TIME'"\
  --session-arg "end_time:u64='$END_TIME'"\
  | jq .result.deploy_hash\
  | tr -d '"')

STATE=$(casper-client get-state-root-hash\
  --node-address $NODE_1_ADDRESS\
  | jq .result.state_root_hash\
  | tr -d '"')