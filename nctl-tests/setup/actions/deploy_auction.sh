#!/bin/bash

# This step is necessary to make sure the path to auction wasm works correctly
CWD=$(pwd)
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"

SELLER_ACCOUNT_ARG=$1
TOKEN_CONTRACT_HASH_ARG=$2
KYC_PACKAGE_HASH_ARG=$3
TOKEN_ID_ARG=$4
FORMAT=$5
RESERVE_PRICE=$6
STARTING_PRICE=$7
# current milliseconds + 1.5 minutes
START_TIME=`expr $(date "+%s%3N") + 90000`
# plus 5 minutes
CANCEL_TIME=`expr $START_TIME + 300000`
# plus 10 minutes
END_TIME=`expr $START_TIME + 600000`

AUCTION_INSTALL_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-path $AUCTION_WASM\
  --session-arg "beneficiary_account:key='$SELLER_ACCOUNT_ARG'"\
  --session-arg "token_contract_hash:key='$TOKEN_CONTRACT_HASH_ARG'"\
  --session-arg "kyc_package_hash:key='$KYC_PACKAGE_HASH_ARG'"\
  --session-arg "format:string='$FORMAT'"\
  --session-arg "starting_price:opt_u512=$STARTING_PRICE"\
  --session-arg "reserve_price:u512='$RESERVE_PRICE'"\
  --session-arg "token_id:string='$TOKEN_ID_ARG'"\
  --session-arg "start_time:u64='$START_TIME'"\
  --session-arg "cancellation_time:u64='$CANCEL_TIME'"\
  --session-arg "end_time:u64='$END_TIME'"\
  | jq .result.deploy_hash\
  | tr -d '"')

sleep 90

cd $CWD

STATE=$(casper-client get-state-root-hash\
  --node-address $NODE_1_ADDRESS\
  | jq .result.state_root_hash\
  | tr -d '"')

AUCTION_PACKAGE_HASH=$(casper-client query-state\
  --state-root-hash $STATE\
  --key $SELLER_KEY\
  --node-address $NODE_1_ADDRESS\
  | jq '.result.stored_value.Account.named_keys[] | select(.name == "auction_contract_package_hash") | .key'\
  | tr -d '"')

AUCTION_CONTRACT_HASH=$(casper-client query-state\
  --state-root-hash $STATE\
  --key $AUCTION_PACKAGE_HASH\
  --node-address $NODE_1_ADDRESS\
  | jq .result.stored_value.ContractPackage.versions[0].contract_hash\
  | sed 's/contract/hash/'\
  | tr -d '"')

echo "Installed auction contract with $AUCTION_CONTRACT_HASH"