use casper_contract::{
    contract_api::{
        runtime::{self, revert},
        storage::{self, new_dictionary},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    runtime_args, CLTyped, ContractPackageHash, RuntimeArgs, URef,
};

use crate::{
    bids::Bids,
    constants::{
        AUCTION_CONTRACT_HASH, AUCTION_PACKAGE_HASH, AUCTION_PURSE, AUCTION_TIMER_EXTENSION,
        BENEFICIARY_ACCOUNT, BIDDER_NUMBER_CAP, CANCEL, COMMISSIONS, END_TIME, ENGLISH_FORMAT,
        EVENTS, EVENTS_COUNT, FINALIZED, FORMAT, HAS_ENHANCED_NFT, KYC_HASH, MARKETPLACE_ACCOUNT,
        MARKETPLACE_COMMISSION, MINIMUM_BID_STEP, NFT_HASH, OWNER, PRICE, RESERVE, START_PRICE,
        START_TIME, TOKEN_ID, WINNER,
    },
    error::AuctionError,
    events::{emit, AuctionEvent},
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
};
use casper_types::{account::AccountHash, contracts::NamedKeys, Key, U512};

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
        .unwrap_or_revert_with(AuctionError::CannotReadKey)
        .into_uref()
        .unwrap_or_revert_with(AuctionError::KeyNotUref)
}

// TODO: This needs A LOT of error handling because we don't want an auction being left in an unrecoverable state if the named keys are bad!
pub fn read_named_key_value<T: CLTyped + FromBytes>(name: &str) -> T {
    let uref = read_named_key_uref(name);

    storage::read(uref)
        .unwrap_or_revert_with(AuctionError::CannotReadKey)
        .unwrap_or_revert_with(AuctionError::NamedKeyNotFound)
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
                .unwrap_or_revert_with(AuctionError::KeyNotHash),
        )
    }

    pub fn get_has_enhanced_nft() -> bool {
        read_named_key_value::<bool>(HAS_ENHANCED_NFT)
    }

    pub fn get_token_id() -> String {
        read_named_key_value::<String>(TOKEN_ID)
    }

    pub fn set_winner(winner: Option<AccountHash>, bid: Option<U512>) {
        write_named_key_value(WINNER, winner);
        write_named_key_value(PRICE, bid);
        emit(&AuctionEvent::SetWinner {
            bidder: winner,
            bid,
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

        let price_range = start_price - end_price;
        let duration = end_time - start_time;

        let step = price_range / duration;
        let time_passed = block_time - start_time;
        start_price - (step * time_passed)
    }

    pub fn get_reserve() -> U512 {
        read_named_key_value::<U512>(RESERVE)
    }

    pub fn get_start() -> u64 {
        read_named_key_value::<u64>(START_TIME)
    }

    pub fn get_end() -> u64 {
        read_named_key_value::<u64>(END_TIME)
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
            .unwrap_or_revert_with(AuctionError::KeyNotAccount)
    }

    pub fn get_price() -> Option<U512> {
        read_named_key_value::<Option<U512>>(PRICE)
    }

    pub fn get_token_contract_hash() -> Option<Key> {
        read_named_key_value::<Option<Key>>(NFT_HASH)
    }

    pub fn get_auction_contract_hash() -> Key {
        runtime::get_key(AUCTION_CONTRACT_HASH)
            .unwrap_or_revert_with(AuctionError::AuctionContractNotFound)
    }

    pub fn get_auction_contract_package_hash() -> Key {
        runtime::get_key(AUCTION_PACKAGE_HASH)
            .unwrap_or_revert_with(AuctionError::AuctionContractNotFound)
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

    pub fn set_commissions(commissions: BTreeMap<String, String>) {
        write_named_key_value(COMMISSIONS, commissions);
    }

    pub fn get_commissions() -> BTreeMap<String, String> {
        read_named_key_value(COMMISSIONS)
    }

    pub fn get_bidder_count_cap() -> Option<u64> {
        read_named_key_value(BIDDER_NUMBER_CAP)
    }

    pub fn get_minimum_bid_step() -> Option<U512> {
        read_named_key_value(MINIMUM_BID_STEP)
    }

    pub fn get_marketplace_data() -> (AccountHash, u32) {
        (
            read_named_key_value(MARKETPLACE_ACCOUNT),
            read_named_key_value(MARKETPLACE_COMMISSION),
        )
    }

    pub fn get_commission_shares() -> BTreeMap<AccountHash, u16> {
        let commissions = Self::get_commissions();
        let mut converted_commissions: BTreeMap<AccountHash, u16> = BTreeMap::new();
        let mut done: BTreeSet<String> = BTreeSet::new();
        let mut share_sum = 0;
        for (key, value) in &commissions {
            let mut split = key.split('_');
            let actor = split
                .next()
                .unwrap_or_revert_with(AuctionError::CommissionActorSplit);
            if done.contains(actor) {
                continue;
            }
            let property = split
                .next()
                .unwrap_or_revert_with(AuctionError::CommissionPropertySplit);
            match property {
                "account" => {
                    let rate = commissions
                        .get(&format!("{actor}_rate"))
                        .unwrap_or_revert_with(AuctionError::MismatchedCommissionAccount);
                    let share_rate = string_to_u16(rate);
                    share_sum += share_rate;
                    converted_commissions.insert(string_to_account_hash(value), share_rate);
                }
                "rate" => {
                    let account = commissions
                        .get(&format!("{actor}_account"))
                        .unwrap_or_revert_with(AuctionError::MismatchedCommissionRate);
                    let share_rate = string_to_u16(value);
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

    pub fn get_kyc_hash() -> ContractPackageHash {
        read_named_key_value(KYC_HASH)
    }

    pub fn is_kyc_proved() -> bool {
        runtime::call_versioned_contract::<bool>(
            Self::get_kyc_hash(),
            None,
            "is_kyc_proved",
            runtime_args! {
                "account" => Key::Account(runtime::get_caller()),
                "index" => Option::<casper_types::U256>::None
            },
        )
    }

    pub fn increase_auction_times() {
        if let Some(increment) = read_named_key_value::<Option<u64>>(AUCTION_TIMER_EXTENSION) {
            write_named_key_value(END_TIME, AuctionData::get_end() + increment);
            write_named_key_value(CANCEL, AuctionData::get_cancel_time() + increment);
        }
    }
}

// TODO: Rewrite to avoid the match guard
fn auction_times_match() -> (u64, u64, u64) {
    let start: u64 = runtime::get_named_arg(START_TIME);
    let cancel: u64 = runtime::get_named_arg(CANCEL);
    let end: u64 = runtime::get_named_arg(END_TIME);
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
    let english_format = match runtime::get_named_arg::<String>(FORMAT).as_str() {
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
    let finalized = false;
    let bidder_count_cap = runtime::get_named_arg::<Option<u64>>(BIDDER_NUMBER_CAP);

    // Get commissions from CEP-47+ nft
    // Exclude CEP-78 without commissions entrypoint for now
    let has_enhanced_nft = runtime::get_named_arg::<bool>(HAS_ENHANCED_NFT);
    let commissions_ret: Option<BTreeMap<String, String>> = if !has_enhanced_nft {
        runtime::call_versioned_contract(
            ContractPackageHash::from(token_contract_hash),
            None,
            "token_commission",
            runtime_args! {
                TOKEN_ID => token_id.clone(),
                "property" => "".to_string(),
            },
        )
    } else {
        None
    };

    let commissions = match commissions_ret {
        Some(com) => com,
        None => BTreeMap::new(),
    };

    let auction_timer_extension = runtime::get_named_arg::<Option<u64>>(AUCTION_TIMER_EXTENSION);
    let minimum_bid_step = runtime::get_named_arg::<Option<U512>>(MINIMUM_BID_STEP);
    let marketplace_account = runtime::get_named_arg::<AccountHash>(MARKETPLACE_ACCOUNT);
    let marketplace_commission = runtime::get_named_arg::<u32>(MARKETPLACE_COMMISSION);

    let mut named_keys = named_keys!(
        (OWNER, token_owner),
        (BENEFICIARY_ACCOUNT, beneficiary_account),
        (NFT_HASH, Key::Hash(token_contract_hash)),
        (KYC_HASH, kyc_contract_hash),
        (ENGLISH_FORMAT, english_format),
        (HAS_ENHANCED_NFT, has_enhanced_nft),
        (TOKEN_ID, token_id),
        (START_TIME, start_time),
        (CANCEL, cancellation_time),
        (END_TIME, end_time),
        (START_PRICE, start_price),
        (RESERVE, reserve_price),
        (PRICE, winning_bid),
        (WINNER, current_winner),
        (FINALIZED, finalized),
        (EVENTS_COUNT, 0_u32),
        (COMMISSIONS, commissions),
        (BIDDER_NUMBER_CAP, bidder_count_cap),
        (AUCTION_TIMER_EXTENSION, auction_timer_extension),
        (MINIMUM_BID_STEP, minimum_bid_step),
        (MARKETPLACE_COMMISSION, marketplace_commission),
        (MARKETPLACE_ACCOUNT, marketplace_account)
    );
    add_empty_dict(&mut named_keys, EVENTS);
    named_keys
}

fn add_empty_dict(named_keys: &mut NamedKeys, name: &str) {
    if runtime::get_key(name).is_some() {
        runtime::remove_key(name);
    }
    let dict = new_dictionary(name).unwrap_or_revert_with(AuctionError::CannotCreateDictionary);
    runtime::remove_key(name);
    named_keys.insert(name.to_string(), dict.into());
}

fn string_to_account_hash(account_string: &str) -> AccountHash {
    let account = if account_string.starts_with("account-hash-") {
        AccountHash::from_formatted_str(account_string)
    } else if account_string.starts_with("Key::Account(") {
        AccountHash::from_formatted_str(
            account_string
                .replace("Key::Account(", "account-hash-")
                .strip_suffix(')')
                .unwrap_or_revert(),
        )
    } else {
        AccountHash::from_formatted_str(&format!("account-hash-{account_string}"))
    };
    match account {
        Ok(acc) => acc,
        Err(_e) => revert(AuctionError::CommissionAccountIncorrectSerialization),
    }
}

fn string_to_u16(ustr: &str) -> u16 {
    match ustr.parse::<u16>() {
        Ok(u) => u,
        Err(_e) => revert(AuctionError::CommissionRateIncorrectSerialization),
    }
}
