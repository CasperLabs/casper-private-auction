use casper_engine_test_support::Hash;
use casper_engine_test_support::{Account, AccountHash};
use casper_types::{runtime_args, Key, PublicKey, RuntimeArgs, SecretKey, U512};

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

/*
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
*/

#[derive(Debug)]
pub struct AuctionArgsBuilder {
    // into Key
    beneficiary_account: AccountHash,
    // into Key
    token_contract_hash: Hash,
    // true is `ENGLISH` | false is `DUTCH`
    is_english: bool,
    // ENGLISH format cannot have a starting price, build turn it into option
    starting_price: U512,
    reserve_price: U512,
    token_id: String,
    start_time: u64,
    cancellation_time: u64,
    end_time: u64,
}

impl AuctionArgsBuilder {
    pub fn new_with_necessary(
        beneficiary: &AccountHash,
        nft: &Hash,
        token_id: &str,
        english: bool,
        start_time: u64,
    ) -> Self {
        AuctionArgsBuilder {
            beneficiary_account: *beneficiary,
            token_contract_hash: *nft,
            is_english: english,
            starting_price: U512::from(0),
            reserve_price: U512::from(1000),
            token_id: token_id.to_string(),
            start_time,
            cancellation_time: 3000,
            end_time: 3500,
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

    pub fn set_token_contract_hash(&mut self, token_contract_hash: &Hash) {
        self.token_contract_hash = *token_contract_hash;
    }

    pub fn set_starting_price(&mut self, starting_price: U512) {
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

    pub fn build(&self) -> RuntimeArgs {
        runtime_args! {
            "beneficiary_account"=>Key::Account(self.beneficiary_account),
            "token_contract_hash"=>Key::Hash(self.token_contract_hash),
            "format"=>if self.is_english{"ENGLISH"}else{"DUTCH"},
            "starting_price"=>if self.is_english{None}else{Some(self.starting_price)},
            "reserve_price"=>self.reserve_price,
            "token_id"=>self.token_id.to_owned(),
            "start_time" => self.start_time,
            "cancellation_time" => self.start_time+self.cancellation_time,
            "end_time" => self.start_time+self.end_time,
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
            token_contract_hash: [0u8; 32],
            is_english: true,
            starting_price: U512::from(0),
            reserve_price: U512::from(1000),
            token_id: "token_id".to_string(),
            start_time: now + 500,
            cancellation_time: 3000,
            end_time: 3500,
        }
    }
}
