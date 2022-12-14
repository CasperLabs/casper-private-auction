use casper_types::{
    account::AccountHash, runtime_args, ContractPackageHash, Key, PublicKey, RuntimeArgs, U512,
};

use super::{
    constants::{
        ARG_NAME, AUCTION_NAME, AUCTION_TIMER_EXTENSION, BENEFICIARY_ACCOUNT, BIDDER_COUNT_CAP,
        CANCELLATION_TIME, DUTCH, END_TIME, ENGLISH, FORMAT, HAS_ENHANCED_NFT,
        KEY_KYC_PACKAGE_HASH, MARKETPLACE_ACCOUNT, MARKETPLACE_COMMISSION, MINIMUM_BID_STEP,
        RESERVE_PRICE, STARTING_PRICE, START_TIME, TOKEN_CONTRACT_HASH, TOKEN_ID,
    },
    utils::get_privayte_keys,
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
    minimum_bid_step: Option<U512>,
    marketplace_account: AccountHash,
    marketplace_commission: u32,
    has_enhanced_nft: bool,
}

impl AuctionArgsBuilder {
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
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

    pub fn set_minimum_bid_step(&mut self, minimum_bid_step: Option<U512>) {
        self.minimum_bid_step = minimum_bid_step;
    }

    pub fn set_marketplace_commission(&mut self, marketplace_commission: u32) {
        self.marketplace_commission = marketplace_commission;
    }

    pub fn set_has_enhanced_nft(&mut self) {
        self.has_enhanced_nft = true;
    }

    pub fn has_enhanced_nft(&self) -> bool {
        self.has_enhanced_nft
    }

    pub fn build(&self) -> RuntimeArgs {
        runtime_args! {
            BENEFICIARY_ACCOUNT => Key::Account(self.beneficiary_account),
            TOKEN_CONTRACT_HASH => Key::Hash(self.token_contract_hash.value()),
            KEY_KYC_PACKAGE_HASH => Key::Hash(self.kyc_package_hash.value()),
            FORMAT => if self.is_english{ENGLISH}else{DUTCH},
            STARTING_PRICE => self.starting_price,
            RESERVE_PRICE => self.reserve_price,
            TOKEN_ID => self.token_id.to_owned(),
            START_TIME => self.start_time,
            CANCELLATION_TIME => self.start_time+self.cancellation_time,
            END_TIME => self.start_time+self.end_time,
            ARG_NAME => self.name.clone(),
            BIDDER_COUNT_CAP => self.bidder_count_cap,
            AUCTION_TIMER_EXTENSION => self.auction_timer_extension,
            MINIMUM_BID_STEP => self.minimum_bid_step,
            MARKETPLACE_ACCOUNT => self.marketplace_account,
            MARKETPLACE_COMMISSION => self.marketplace_commission,
            HAS_ENHANCED_NFT => self.has_enhanced_nft,
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
        let (admin_secret, _, _) = get_privayte_keys();
        let now: u64 = Self::get_now_u64();
        AuctionArgsBuilder {
            beneficiary_account: AccountHash::from(&PublicKey::from(&admin_secret)),
            token_contract_hash: ContractPackageHash::new([0u8; 32]),
            kyc_package_hash: ContractPackageHash::new([0u8; 32]),
            is_english: true,
            starting_price: None,
            reserve_price: U512::from(1000),
            token_id: TOKEN_ID.to_string(),
            start_time: now + 500,
            cancellation_time: 3000,
            end_time: 3500,
            name: AUCTION_NAME.to_string(),
            bidder_count_cap: None,
            auction_timer_extension: None,
            minimum_bid_step: None,
            marketplace_account: AccountHash::new([11_u8; 32]),
            marketplace_commission: 75,
            has_enhanced_nft: false,
        }
    }
}
