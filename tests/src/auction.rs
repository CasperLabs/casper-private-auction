use std::collections::BTreeMap;

use crate::{auction_args::AuctionArgsBuilder, nft};
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
    pub admin: AccountHash,
    pub ali: AccountHash,
    pub bob: AccountHash,
}

impl AuctionContract {
    pub fn deploy_with_default_args(english: bool, start_time: u64) -> Self {
        let token_id = String::from("custom_token_id");
        let mut commissions = BTreeMap::new();
        let nft::CasperCEP47Contract {
            mut context,
            hash,
            kyc_hash,
            kyc_package_hash,
            nft_package,
            admin,
            ali,
            bob,
        } = Self::nft_deploy_and_mint(&token_id, commissions);
        let auction_args = AuctionArgsBuilder::new_with_necessary(
            &admin,
            &nft_package,
            &kyc_package_hash,
            &token_id,
            english,
            start_time,
        );
        Self::deploy_auction(
            context,
            auction_args.build(),
            auction_args.start_time,
            admin,
            ali,
            bob,
        )
    }

    pub fn deploy(mut auction_args: AuctionArgsBuilder) -> Self {
        let token_id = String::from("custom_token_id");
        let mut commissions = BTreeMap::new();
        let nft::CasperCEP47Contract {
            mut context,
            hash,
            kyc_hash,
            kyc_package_hash,
            nft_package,
            admin,
            ali,
            bob,
        } = Self::nft_deploy_and_mint(&token_id, commissions);
        auction_args.set_beneficiary(&admin);
        auction_args.set_token_contract_hash(&nft_package);
        auction_args.set_kyc_package_hash(&kyc_package_hash);
        auction_args.set_token_id(&token_id);
        let start_time = auction_args.start_time;
        Self::deploy_auction(
            context,
            auction_args.build(),
            auction_args.start_time,
            admin,
            ali,
            bob,
        )
    }

    pub fn nft_deploy_and_mint(
        token_id: &str,
        commissions: BTreeMap<String, String>,
    ) -> nft::CasperCEP47Contract {
        let mut cep47 = nft::CasperCEP47Contract::deploy();
        let token_meta = nft::meta::red_dragon();
        cep47.mint(
            &Key::Account(cep47.admin),
            token_id,
            &token_meta,
            &(cep47.admin.clone()),
            commissions,
        );

        cep47.add_kyc(cep47.admin);
        cep47.add_kyc(cep47.ali);
        cep47.add_kyc(cep47.bob);
        cep47
    }

    pub fn deploy_auction(
        mut context: TestContext,
        args: RuntimeArgs,
        start_time: u64,
        admin: AccountHash,
        ali: AccountHash,
        bob: AccountHash,
    ) -> Self {
        let session_code = Code::from("casper-private-auction-installer.wasm");
        let session = SessionBuilder::new(session_code, args)
            .with_address(admin)
            .with_authorization_keys(&[admin])
            .with_block_time(start_time - 1)
            .build();
        context.run(session);
        let contract_hash = context
            .query(admin, &["auction_contract_hash_wrapped".into()])
            .unwrap()
            .into_t()
            .unwrap();
        Self {
            context,
            contract_hash,
            admin,
            ali,
            bob,
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

    pub fn cancel_bid(&mut self, caller: &AccountHash, time: u64) {
        self.call(caller, "cancel_bid", runtime_args! {}, time)
    }

    pub fn finalize(&mut self, caller: &AccountHash, time: u64) {
        self.call(caller, "finalize", runtime_args! {}, time)
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
    fn call(&mut self, caller: &AccountHash, method: &str, args: RuntimeArgs, time: u64) {
        let code = Code::Hash(self.contract_hash, method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(*caller)
            .with_authorization_keys(&[*caller])
            .with_block_time(time)
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
            self.admin,
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

    pub fn get_account_balance(&self, account: &AccountHash) -> U512 {
        self.context
            .get_balance(self.context.main_purse_address(*account).unwrap().addr())
    }
}
