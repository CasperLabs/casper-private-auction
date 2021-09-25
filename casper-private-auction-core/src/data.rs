use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage::{self, new_dictionary},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    runtime_args, ApiError, CLTyped, ContractPackageHash, RuntimeArgs, URef,
};

use crate::{
    bids::Bids,
    error::AuctionError,
    events::{emit, AuctionEvent},
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
};
use casper_types::{account::AccountHash, contracts::NamedKeys, Key, U512};

// TODO: Either separate arg name and named key consistently, or not at all
pub const OWNER: &str = "token_owner";
pub const BENEFICIARY_ACCOUNT: &str = "beneficiary_account";
pub const AUCTION_PURSE: &str = "auction_purse";
pub const NFT_HASH: &str = "token_contract_hash";
pub const ENGLISH_FORMAT: &str = "english_format";
pub const TOKEN_ID: &str = "token_id";
pub const START: &str = "start_time";
pub const CANCEL: &str = "cancellation_time";
pub const END: &str = "end_time";
pub const RESERVE: &str = "reserve_price";
pub const START_PRICE: &str = "starting_price";
pub const PRICE: &str = "winning_bid";
pub const WINNER: &str = "current_winner";
pub const BIDS: &str = "bids";
pub const FINALIZED: &str = "finalized";
pub const BID: &str = "bid";
pub const BID_PURSE: &str = "bid_purse";
pub const CANCEL_FUNC: &str = "cancel_bid";
pub const FINALIZE_FUNC: &str = "finalize";
pub const AUCTION_CONTRACT_HASH: &str = "auction_contract_package_hash";
pub const AUCTION_ACCESS_TOKEN: &str = "auction_access_token";
pub const EVENTS: &str = "auction_events";
pub const EVENTS_COUNT: &str = "auction_events_count";
pub const COMISSIONS: &str = "comissions";
pub const KYC_HASH: &str = "kyc_package_hash";

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
pub fn read_named_key_uref(name: &str) -> URef {
    runtime::get_key(name)
        .unwrap_or_revert_with(ApiError::User(100))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(101))
}

// TODO: This needs A LOT of error handling because we don't want an auction being left in an unrecoverable state if the named keys are bad!
pub fn read_named_key_value<T: CLTyped + FromBytes>(name: &str) -> T {
    let uref = read_named_key_uref(name);

    storage::read(uref)
        .unwrap_or_revert_with(ApiError::User(102))
        .unwrap_or_revert_with(ApiError::User(103))
}

pub fn write_named_key_value<T: CLTyped + ToBytes>(name: &str, value: T) {
    let uref = read_named_key_uref(name);
    storage::write(uref, value);
}
pub struct AuctionData;

impl AuctionData {
    pub fn get_token_owner() -> Key {
        read_named_key_value::<Key>(OWNER)
    }
    pub fn get_nft_hash() -> ContractPackageHash {
        ContractPackageHash::new(
            read_named_key_value::<Key>(NFT_HASH)
                .into_hash()
                .unwrap_or_revert(),
        )
    }
    pub fn get_token_id() -> String {
        read_named_key_value::<String>(TOKEN_ID)
    }

    pub fn set_winner(winner: Option<AccountHash>, bid: Option<U512>) {
        write_named_key_value(WINNER, winner);
        write_named_key_value(PRICE, bid);
        emit(&AuctionEvent::SetWinner {
            bidder: winner.unwrap_or_revert(),
            bid: bid.unwrap_or_revert(),
        })
    }

    pub fn is_english_format() -> bool {
        read_named_key_value::<bool>(ENGLISH_FORMAT)
    }

    pub fn get_bids() -> Bids {
        Bids::at()
    }

    pub fn is_finalized() -> bool {
        read_named_key_value::<bool>(FINALIZED)
    }

    pub fn set_finalized() {
        write_named_key_value(FINALIZED, true);
    }

    pub fn get_winner() -> Option<AccountHash> {
        read_named_key_value::<Option<AccountHash>>(WINNER)
    }

    pub fn get_start_price() -> Option<U512> {
        read_named_key_value::<Option<U512>>(START_PRICE)
    }

    pub fn get_current_price() -> U512 {
        let block_time = u64::from(runtime::get_blocktime());
        let start_price = match Self::get_start_price() {
            Some(p) => p,
            None => runtime::revert(AuctionError::BadState),
        };
        let end_price = AuctionData::get_reserve();
        let start_time = AuctionData::get_start();
        let end_time = AuctionData::get_end();

        let duration = end_time - start_time;
        let time_diff = block_time - start_time;

        if time_diff == 0u64 {
            start_price
        } else {
            let time_ratio = duration / time_diff;
            let price_range = start_price - end_price;
            let price_delta = price_range / U512::from(time_ratio);
            start_price - price_delta
        }
    }

    pub fn get_reserve() -> U512 {
        read_named_key_value::<U512>(RESERVE)
    }

    pub fn get_start() -> u64 {
        read_named_key_value::<u64>(START)
    }

    pub fn get_end() -> u64 {
        read_named_key_value::<u64>(END)
    }

    pub fn get_cancel_time() -> u64 {
        read_named_key_value::<u64>(CANCEL)
    }

    pub fn get_auction_purse() -> URef {
        read_named_key_uref(AUCTION_PURSE)
    }

    pub fn get_beneficiary() -> AccountHash {
        read_named_key_value::<Key>(BENEFICIARY_ACCOUNT)
            .into_account()
            .unwrap_or_revert()
    }

    pub fn get_price() -> Option<U512> {
        read_named_key_value::<Option<U512>>(PRICE)
    }

    pub fn get_token_contract_hash() -> Option<Key> {
        read_named_key_value::<Option<Key>>(NFT_HASH)
    }

    pub fn is_auction_live() -> bool {
        // Check that it's not too late and that the auction isn't finalized
        let start_time = AuctionData::get_start();
        let end_time = AuctionData::get_end();
        let block_time = u64::from(runtime::get_blocktime());
        if block_time < start_time {
            runtime::revert(AuctionError::EarlyBid)
        }
        if block_time >= end_time {
            runtime::revert(AuctionError::LateBid)
        }
        block_time < end_time && block_time >= start_time
    }

    // pub fn fetch_comissions() -> BTreeMap<String, String> {
    //     if let Some(Key::Hash(contract_package_hash)) = Self::get_token_contract_hash() {
    //         let token_id = Self::get_token_id();
    //         runtime::call_versioned_contract(
    //             ContractPackageHash::from(contract_package_hash),
    //             None,
    //             "token_commission",
    //             runtime_args! {"token_id" => token_id},
    //         )
    //     }
    //     revert(AuctionError::BadState)
    // }

    pub fn set_comissions(comissions: BTreeMap<String, String>) {
        write_named_key_value(COMISSIONS, comissions);
    }

    pub fn get_comissions() -> BTreeMap<String, String> {
        read_named_key_value(COMISSIONS)
    }

    pub fn get_comission_shares() -> BTreeMap<AccountHash, u16> {
        let comissions = Self::get_comissions();
        let mut converted_comissions: BTreeMap<AccountHash, u16> = BTreeMap::new();
        let mut done: BTreeSet<String> = BTreeSet::new();
        let mut share_sum = 0;
        for (key, value) in &comissions {
            let mut split = key.split('_');
            let actor = split.next().unwrap_or_revert();
            if done.contains(actor) {
                continue;
            }
            let proterty = split.next().unwrap_or_revert();
            match proterty {
                "account" => {
                    let rate = comissions
                        .get(&format!("{}_rate", actor))
                        .unwrap_or_revert();
                    let share_rate = string_to_u16(rate);
                    share_sum += share_rate;
                    converted_comissions.insert(string_to_account_hash(value), share_rate);
                }
                "rate" => {
                    let account = comissions
                        .get(&format!("{}_account", actor))
                        .unwrap_or_revert();
                    let share_rate = string_to_u16(value);
                    share_sum += share_rate;
                    converted_comissions.insert(string_to_account_hash(account), share_rate);
                }
                _ => revert(AuctionError::InvalidComissionProperty),
            }
            done.insert(actor.to_string());
        }
        if share_sum > 1000 {
            revert(AuctionError::ComissionTooManyShares)
        }
        converted_comissions
    }

    pub fn get_kyc_hash() -> ContractPackageHash {
        read_named_key_value(KYC_HASH)
    }

    pub fn is_kyc_proved() -> bool {
        /*
        fn is_kyc_proved() {
        let account = runtime::get_named_arg::<Key>("account");
        let index = runtime::get_named_arg::<Option<U256>>("index");
        */
        runtime::get_caller();
        runtime::call_versioned_contract::<bool>(
            Self::get_kyc_hash(),
            None,
            "is_kyc_proved",
            runtime_args! {
                "account" => Key::Account(runtime::get_caller()),
                "index" => Option::<casper_types::U256>::None
            },
        );
        true
    }
}

// TODO: Rewrite to avoid the match guard
fn auction_times_match() -> (u64, u64, u64) {
    let start: u64 = runtime::get_named_arg(START);
    let cancel: u64 = runtime::get_named_arg(CANCEL);
    let end: u64 = runtime::get_named_arg(END);
    if u64::from(runtime::get_blocktime()) <= start
        && start <= cancel
        && cancel <= end
        && start < end
    {
        return (start, cancel, end);
    }
    runtime::revert(AuctionError::InvalidTimes)
}

pub fn create_auction_named_keys() -> NamedKeys {
    // Get the owner
    let token_owner = Key::Account(runtime::get_caller());
    // Get the beneficiary purse
    let beneficiary_account = match runtime::get_named_arg::<Key>(BENEFICIARY_ACCOUNT) {
        key @ Key::Account(_) => key,
        _ => runtime::revert(AuctionError::InvalidBeneficiary),
    };

    // Get the auction parameters from the command line args
    let token_contract_hash: [u8; 32] = runtime::get_named_arg::<Key>(NFT_HASH)
        .into_hash()
        .unwrap_or_default();
    let kyc_contract_hash: [u8; 32] = runtime::get_named_arg::<Key>(KYC_HASH)
        .into_hash()
        .unwrap_or_default();
    let english_format = match runtime::get_named_arg::<String>("format").as_str() {
        "ENGLISH" => true,
        "DUTCH" => false,
        _ => revert(AuctionError::UnknownFormat),
    };
    // Consider optimizing away the storage of start price key for English auctions
    let (start_price, reserve_price) = match (
        english_format,
        runtime::get_named_arg::<Option<U512>>(START_PRICE),
        runtime::get_named_arg::<U512>(RESERVE),
    ) {
        (false, Some(starting_price), reserver_price) if starting_price >= reserver_price => {
            (Some(starting_price), reserver_price)
        }
        (true, None, reserver_price) => (None, reserver_price),
        _ => runtime::revert(AuctionError::InvalidPrices),
    };

    let token_id = runtime::get_named_arg::<String>(TOKEN_ID);
    let (start_time, cancellation_time, end_time): (u64, u64, u64) = auction_times_match();
    let winning_bid: Option<U512> = None;
    let current_winner: Option<Key> = None;
    // Consider optimizing away for Dutch auctions
    let bids: BTreeMap<AccountHash, U512> = BTreeMap::new();
    let finalized = false;

    // Get comissions from nft

    let comissions_ret: Option<BTreeMap<String, String>> = runtime::call_versioned_contract(
        ContractPackageHash::from(token_contract_hash),
        None,
        "token_commission",
        runtime_args! {
            "token_id" => token_id.clone(),
            "property" => "".to_string(),
        },
    );

    let comissions = match comissions_ret {
        Some(com) => com,
        None => BTreeMap::new(),
    };

    let mut named_keys = named_keys!(
        (OWNER, token_owner),
        (BENEFICIARY_ACCOUNT, beneficiary_account),
        (NFT_HASH, Key::Hash(token_contract_hash)),
        (KYC_HASH, Key::Hash(kyc_contract_hash)),
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
        (FINALIZED, finalized),
        (EVENTS_COUNT, 0_u32),
        (COMISSIONS, comissions)
    );
    add_empty_dict(&mut named_keys, EVENTS);
    named_keys
}

fn add_empty_dict(named_keys: &mut NamedKeys, name: &str) {
    if runtime::get_key(name).is_some() {
        runtime::remove_key(name);
    }
    let dict = new_dictionary(name).unwrap_or_revert();
    runtime::remove_key(name);
    named_keys.insert(name.to_string(), dict.into());
}

fn string_to_account_hash(account_string: &str) -> AccountHash {
    let account = if account_string.starts_with("account-hash-") {
        AccountHash::from_formatted_str(account_string)
    } else {
        AccountHash::from_formatted_str(&format!("account-hash-{}", account_string))
    };
    match account {
        Ok(acc) => acc,
        Err(_e) => revert(AuctionError::ComissionAccountIncorrectSerialization),
    }
}

fn string_to_u16(ustr: &str) -> u16 {
    match ustr.parse::<u16>() {
        Ok(u) => u,
        Err(_e) => revert(AuctionError::ComissionRateIncorrectSerialization),
    }
}
