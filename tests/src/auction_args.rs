use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, ContractPackageHash, Key, PublicKey,
    RuntimeArgs, SecretKey, U512,
};

#[derive(Debug)]
pub struct AuctionArgsBuilder {
    // into Key
    beneficiary_account: AccountHash,
    // into Key
    token_contract_hash: ContractPackageHash,
    // into Key
    kyc_package_hash: ContractPackageHash,
    reserve_price: U512,
    starting_price: U512,
    token_id: String,
    pub start_time: u64,
    cancellation_time: u64,
    end_time: u64,
    name: String,
    bidder_count_cap: Option<u64>,
    auction_timer_extension: Option<u64>,
    minimum_bid_step: Option<U512>,
    // just a trigger on this level, not included into the actual args
    english: bool,
}

impl AuctionArgsBuilder {
    pub fn new_with_necessary(
        beneficiary: &AccountHash,
        token_contract_hash: &ContractPackageHash,
        kyc_package_hash: &ContractPackageHash,
        token_id: &str,
        english: bool,
        start_time: u64,
    ) -> Self {
        AuctionArgsBuilder {
            beneficiary_account: *beneficiary,
            token_contract_hash: *token_contract_hash,
            kyc_package_hash: *kyc_package_hash,
            reserve_price: U512::from(1000),
            starting_price: U512::from(100000),
            token_id: token_id.to_string(),
            start_time,
            cancellation_time: 3000,
            end_time: 3500,
            name: "test".to_string(),
            bidder_count_cap: None,
            auction_timer_extension: None,
            minimum_bid_step: None,
            english: true,
        }
    }

    pub fn set_beneficiary(&mut self, beneficiary: &AccountHash) {
        self.beneficiary_account = *beneficiary;
    }

    pub fn set_token_id(&mut self, token_id: &str) {
        self.token_id = token_id.to_string();
    }

    pub fn set_token_contract_hash(&mut self, token_contract_hash: &ContractPackageHash) {
        self.token_contract_hash = *token_contract_hash;
    }

    pub fn set_kyc_package_hash(&mut self, kyc_package_hash: &ContractPackageHash) {
        self.kyc_package_hash = *kyc_package_hash;
    }

    pub fn set_reserve_price(&mut self, reserve_price: U512) {
        self.reserve_price = reserve_price;
    }

    pub fn set_starting_price(&mut self, starting_price: U512) {
        self.starting_price = starting_price;
    }

    pub fn set_start_time(&mut self, start_time: u64) {
        self.start_time = start_time;
    }

    pub fn set_cancellation_time(&mut self, cancellation_time: u64) {
        self.cancellation_time = cancellation_time;
    }

    pub fn set_end_time(&mut self, end_time: u64) {
        self.end_time = end_time;
    }

    pub fn set_bidder_count_cap(&mut self, bidder_count_cap: Option<u64>) {
        self.bidder_count_cap = bidder_count_cap;
    }

    pub fn set_auction_timer_extension(&mut self, auction_timer_extension: Option<u64>) {
        self.auction_timer_extension = auction_timer_extension;
    }

    pub fn set_minimum_bid_step(&mut self, minimum_bid_step: Option<U512>) {
        self.minimum_bid_step = minimum_bid_step;
    }

    pub fn set_is_english(&mut self, english: bool) {
        self.english = english;
    }

    pub fn build(&self) -> RuntimeArgs {
        let mut args = runtime_args! {
            "beneficiary_account"=>Key::Account(self.beneficiary_account),
            "token_contract_hash"=>Key::Hash(self.token_contract_hash.value()),
            "kyc_package_hash"=>Key::Hash(self.kyc_package_hash.value()),
            "reserve_price"=>self.reserve_price,
            "token_id"=>self.token_id.to_owned(),
            "start_time" => self.start_time,
            "end_time" => self.start_time+self.end_time,
            "name" => self.name.clone(),
        };
        if self.english {
            args.insert("bidder_count_cap", self.bidder_count_cap);
            args.insert("auction_timer_extension", self.auction_timer_extension);
            args.insert("minimum_bid_step", self.minimum_bid_step);
            args.insert(
                "cancellation_time",
                self.start_time + self.cancellation_time,
            );
        } else {
            args.insert("starting_price", self.starting_price);
        }
        args
    }

    pub fn get_now_u64() -> u64 {
        use std::time::SystemTime;
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis() as u64,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
}

impl Default for AuctionArgsBuilder {
    fn default() -> Self {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let now: u64 = Self::get_now_u64();
        AuctionArgsBuilder {
            beneficiary_account: PublicKey::from(&admin_secret).to_account_hash(),
            token_contract_hash: ContractPackageHash::new([0u8; 32]),
            kyc_package_hash: ContractPackageHash::new([0u8; 32]),
            reserve_price: U512::from(1000),
            starting_price: U512::from(100000),
            token_id: "token_id".to_string(),
            start_time: now + 500,
            cancellation_time: 3000,
            end_time: 3500,
            name: "test".to_string(),
            bidder_count_cap: None,
            auction_timer_extension: None,
            minimum_bid_step: None,
            english: true,
        }
    }
}
