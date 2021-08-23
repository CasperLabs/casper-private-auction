RESULT=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-4ab94e0472eeabde179b881e570f77458eb726916f8f04afecacd927c7fe9dba\
  --session-entry-point "bid"\
  --session-arg "bid:u512='666'"\
  --session-arg "bid_purse:uref='$SELLER_PURSE'"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-020b4ca13873dfebd683248cb484044a5559f33a44393487d9ddfb74ad7cd6a0\
  --session-entry-point "finalize"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT_2_BID=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_2_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-ed8cce7d8a59c79928f08843481a05feafc1ccb963922e458a94e1b14ef3e530\
  --session-entry-point "bid"\
  --session-arg "bid:u512='1200'"\
  --session-arg "bid_purse:uref='$BUYER_2_PURSE'"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT_3_BID=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_3_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-ed8cce7d8a59c79928f08843481a05feafc1ccb963922e458a94e1b14ef3e530\
  --session-entry-point "bid"\
  --session-arg "bid:u512='1500'"\
  --session-arg "bid_purse:uref='$BUYER_3_PURSE'"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT_4_BID=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_4_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-020b4ca13873dfebd683248cb484044a5559f33a44393487d9ddfb74ad7cd6a0\
  --session-entry-point "bid"\
  --session-arg "bid:u512='2000'"\
  --session-arg "bid_purse:uref='$BUYER_4_PURSE'"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT_4_CANCEL=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_4_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-020b4ca13873dfebd683248cb484044a5559f33a44393487d9ddfb74ad7cd6a0\
  --session-entry-point "cancel_bid"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT_3_CANCEL=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_3_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-0f70f388eb871354a77572a615c473f3f755aa63845767b56a1a5fe976978a86\
  --session-entry-point "cancel_bid"\
  | jq .result.deploy_hash\
  | tr -d '"')

RESULT=$(casper-client put-deploy\
  --chain-name $NETWORK_NAME\
  --node-address $NODE_1_ADDRESS\
  --secret-key $USER_1_SECRET_KEY\
  --payment-amount $GAS_LIMIT\
  --session-hash hash-d93d3e92eef7c97ee9cd6c22d69f3d3ba88a20e6dd2d916a913345b71fa34d13\
  --session-entry-point "bid"\
  --session-arg "bid:u512='1200'"\
  --session-arg "bid_purse:uref='$SELLER_PURSE'"\
  | jq .result.deploy_hash\
  | tr -d '"')