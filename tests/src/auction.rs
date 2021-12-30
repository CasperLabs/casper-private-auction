use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    auction_args::AuctionArgsBuilder,
    nft,
    utils::{deploy, query, DeploySource, query_dictionary_item},
};
use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, URef, U512,
};

use crate::Hash;

pub struct AuctionContract {
    pub builder: InMemoryWasmTestBuilder,
    pub contract_hash: Hash,
    pub contract_package: Hash,
    pub admin: AccountHash,
    pub ali: AccountHash,
    pub bob: AccountHash,
}

impl AuctionContract {
    pub fn deploy_with_default_args(english: bool, start_time: u64) -> Self {
        let token_id = String::from("custom_token_id");
        let mut commissions = BTreeMap::new();
        let nft::CasperCEP47Contract {
            builder: mut context,
            nft_hash: hash,
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
            builder: mut context,
            nft_hash: hash,
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
        mut builder: InMemoryWasmTestBuilder,
        args: RuntimeArgs,
        start_time: u64,
        admin: AccountHash,
        ali: AccountHash,
        bob: AccountHash,
    ) -> Self {
        let auction_code = PathBuf::from("casper-private-auction-installer.wasm");
        deploy(
            &mut builder,
            &admin,
            &DeploySource::Code(auction_code),
            args,
            true,
            Some(start_time - 1),
        );

        let contract_hash = query(
            &builder,
            Key::Account(admin),
            &["test_auction_contract_hash_wrapped".to_string()],
        );
        let contract_package = query(
            &builder,
            Key::Account(admin),
            &["test_auction_contract_package_hash".to_string()],
        );

        Self {
            builder,
            contract_hash,
            contract_package,
            admin,
            ali,
            bob,
        }
    }

    pub fn bid(&mut self, bidder: &AccountHash, bid: U512, block_time: u64) {
        let session_code = PathBuf::from("bid-purse.wasm");
        deploy(
            &mut self.builder,
            &bidder,
            &DeploySource::Code(session_code),
            runtime_args! {
                "bid" => bid,
                "auction_contract" => self.contract_hash
            },
            true,
            Some(block_time),
        );
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
            Key::Hash(contract_hash),
            if contract_hash != self.contract_hash {
                "events"
            } else {
                "auction_events"
            },
            index.to_string(),
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
        deploy(
            &mut self.builder,
            &caller,
            &DeploySource::ByHash {
                hash: self.contract_package,
                method: method.to_string(),
            },
            args,
            true,
            Some(time),
        );
    }

    fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        base: Key,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        query_dictionary_item(
            &self.builder,
            base,
            Some(dict_name.to_string()),
            key,
        ).expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t()
        .expect("Wrong type in query result.")
    }

    fn query_contract<T: CLTyped + FromBytes>(
        &self,
        contract_hash: [u8; 32],
        name: &str,
    ) -> Option<T> {
        query(
            &self.builder,
            Key::Account(self.admin),
            &[
                if contract_hash != self.contract_hash {
                    "DragonsNFT_contract".to_string()
                } else {
                    "test_auction_contract_hash".to_string()
                },
                name.to_string(),
            ],
        )
    }

    /// Getter function for the balance of an account.
    fn get_account_balance(&self, account_key: &AccountHash) -> U512 {
        let account = self
            .builder
            .get_account(*account_key)
            .expect("should get genesis account");
        self.builder.get_purse_balance(account.main_purse())
    }

    /// Shorthand to get the balances of all 3 accounts in order.
    pub fn get_all_accounts_balance(&self) -> (U512, U512, U512) {
        (
            self.get_account_balance(&self.admin),
            self.get_account_balance(&self.ali),
            self.get_account_balance(&self.bob),
        )
    }
}
