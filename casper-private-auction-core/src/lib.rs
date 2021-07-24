#![no_std]

extern crate alloc;
use casper_types::{ApiError, contracts::NamedKeys, U256, U512, Key, ContractHash, CLType, CLValue};
use casper_contract::{unwrap_or_revert::UnwrapOrRevert, contract_api::{runtime, storage}};
use alloc::string::String;
use alloc::collections::BTreeMap;

const NFT_HASH_ARG: &str = "token_contract_hash";
const NFT_HASH: &str = NFT_HASH_ARG;
const FORMAT_ARG: &str = "format";
const ENGLISH_FORMAT: &str = "english_format";
const ENGLISH_MATCH: &str = "English";
const DUTCH_MATCH: &str = "Dutch";
const TOKEN_ID_ARG: &str = "token_id";
const TOKEN_ID: &str = TOKEN_ID_ARG;
const START_ARG: &str = "start_time";
const CANCEL_ARG: &str = "cancellation_time";
const END_ARG: &str = "end_time";
const START: &str = START_ARG;
const CANCEL: &str = CANCEL_ARG;
const END: &str = END_ARG;
const RESERVE_ARG: &str = "reserve_price";
const RESERVE: &str = RESERVE_ARG;
const START_PRICE_ARG: &str = "starting_price";
const START_PRICE: &str = START_PRICE_ARG;
const PRICE: &str = "current_price";
const BIDS: &str = "bids";

pub fn english_format_match() -> bool {
    match &runtime::get_named_arg::<String>(FORMAT_ARG)[..] {
        ENGLISH_MATCH => true,
        DUTCH_MATCH => false,
        _ => runtime::revert(ApiError::InvalidArgument),
    }
}

pub fn auction_times_match() -> (u64, u64, u64) {
    match (runtime::get_named_arg(START_ARG), runtime::get_named_arg(CANCEL_ARG), runtime::get_named_arg(END_ARG)) {
        (start, cancel, end) if start <= cancel && cancel <= end => (start, cancel, end),
        _ => runtime::revert(ApiError::InvalidArgument),
    }
}

pub fn create_auction_named_keys() -> NamedKeys {
    // Get the auction parameters from the command line args
    let token_contract_hash = ContractHash::new(runtime::get_named_arg::<Key>(NFT_HASH_ARG).into_hash().unwrap_or_revert());
    let english_format = english_format_match();
    // Consider optimizing away the storage of start price key for English auctions
    let start_price = match (english_format, runtime::get_named_arg::<Option<U512>>(START_PRICE_ARG)) {
        (false, Some(p)) => p,
        (true, None) => 0.into(),
        _ => runtime::revert(ApiError::InvalidArgument),
    };
    let token_id: U256 = runtime::get_named_arg(TOKEN_ID_ARG);
    let (start_time, cancellation_time, end_time): (u64, u64, u64) = auction_times_match();
    let reserve_price: U512 = runtime::get_named_arg(RESERVE_ARG);
    let current_price: U512 = reserve_price.clone();
    let bids: BTreeMap<Key, U512> = BTreeMap::new();

    // Create and return the named keys struct with parameters and the dictionary
    let mut named_keys = NamedKeys::new();

    let vars: [(&str, CLValue); 10] = [(NFT_HASH, token_contract_hash.into()), (ENGLISH_FORMAT, english_format.into()), (TOKEN_ID, token_id.into()), (START, start_time.into()), (CANCEL, cancellation_time.into()), (END, end_time.into()), (START_PRICE, start_price.into()), (RESERVE, reserve_price.into()), (PRICE, current_price.into()), (BIDS, bids.into())];
    for (name, value) in vars {
        let value_uref = storage::new_uref(value);
        named_keys.insert(name.into(), value_uref.into());
    }
    return named_keys
}
