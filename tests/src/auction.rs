use std::collections::BTreeMap;

use crate::auction_args::AuctionArgsBuilder;
use casper_engine_test_support::{
    internal::TIMESTAMP_MILLIS_INCREMENT, Code, Hash, SessionBuilder, TestContext,
    TestContextBuilder,
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};

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
        let now = AuctionArgsBuilder::get_now_u64();
        let auction_args =
            AuctionArgsBuilder::new_with_necessary(&deployer, &nft, token_id, english, now + 500);
        let session_code = Code::from("casper-private-auction-installer.wasm");
        let session = SessionBuilder::new(session_code, auction_args.build())
            .with_address(deployer)
            .with_authorization_keys(&[deployer])
            .with_block_time(now)
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

    pub fn bid(&mut self, bidder: &AccountHash, bid: U512, block_time: u64) {
        let session_code = Code::from("bid-purse.wasm");
        let session = SessionBuilder::new(
            session_code,
            runtime_args! {
                "bid" => bid,
                "auction_contract" => self.contract_hash
            },
        )
        .with_address(*bidder)
        .with_authorization_keys(&[*bidder])
        .with_block_time(block_time)
        .build();
        self.context.run(session);
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

    pub fn get_winner(&self) -> Option<AccountHash> {
        self.query_contract(self.contract_hash, "current_winner")
            .unwrap()
    }

    pub fn get_winning_bid(&self) -> Option<U512> {
        self.query_contract(self.contract_hash, "winning_bid")
            .unwrap()
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
            .with_block_time(AuctionArgsBuilder::get_now_u64())
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
