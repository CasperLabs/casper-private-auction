#!/bin/bash

CWD=$(pwd)
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

# Auction parameters
SELLER_ACCOUNT_ARG=$SELLER_KEY
TOKEN_CONTRACT_HASH_ARG=$TOKEN_CONTRACT_HASH
TOKEN_ID_ARG=$TOKEN_ID
FORMAT="DUTCH"
RESERVE_PRICE=500
STARTING_PRICE="'1000'"

# Bids
BID_1=1200
SECRET_KEY_1=$USER_2_SECRET_KEY
PURSE_1=$BUYER_2_PURSE

. ../setup/actions/deploy_auction.sh $SELLER_ACCOUNT_ARG $TOKEN_CONTRACT_HASH_ARG $TOKEN_ID_ARG $FORMAT $RESERVE_PRICE $STARTING_PRICE

DUTCH_AUCTION_HASH=$AUCTION_CONTRACT_HASH

. ../operation/bid.sh $SECRET_KEY_1 $DUTCH_AUCTION_HASH $BID_1 $PURSE_1
BID_1_DEPLOY=$BID_DEPLOY

sleep 90

echo "Observed bid status, bid amount $BID_1, should succeed"
sleep 2
nctl-view-chain-deploy deploy=$BID_1_DEPLOY

cd $CWD

STATE=$(casper-client get-state-root-hash\
  --node-address $NODE_1_ADDRESS\
  | jq .result.state_root_hash\
  | tr -d '"')