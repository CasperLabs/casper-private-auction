#!/bin/bash

CWD=$(pwd)
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

# Auction parameters
SELLER_ACCOUNT_ARG=$SELLER_KEY
TOKEN_PACKAGE_HASH_ARG=$TOKEN_PACKAGE_HASH
KYC_PACKAGE_HASH_ARG=$KYC_PACKAGE_HASH
TOKEN_ID_ARG=$TOKEN_ID
FORMAT="ENGLISH"
RESERVE_PRICE=500
STARTING_PRICE=null

# KYC grants
. ../setup/actions/mint_kyc.sh $BUYER_2_KEY
. ../setup/actions/mint_kyc.sh $BUYER_3_KEY

# Bids
BID_1=400
BID_2=600
SECRET_KEY_1=$USER_2_SECRET_KEY
SECRET_KEY_2=$USER_3_SECRET_KEY
PURSE_1=$BUYER_2_PURSE
PURSE_2=$BUYER_3_PURSE

. ../setup/actions/deploy_auction.sh $SELLER_ACCOUNT_ARG $TOKEN_PACKAGE_HASH_ARG $KYC_PACKAGE_HASH_ARG $TOKEN_ID_ARG $FORMAT $RESERVE_PRICE $STARTING_PRICE

ENGLISH_AUCTION_HASH=$AUCTION_CONTRACT_HASH

. ../operation/bid.sh $SECRET_KEY_1 $ENGLISH_AUCTION_HASH $BID_1 $PURSE_1
BID_1_DEPLOY=$BID_DEPLOY

. ../operation/bid.sh $SECRET_KEY_2 $ENGLISH_AUCTION_HASH $BID_2 $PURSE_2
BID_2_DEPLOY=$BID_DEPLOY

sleep 90

echo "Observed bid status, bid amount $BID_1, should fail with error 3 as too low"
sleep 2
nctl-view-chain-deploy deploy=$BID_1_DEPLOY
echo "Observed bid status, bid amount $BID_2"
sleep 2
nctl-view-chain-deploy deploy=$BID_2_DEPLOY

cd $CWD

# STATE=$(casper-client get-state-root-hash\
#   --node-address $NODE_1_ADDRESS\
#   | jq .result.state_root_hash\
#   | tr -d '"')