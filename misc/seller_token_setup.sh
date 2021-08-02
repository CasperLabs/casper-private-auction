#!/bin/bash

# WHAT DOES THIS DO?
# - sets up critical local variables for casper-client calls
# - retrieves user 1 (seller) key and main purse
# - deploys a toy NFT contract, mints a token and obtains both contract hash and token id

# ASSUMPTIONS:
# - running NCTL network
# - user 1 with sufficient tokens (normally the case!)

. client_put_deploy_config.sh

SELLER_KEY=$(nctl-view-user-account user=1\
  | grep -Pom1 "(?<=account_hash\": \")account-hash-[0-9|a-z]{64}")

SELLER_PURSE=$(nctl-view-user-account user=1\
  | grep -Po "(?<=main_purse\": \")uref-[0-9|a-z]{64}-007")

. deploy_nft.sh

sleep 90

# This needs to be changed to use jq
TOKEN_CONTRACT_HASH=$(nctl-view-user-account user=1\
  | grep -Pom1 "(?<=key\": \")hash-[0-9|a-z]{64}")

echo "Obtained seller key $SELLER_KEY, seller purse $SELLER_PURSE and contract hash $TOKEN_CONTRACT_HASH"

DRAGONS_MINT_DEPLOY=$(casper-client put-deploy\
        --chain-name $NETWORK_NAME\
        --node-address http://localhost:$NODE_1_RPC_PORT\
        --secret-key $USER_1_SECRET_KEY\
        --payment-amount $GAS_LIMIT\
        --session-hash $TOKEN_CONTRACT_HASH\
        --session-entry-point "mint_one"\
        --session-arg "recipient:key='$SELLER_KEY'"\
        --session-arg "token_meta:string=''"\
        | jq .result.deploy_hash\
        | tr -d '"')

sleep 90

STATE=$(casper-client get-state-root-hash\
  --node-address http://localhost:$NODE_1_RPC_PORT\
  | jq .result.state_root_hash\
  | tr -d '"')

TOKEN_ID=$(casper-client query-state\
  --node-address http://localhost:$NODE_1_RPC_PORT\
  --state-root-hash $STATE\
  --key $TOKEN_CONTRACT_HASH\
  | grep -Pom1 "(?<=metas_)[0-9]{1,}")

echo "Obtained token $TOKEN_ID"