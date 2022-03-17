use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage::{self},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    runtime_args, ApiError, CLTyped, ContractPackageHash, RuntimeArgs, URef,
};

use crate::error::AuctionError;
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
pub const TOKEN_ID: &str = "token_id";
pub const START: &str = "start_time";
pub const END: &str = "end_time";
pub const RESERVE: &str = "reserve_price";
pub const STARTING_PRICE: &str = "starting_price";
pub const PRICE: &str = "winning_bid";
pub const WINNER: &str = "current_winner";
pub const FINALIZED: &str = "finalized";
pub const BID: &str = "bid";
pub const BID_PURSE: &str = "bid_purse";
pub const CANCEL_AUCTION_FUNC: &str = "cancel_auction";
pub const AUCTION_CONTRACT_HASH: &str = "auction_contract_package_hash";
pub const AUCTION_ACCESS_TOKEN: &str = "auction_access_token";
pub const COMMISSIONS: &str = "commissions";
pub const KYC_HASH: &str = "kyc_package_hash";
pub const ACCOUNT_HASH_PREFIX: &str = "account-hash-";
pub const ACCOUNT: &str = "account";
pub const RATE: &str = "rate";

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
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub fn write_named_key_value<T: CLTyped + ToBytes>(name: &str, value: T) {
    let uref = read_named_key_uref(name);
    storage::write(uref, value);
}
pub struct AuctionData;

impl AuctionData {
    pub fn get_current_price() -> U512 {
        let block_time = u64::from(runtime::get_blocktime());
        let start_price = read_named_key_value::<U512>(STARTING_PRICE);
        let end_price = read_named_key_value::<U512>(RESERVE);
        let start_time = read_named_key_value::<u64>(START);
        let end_time = read_named_key_value::<u64>(END);

        let price_range = start_price - end_price;
        let duration = end_time - start_time;

        let step = price_range / duration;
        let time_passed = block_time - start_time;
        start_price - (step * time_passed)
    }

    pub fn is_auction_live() -> bool {
        // Check that it's not too late and that the auction isn't finalized
        let start_time = read_named_key_value::<u64>(START);
        let end_time = read_named_key_value::<u64>(END);
        let block_time = u64::from(runtime::get_blocktime());
        if block_time < start_time {
            runtime::revert(AuctionError::EarlyBid)
        }
        if block_time >= end_time {
            runtime::revert(AuctionError::LateBid)
        }
        block_time < end_time && block_time >= start_time
    }

    pub fn get_commission_shares() -> BTreeMap<AccountHash, u16> {
        let commissions: BTreeMap<String, String> = read_named_key_value(COMMISSIONS);
        let mut converted_commissions: BTreeMap<AccountHash, u16> = BTreeMap::new();
        let mut done: BTreeSet<String> = BTreeSet::new();
        let mut share_sum = 0;
        for (key, value) in &commissions {
            let mut split = key.split('_');
            let actor = split.next().unwrap_or_revert();
            if done.contains(actor) {
                continue;
            }
            let property = split.next().unwrap_or_revert();
            match property {
                ACCOUNT => {
                    let rate = commissions
                        .get(&format!("{}_{}", actor, RATE))
                        .unwrap_or_revert();
                    let share_rate = match rate.parse::<u16>() {
                        Ok(u) => u,
                        Err(_e) => revert(AuctionError::CommissionRateIncorrectSerialization),
                    };
                    share_sum += share_rate;
                    converted_commissions.insert(string_to_account_hash(value), share_rate);
                }
                RATE => {
                    let account = commissions
                        .get(&format!("{}_{}", actor, ACCOUNT))
                        .unwrap_or_revert();
                    let share_rate = match value.parse::<u16>() {
                        Ok(u) => u,
                        Err(_e) => revert(AuctionError::CommissionRateIncorrectSerialization),
                    };
                    share_sum += share_rate;
                    converted_commissions.insert(string_to_account_hash(account), share_rate);
                }
                _ => revert(AuctionError::InvalidcommissionProperty),
            }
            done.insert(actor.to_string());
        }
        if share_sum > 1000 {
            revert(AuctionError::CommissionTooManyShares)
        }
        converted_commissions
    }

    pub fn is_kyc_proved() -> bool {
        runtime::call_versioned_contract::<bool>(
            read_named_key_value(KYC_HASH),
            None,
            "is_kyc_proved",
            runtime_args! {
                ACCOUNT => Key::Account(runtime::get_caller()),
                "index" => Option::<casper_types::U256>::None
            },
        )
    }
}

// TODO: Rewrite to avoid the match guard
fn auction_times_match() -> (u64, u64) {
    let start: u64 = runtime::get_named_arg(START);
    let end: u64 = runtime::get_named_arg(END);
    if u64::from(runtime::get_blocktime()) <= start && start < end {
        return (start, end);
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

    let starting_price = runtime::get_named_arg::<U512>(STARTING_PRICE);
    let reserve_price = runtime::get_named_arg::<U512>(RESERVE);

    if starting_price < reserve_price {
        runtime::revert(AuctionError::InvalidPrices);
    }

    let token_id = runtime::get_named_arg::<String>(TOKEN_ID);
    let (start_time, end_time): (u64, u64) = auction_times_match();
    let winning_bid: Option<U512> = None;
    let current_winner: Option<Key> = None;
    let finalized = false;
    // Get commissions from nft

    let commissions_ret: Option<BTreeMap<String, String>> = runtime::call_versioned_contract(
        ContractPackageHash::from(token_contract_hash),
        None,
        "token_commission",
        runtime_args! {
            TOKEN_ID => token_id.clone(),
            "property" => "".to_string(),
        },
    );

    let commissions = match commissions_ret {
        Some(com) => com,
        None => BTreeMap::new(),
    };
    named_keys!(
        (OWNER, token_owner),
        (BENEFICIARY_ACCOUNT, beneficiary_account),
        (NFT_HASH, Key::Hash(token_contract_hash)),
        (KYC_HASH, kyc_contract_hash),
        (TOKEN_ID, token_id),
        (START, start_time),
        (END, end_time),
        (STARTING_PRICE, starting_price),
        (RESERVE, reserve_price),
        (PRICE, winning_bid),
        (WINNER, current_winner),
        (FINALIZED, finalized),
        (COMMISSIONS, commissions)
    )
}

fn string_to_account_hash(account_string: &str) -> AccountHash {
    let account = if account_string.starts_with(ACCOUNT_HASH_PREFIX) {
        AccountHash::from_formatted_str(account_string)
    } else {
        AccountHash::from_formatted_str(&format!("{}{}", ACCOUNT_HASH_PREFIX, account_string))
    };
    match account {
        Ok(acc) => acc,
        Err(_e) => revert(AuctionError::CommissionAccountIncorrectSerialization),
    }
}
