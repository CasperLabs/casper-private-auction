use std::collections::BTreeMap;

use casper_engine_test_support::{
    internal::TIMESTAMP_MILLIS_INCREMENT, Code, Hash, SessionBuilder, TestContext,
    TestContextBuilder,
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};
/*
  --session-arg "beneficiary_account    :   key         ='$SELLER_ACCOUNT_ARG'"\
  --session-arg "token_contract_hash    :   key         ='$TOKEN_CONTRACT_HASH_ARG'"\
  --session-arg "format                 :   string      ='$FORMAT'"\
  --session-arg "starting_price         :   opt_u512    =$STARTING_PRICE"\
  --session-arg "reserve_price          :   u512        ='$RESERVE_PRICE'"\
  --session-arg "token_id               :   string      ='$TOKEN_ID_ARG'"\
  --session-arg "start_time             :   u64         ='$START_TIME'"\
  --session-arg "cancellation_time      :   u64         ='$CANCEL_TIME'"\
  --session-arg "end_time               :   u64         ='$END_TIME'"\
*/
pub fn get_now_u64() -> u64 {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis() as u64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn deploy_args(
    beneficiary: &AccountHash,
    nft: &Hash,
    token_id: &str,
    english: bool,
) -> RuntimeArgs {
    let now: u64 = get_now_u64();
    runtime_args! {
        "beneficiary_account"=>Key::Account(*beneficiary),
        "token_contract_hash"=>Key::Hash(*nft),
        "format"=>if english{"ENGLISH"}else{"DUTCH"},
        "starting_price"=>if english{None}else{Some(U512::from(1000))},
        "reserve_price"=>U512::from(1000),
        "token_id"=>token_id,
        "start_time" => now + 500,
        "cancellation_time" => now + 3000,
        "end_time" => now + 3500,
    }
}
pub struct AuctionContract {
    pub context: TestContext,
    pub contract_hash: Hash,
    pub deployer: AccountHash,
}

impl AuctionContract {
    pub fn deploy(mut context: TestContext, nft: Hash, token_id: &str, english: bool) -> Self {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let public_key: PublicKey = (&admin_secret).into();
        let deployer = AccountHash::from(&public_key);

        let session_code = Code::from("casper-private-auction-installer.wasm");
        let session = SessionBuilder::new(
            session_code,
            deploy_args(&deployer, &nft, token_id, english),
        )
        .with_address(deployer)
        .with_authorization_keys(&[deployer])
        .with_block_time(get_now_u64())
        .build();
        context.run(session);
        let contract_hash = context
            .query(deployer, &["auction_contract_hash_wrapped".into()])
            .unwrap()
            .into_t()
            .unwrap();
        Self {
            context,
            contract_hash,
            deployer,
        }
    }

    pub fn bid(&mut self, bid: U512) {
        self.call(
            "bid",
            runtime_args! {
                //"bid" => bid,
                //"bid_purse" => URef
            },
        )
    }

    pub fn cancel(&mut self) {
        self.call("cancel", runtime_args! {})
    }

    pub fn finalize(&mut self) {
        self.call("finalize", runtime_args! {})
    }

    pub fn is_finalized(&self) -> bool {
        self.query_contract(self.contract_hash, "finalized")
            .unwrap()
    }

    pub fn get_end(&self) -> u64 {
        self.query_contract(self.contract_hash, "end_time").unwrap()
    }

    pub fn get_event(&self, contract_hash: [u8; 32], index: u32) -> BTreeMap<String, String> {
        self.query_dictionary_value(
            contract_hash,
            if contract_hash != self.contract_hash {
                "events"
            } else {
                "auction_events"
            },
            &index.to_string(),
        )
        .unwrap()
    }

    pub fn get_events(&self, contract_hash: [u8; 32]) -> Vec<BTreeMap<String, String>> {
        let mut events = Vec::new();
        for i in 0..self.get_events_count(contract_hash) {
            events.push(self.get_event(contract_hash, i));
        }
        events
    }

    pub fn get_events_count(&self, contract_hash: [u8; 32]) -> u32 {
        self.query_contract(
            contract_hash,
            if contract_hash != self.contract_hash {
                "events_count"
            } else {
                "auction_events_count"
            },
        )
        .unwrap()
    }

    /// Wrapper function for calling an entrypoint on the contract with the access rights of the deployer.
    fn call(&mut self, method: &str, args: RuntimeArgs) {
        let code = Code::Hash(self.contract_hash, method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(self.deployer)
            .with_authorization_keys(&[self.deployer])
            .with_block_time(get_now_u64())
            .build();
        self.context.run(session);
    }

    /// Wrapper for querying a dictionary entry.
    pub fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        contract_hash: [u8; 32],
        dict_name: &str,
        key: &str,
    ) -> Option<T> {
        // We can construct the first parameter for this call with either the hash of the function,
        // or the address of the deployer, depending on where we initiated the dictionary.
        // In this example the dictionary can be reached from both.
        match self.context.query_dictionary_item(
            Key::Hash(contract_hash),
            Some(dict_name.to_string()),
            key.to_string(),
        ) {
            Err(_) => None,
            Ok(maybe_value) => maybe_value.into_t().unwrap(),
        }
    }

    fn query_contract<T: CLTyped + FromBytes>(
        &self,
        contract_hash: [u8; 32],
        name: &str,
    ) -> Option<T> {
        match self.context.query(
            self.deployer,
            &[
                if contract_hash != self.contract_hash {
                    "DragonsNFT_contract".to_string()
                } else {
                    "auction_contract_hash".to_string()
                },
                name.to_string(),
            ],
        ) {
            Err(e) => panic!("{:?}", e),
            Ok(maybe_value) => {
                let value = maybe_value.into_t().unwrap();
                Some(value)
            }
        }
    }
}
