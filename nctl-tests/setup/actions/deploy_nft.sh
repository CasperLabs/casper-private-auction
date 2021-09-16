#!/bin/bash

NFT_INSTALL_DEPLOY=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-path $NFT_WASM\
  --session-arg "name:string='Dragon'"\
  --session-arg "symbol:string='DRG'"\
  --session-arg "meta:string=''"\
  --session-arg "admin:key='$SELLER_KEY'"\
  --session-arg "contract_name:string='TestCaskNFT'"\
  | jq .result.deploy_hash\
  | tr -d '"')