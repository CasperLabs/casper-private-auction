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
    // true is `ENGLISH` | false is `DUTCH`
    is_english: bool,
    // ENGLISH format cannot have a starting price, build turn it into option
    starting_price: Option<U512>,
    reserve_price: U512,
    token_id: String,
    pub start_time: u64,
    cancellation_time: u64,
    end_time: u64,
    name: String,
    bidder_count_cap: Option<u64>,
    auction_timer_extension: Option<u64>,
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
            is_english: english,
            starting_price: None,
            reserve_price: U512::from(1000),
            token_id: token_id.to_string(),
            start_time,
            cancellation_time: 3000,
            end_time: 3500,
            name: "test".to_string(),
            bidder_count_cap: None,
            auction_timer_extension: None,
        }
    }

    pub fn set_english(&mut self) {
        self.is_english = true;
    }

    pub fn set_dutch(&mut self) {
        self.is_english = false;
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

    pub fn set_starting_price(&mut self, starting_price: Option<U512>) {
        self.starting_price = starting_price;
    }

    pub fn set_reserve_price(&mut self, reserve_price: U512) {
        self.reserve_price = reserve_price;
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

    pub fn build(&self) -> RuntimeArgs {
        runtime_args! {
            "beneficiary_account"=>Key::Account(self.beneficiary_account),
            "token_contract_hash"=>Key::Hash(self.token_contract_hash.value()),
            "kyc_package_hash"=>Key::Hash(self.kyc_package_hash.value()),
            "format"=>if self.is_english{"ENGLISH"}else{"DUTCH"},
            "starting_price"=> self.starting_price,
            "reserve_price"=>self.reserve_price,
            "token_id"=>self.token_id.to_owned(),
            "start_time" => self.start_time,
            "cancellation_time" => self.start_time+self.cancellation_time,
            "end_time" => self.start_time+self.end_time,
            "name" => self.name.clone(),
            "bidder_count_cap" => self.bidder_count_cap,
            "auction_timer_extension" => self.auction_timer_extension
        }
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
            beneficiary_account: AccountHash::from(&(&admin_secret).into()),
            token_contract_hash: ContractPackageHash::new([0u8; 32]),
            kyc_package_hash: ContractPackageHash::new([0u8; 32]),
            is_english: true,
            starting_price: None,
            reserve_price: U512::from(1000),
            token_id: "token_id".to_string(),
            start_time: now + 500,
            cancellation_time: 3000,
            end_time: 3500,
            name: "test".to_string(),
            bidder_count_cap: None,
            auction_timer_extension: None,
        }
    }
}
