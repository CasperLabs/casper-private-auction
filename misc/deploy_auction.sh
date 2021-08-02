#!/bin/bash

. client_put_deploy_config.sh

AUCTION_WASM=~/CasperLabs/casper-private-auction/target/wasm32-unknown-unknown/release/casper-private-auction-installer.wasm

# NOTE: Change variable names to avoid over-writing if some arguments aren't supplied by mistake
SELLER_PURSE=$1
TOKEN_CONTRACT_HASH=$2
FORMAT="English"
RESERVE_PRICE=0
STARTING_PRICE=null
TOKEN_ID=$3
# current milliseconds + 1.5 minutes
START_TIME=`expr $(date "+%s%3N") + 90000`
# plus 10 minutes
CANCEL_TIME=`expr $START_TIME + 600000`
# plus 20 minutes
END_TIME=`expr $START_TIME + 1200000`

AUCTION_INSTALL_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
	--node-address http://localhost:$NODE_1_RPC_PORT\
	--secret-key $USER_1_SECRET_KEY\
	--payment-amount $GAS_LIMIT\
	--session-path $AUCTION_WASM\
  --session-arg "seller_purse:uref='$SELLER_PURSE'"\
	--session-arg "token_contract_hash:key='$TOKEN_CONTRACT_HASH'"\
	--session-arg "format:string='$FORMAT'"\
	--session-arg "starting_price:opt_u512=$STARTING_PRICE"\
	--session-arg "reserve_price:u512='$RESERVE_PRICE'"\
	--session-arg "token_id:string='$TOKEN_ID'"\
	--session-arg "start_time:u64='$START_TIME'"\
	--session-arg "cancellation_time:u64='$CANCEL_TIME'"\
	--session-arg "end_time:u64='$END_TIME'"\
	| jq .result.deploy_hash\
	| tr -d '"')