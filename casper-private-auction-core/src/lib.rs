#![no_std]

extern crate alloc;
use casper_types::{ApiError, contracts::NamedKeys, U512, Key, ContractHash, URef, CLTyped, bytesrepr::FromBytes, runtime_args, RuntimeArgs, system::CallStackElement};
use casper_contract::{unwrap_or_revert::UnwrapOrRevert, contract_api::{runtime, storage, system}};
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use casper_types::bytesrepr::ToBytes;
use casper_types::account::AccountHash;

// TODO: Either separate arg name and named key consistently, or not at all
const OWNER: &str = "token_owner";
const BENEFICIARY_ACCOUNT: &str = "beneficiary_account";
pub const AUCTION_PURSE: &str = "auction_purse";
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
const PRICE: &str = "winning_bid";
const WINNER: &str = "current_winner";
const BIDS: &str = "bids";
const FINALIZED: &str = "finalized";
const ERROR_EARLY: u16 = 0;
const ERROR_INVALID_CALLER: u16 = 1;
const ERROR_LATE_BID: u16 = 2;
const ERROR_BID_TOO_LOW: u16 = 3;
const ERROR_ALREADY_FINAL: u16 = 4;
const ERROR_BAD_STATE: u16 = 5;
const ERROR_NO_BID: u16 = 6;
const ERROR_LATE_CANCELLATION: u16 = 7;
const ERROR_UNKNOWN_FORMAT: u16 = 8;
const ERROR_INVALID_TIMES: u16 = 9;
const ERROR_INVALID_PRICES: u16 = 10;
const ERROR_EARLY_BID: u16 = 11;
const ERROR_INVALID_BENEFICIARY: u16 = 12;
pub const BID: &str = "bid";
pub const BID_PURSE: &str = "bid_purse";
pub const BID_FUNC: &str = BID;
pub const CANCEL_FUNC: &str = "cancel_bid";
pub const FINALIZE_FUNC: &str = "finalize";
pub const AUCTION_CONTRACT_HASH: &str = "auction_contract_package_hash";
pub const AUCTION_ACCESS_TOKEN: &str = "auction_access_token";

macro_rules! named_keys {
    ( $( ($name:expr, $value:expr) ),* ) => {
        {
            let mut named_keys = NamedKeys::new();
            $( named_keys.insert($name.into(), storage::new_uref($value).into()); )*
            named_keys
        }
    };
}

// TODO: This needs A LOT of error handling because we don't want an auction being left in an unrecoverable state if the named keys are bad!
fn read_named_key_uref(name: &str) -> URef {
    let uref = runtime::get_key(name)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    return uref;
}

// TODO: This needs A LOT of error handling because we don't want an auction being left in an unrecoverable state if the named keys are bad!
fn read_named_key_value<T: CLTyped + FromBytes>(name: &str) -> T {
    let uref= read_named_key_uref(name);

    let value: T = storage::read(uref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    return value
}

fn write_named_key_value<T: CLTyped + ToBytes>(name: &str, value: T) -> () {
    let uref = read_named_key_uref(name);
    storage::write(uref, value);
}

// TODO: This whole thing should use enums
fn english_format_match() -> bool {
    match &runtime::get_named_arg::<String>(FORMAT_ARG)[..] {
        ENGLISH_MATCH => true,
        DUTCH_MATCH => false,
        _ => runtime::revert(ApiError::User(ERROR_UNKNOWN_FORMAT)),
    }
}

// TODO: Rewrite to avoid the match guard
fn auction_times_match() -> (u64, u64, u64) {
    match (runtime::get_named_arg(START_ARG), runtime::get_named_arg(CANCEL_ARG), runtime::get_named_arg(END_ARG)) {
        (start, cancel, end) if u64::from(runtime::get_blocktime()) <= start && start <= cancel && cancel <= end => (start, cancel, end),
        _ => runtime::revert(ApiError::User(ERROR_INVALID_TIMES)),
    }
}

pub fn create_auction_named_keys() -> NamedKeys {
    // Get the owner
    let token_owner = Key::Account(runtime::get_caller());
    // Get the beneficiary purse
    let beneficiary_account =
        match runtime::get_named_arg::<Key>(BENEFICIARY_ACCOUNT) {
            key @ Key::Account(_) => key,
            _ => runtime::revert(ApiError::User(ERROR_INVALID_BENEFICIARY)),
        };

    // Get the auction parameters from the command line args
    let token_contract_hash: Key = Key::Hash(runtime::get_named_arg::<Key>(NFT_HASH_ARG).into_hash().unwrap_or_revert());
    let english_format = english_format_match();
    // Consider optimizing away the storage of start price key for English auctions
    let (start_price, reserve_price) = match (english_format, runtime::get_named_arg::<Option<U512>>(START_PRICE_ARG), runtime::get_named_arg::<U512>(RESERVE_ARG)) {
        (false, Some(p), r) if p >= r => (Some(p), r),
        (true, None, r) => (None, r),
        _ => runtime::revert(ApiError::User(ERROR_INVALID_PRICES)),
    };
    let token_id = runtime::get_named_arg::<String>(TOKEN_ID_ARG);
    let (start_time, cancellation_time, end_time): (u64, u64, u64) = auction_times_match();
    let winning_bid: Option<U512> = None;
    let current_winner: Option<Key> = None;
    // Consider optimizing away for Dutch auctions
    let bids: BTreeMap<AccountHash, U512> = BTreeMap::new();
    let finalized = false;

    return named_keys!(
        (OWNER, token_owner),
        (BENEFICIARY_ACCOUNT, beneficiary_account),
        (NFT_HASH, token_contract_hash),
        (ENGLISH_FORMAT, english_format),
        (TOKEN_ID, token_id),
        (START, start_time),
        (CANCEL, cancellation_time),
        (END, end_time),
        (START_PRICE, start_price),
        (RESERVE, reserve_price),
        (PRICE, winning_bid),
        (WINNER, current_winner),
        (BIDS, bids),
        (FINALIZED, finalized))
}

// TODO: Consider refactoring and combining with named arg creation to avoid duplicating host side function calls
pub fn auction_receive_token(auction_key: Key) -> () {
    let token_owner = Key::Account(runtime::get_caller());
    let token_contract_hash = ContractHash::new(runtime::get_named_arg::<Key>(NFT_HASH_ARG).into_hash().unwrap_or_revert());
    let token_id_str = runtime::get_named_arg::<String>(TOKEN_ID_ARG);

    runtime::call_contract(
        token_contract_hash,
        "transfer_token",
        runtime_args! {
            "sender" => token_owner,
            "recipient" => auction_key,
            "token_id" => token_id_str,
          }
    )
}

fn auction_transfer_token(recipient: Key) -> () {
    let auction_key: Key = {
        let call_stack = runtime::get_call_stack();
        let caller: CallStackElement = call_stack.last().unwrap_or_revert().clone();
        let auction_contract_key = match caller {
            CallStackElement::StoredContract { contract_package_hash: _, contract_hash: contract_hash_addr_caller} =>
                Key::Hash(contract_hash_addr_caller.value()),
            _ => runtime::revert(ApiError::User(ERROR_INVALID_CALLER)),
        };
        auction_contract_key
    };

    let token_contract_hash = ContractHash::new(read_named_key_value::<Key>(NFT_HASH).into_hash().unwrap_or_revert());
    let token_id_str = read_named_key_value::<String>(TOKEN_ID);

    runtime::call_contract(
        token_contract_hash,
        "transfer_token",
        runtime_args! {
            "sender" => auction_key,
            "recipient" => recipient,
            "token_id" => token_id_str,
          }
    )
}

fn get_bidder() -> Key {
    // Figure out who is trying to bid and what their bid is
    let mut call_stack = runtime::get_call_stack();
    call_stack.pop();

    //if session { () } else { call_stack.pop(); () };

    let caller: CallStackElement = call_stack.last().unwrap_or_revert().clone();
    // TODO: Contracts should probably be disallowed, since they can't be verified by Civic in a meaningful way
    let bidder = match caller {
        CallStackElement::Session { account_hash: account_hash_caller} => Key::Account(account_hash_caller),
        CallStackElement::StoredContract { contract_package_hash: _, contract_hash: contract_hash_addr_caller} => Key::Hash(contract_hash_addr_caller.value()),
        _ => runtime::revert(ApiError::User(ERROR_INVALID_CALLER)),
    };

    return bidder;
}

fn reset_winner(winner: Option<AccountHash>, bid: Option<U512>) -> () {
    write_named_key_value(WINNER, winner);
    write_named_key_value(PRICE, bid);
}

fn find_new_winner() -> Option<(AccountHash, U512)> {
    let bids = read_named_key_value::<BTreeMap<AccountHash, U512>>(BIDS);
    let winning_pair = bids
        .iter()
        .max_by_key( |p| p.1 );
    match winning_pair {
        Some((key, bid)) => Some((key.clone(), bid.clone())),
        _ => None,
    }
}

fn get_current_price() -> U512 {
    let block_time = u64::from(runtime::get_blocktime());
    let start_price = match read_named_key_value::<Option<U512>>(START_PRICE) {
        Some(p) => p,
        None => runtime::revert(ApiError::User(ERROR_BAD_STATE)),
    };
    let end_price = read_named_key_value::<U512>(RESERVE);
    let start_time = read_named_key_value::<u64>(START);
    let end_time = read_named_key_value::<u64>(END);

    let duration = end_time - start_time;
    let time_diff = block_time - start_time;

    if time_diff == 0u64 {
        return start_price;
    } else {
        let time_ratio = duration/time_diff;
        let price_range = start_price - end_price;
        let price_delta = price_range/U512::from(time_ratio);
        return start_price - price_delta;
    };

}

pub fn auction_bid() -> () {
    fn add_bid(bidder: AccountHash, bidder_purse: URef, bid: U512) -> () {
        // Get the existing bid, if any
        let mut bids = read_named_key_value::<BTreeMap<AccountHash, U512>>(BIDS);
        match bids.get(&bidder) {
            Some(current_bid) =>
                if &bid <= current_bid {
                    runtime::revert(ApiError::User(ERROR_BID_TOO_LOW))
                } else {
                    let auction_purse = read_named_key_uref(AUCTION_PURSE);
                    system::transfer_from_purse_to_purse(bidder_purse, auction_purse, bid - current_bid, None);
                    bids.insert(bidder, bid);
                    write_named_key_value(BIDS, bids);
                },
            _ =>
                {
                    let auction_purse = read_named_key_uref(AUCTION_PURSE);
                    system::transfer_from_purse_to_purse(bidder_purse, auction_purse, bid, None);
                    bids.insert(bidder, bid);
                    write_named_key_value(BIDS, bids);
                },
        }
    }

    // Check that it's not too late and that the auction isn't finalized
    let finalized = read_named_key_value::<bool>(FINALIZED);
    let start_time = read_named_key_value::<u64>(START);
    let end_time = read_named_key_value::<u64>(END);
    let block_time = u64::from(runtime::get_blocktime());

    match (finalized, block_time <= end_time, block_time >= start_time) {
        // Auction ongoing
        (false, true, true) => (),
        // Auction ended or is finalized
        (false, false, true) | (true, _, _) => runtime::revert(ApiError::User(ERROR_LATE_BID)),
        // Auction didn't start yet
        (false, true, false) => runtime::revert(ApiError::User(ERROR_EARLY_BID)),
        // Should not be in a state where current time is both before start and after the end
        (_, false, false) => runtime::revert(ApiError::User(ERROR_BAD_STATE))
    }

    // Figure out who is trying to bid and what their bid is
    let bidder= get_bidder().into_account().unwrap_or_revert_with(ApiError::User(ERROR_INVALID_CALLER));
    let bid = runtime::get_named_arg::<U512>(BID);
    if bid < read_named_key_value::<U512>(RESERVE) {
        runtime::revert(ApiError::User(ERROR_BID_TOO_LOW));
    }
    let bidder_purse = runtime::get_named_arg::<URef>(BID_PURSE);

    // Adding the bid, doing the purse transfer and resetting the winner if necessary, as well as possibly ending a Dutch auction
    match (read_named_key_value::<bool>(ENGLISH_FORMAT), read_named_key_value::<Option<AccountHash>>(WINNER), read_named_key_value::<Option<U512>>(PRICE)) {
        (true, None, None) => {
            add_bid(bidder, bidder_purse, bid);
            reset_winner(Some(bidder), Some(bid));
        },
        (true, Some(_), Some(current_price)) =>
            if bid <= current_price {
                add_bid(bidder, bidder_purse, bid)
            } else {
                add_bid(bidder, bidder_purse, bid);
                reset_winner(Some(bidder), Some(bid))
            },
        (false, None, None) =>
            if bid >= get_current_price() {
                // TODO: Add the current price, not the bid, in the Dutch auction
                add_bid(bidder, bidder_purse, bid);
                reset_winner(Some(bidder), Some(bid));
                auction_finalize(false);
            }
            else {
                runtime::revert(ApiError::User(ERROR_BID_TOO_LOW));
            },
        _ => runtime::revert(ApiError::User(ERROR_BAD_STATE)),
    }

}

pub fn auction_cancel_bid() -> () {
    let bidder = get_bidder().into_account().unwrap_or_revert_with(ApiError::User(ERROR_INVALID_CALLER));
    let block_time = u64::from(runtime::get_blocktime());
    let cancellation_time = read_named_key_value::<u64>(CANCEL);

    if block_time < cancellation_time {
        let mut bids = read_named_key_value::<BTreeMap<AccountHash, U512>>(BIDS);
        match bids.get(&bidder) {
            Some(current_bid) =>
                {
                    let auction_purse = read_named_key_uref(AUCTION_PURSE);
                    system::transfer_from_purse_to_account(auction_purse, bidder.clone(), current_bid.clone(), None);
                    bids.remove(&bidder);
                    write_named_key_value(BIDS, bids);
                    match find_new_winner() {
                        Some((winner, bid)) => reset_winner(Some(winner), Some(bid)),
                        _ => reset_winner(None, None),
                    }
                    return ();
                },
            _ => runtime::revert(ApiError::User(ERROR_NO_BID)),
        }
    } else {
        runtime::revert(ApiError::User(ERROR_LATE_CANCELLATION))
    }
}

fn auction_allocate(winner: Option<AccountHash>) -> () {
    match winner {
        Some(acct) => auction_transfer_token(Key::Account(acct)),
        _ => auction_transfer_token(read_named_key_value::<Key>(OWNER)),
    }
}

fn auction_transfer(winner: Option<AccountHash>) -> () {
    fn return_bids(mut bids: BTreeMap<AccountHash, U512>, auction_purse: URef) -> () {
        for (bidder, bid) in &bids {
            system::transfer_from_purse_to_account(auction_purse, bidder.clone(), bid.clone(), None);
        }
        bids.clear();
        write_named_key_value(BIDS, bids);
    }
    match winner {
        Some(key) => {
            let auction_purse = read_named_key_uref(AUCTION_PURSE);
            let beneficiary_account = read_named_key_value::<Key>(BENEFICIARY_ACCOUNT).into_account().unwrap_or_revert();
            let mut bids = read_named_key_value::<BTreeMap<AccountHash, U512>>(BIDS);
            match bids.get(&key) {
                Some(bid) => {
                    system::transfer_from_purse_to_account(auction_purse, beneficiary_account, bid.clone(), None);
                    bids.remove(&key);
                    return_bids(bids, auction_purse);
                },
                // Something went wrong, so better return everyone's money
                _ => return_bids(bids, auction_purse),
            }
        },
        _ => {
            let auction_purse = read_named_key_uref(AUCTION_PURSE);
            let mut bids = read_named_key_value::<BTreeMap<AccountHash, U512>>(BIDS);
            return_bids(bids, auction_purse);
        },
    }
}

pub fn auction_finalize(time_check: bool) -> () {
    // Get finalization and check if we're done
    let finalized = read_named_key_value::<bool>(FINALIZED);
    if finalized {
        runtime::revert(ApiError::User(ERROR_ALREADY_FINAL))
    };

    // We're not finalized, so let's get all the other arguments, as well as time to make sure we're not too early
    let end_time = read_named_key_value::<u64>(END);
    let block_time = u64::from(runtime::get_blocktime());
    if time_check && block_time < end_time {
            runtime::revert(ApiError::User(ERROR_EARLY))
    }

    // TODO: Figure out how to gracefully finalize if the keys are bad
    match (finalized, read_named_key_value::<Option<U512>>(PRICE), read_named_key_value::<Option<AccountHash>>(WINNER)) {
        (false, Some(_), Some(winner)) => {
            auction_allocate(Some(winner));
            auction_transfer(Some(winner));
            write_named_key_value(FINALIZED, true);
        },
        _ => {
            auction_allocate(None);
            auction_transfer(None);
            write_named_key_value(FINALIZED, true);
        }
    }
}