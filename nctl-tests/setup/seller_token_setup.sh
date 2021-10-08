#!/bin/bash

CWD=$(pwd)
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$parent_path"
. misc/client_put_deploy_config.sh
cd $CWD

SELLER_KEY=$(nctl-view-user-account user=1\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

SELLER_PURSE=$(nctl-view-user-account user=1\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

BUYER_2_KEY=$(nctl-view-user-account user=2\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

BUYER_2_PURSE=$(nctl-view-user-account user=2\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

BUYER_3_KEY=$(nctl-view-user-account user=3\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

BUYER_3_PURSE=$(nctl-view-user-account user=3\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

BUYER_4_KEY=$(nctl-view-user-account user=4\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

BUYER_4_PURSE=$(nctl-view-user-account user=4\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

BUYER_5_KEY=$(nctl-view-user-account user=5\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

BUYER_5_PURSE=$(nctl-view-user-account user=5\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

. setup/actions/deploy_kyc.sh

sleep 90

KYC_PACKAGE_HASH=$(nctl-view-user-account user=1\
  | tr -d "\n"\
  | grep -o  "{.*"\
  | jq '.stored_value.Account.named_keys[] | select(.name == "TestKYCNFT_package_hash") | .key'\
  | tr -d '"')

. setup/actions/deploy_nft.sh

sleep 90

TOKEN_CONTRACT_HASH=$(nctl-view-user-account user=1\
  | tr -d "\n"\
  | grep -o  "{.*"\
  | jq '.stored_value.Account.named_keys[] | select(.name == "TestCaskNFT_contract_hash") | .key'\
  | tr -d '"')

echo "Obtained the following hashes:
 seller key - $SELLER_KEY
 KYC contract package hash - $KYC_CONTRACT_HASH
 NFT contract hash - $TOKEN_CONTRACT_HASH
 "

echo "Constructing complex args"

cd setup/fixtures
./metacask-runtime-arg-builder $SELLER_KEY $SELLER_KEY "artist,$BUYER_5_KEY,100|broker,$BUYER_4_KEY,200"
cd ../..

. setup/actions/mint_nft.sh

sleep 90

STATE=$(casper-client get-state-root-hash\
  --node-address $NODE_1_ADDRESS\
  | jq .result.state_root_hash\
  | tr -d '"')

OWNED_TOKENS_DICT=$(casper-client query-state\
  --node-address $NODE_1_ADDRESS\
  --state-root-hash $STATE\
  --key $TOKEN_CONTRACT_HASH\
  | jq '.result.stored_value.Contract.named_keys[] | select(.name == "owned_tokens_by_index") | .key'\
  | tr -d '"')

TOKEN_INDEX=$(rust-script setup/misc/encode_owner_token.rs $SELLER_KEY 0 | tail -1)

TOKEN_ID=$(casper-client get-dictionary-item\
  --node-address $NODE_1_ADDRESS\
  --state-root-hash $STATE\
  --seed-uref $OWNED_TOKENS_DICT\
  --dictionary-item-key $TOKEN_INDEX\
  | jq .result.stored_value.CLValue.parsed\
  | tr -d '"')

echo "Obtained token with id (expected 'test_token'):
 $TOKEN_ID"