use casper_contract::{
    contract_api::{
        runtime::{self, get_call_stack, revert},
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
pub const FINALIZED: &str = "finalized";
pub const BID: &str = "bid";
pub const BID_PURSE: &str = "bid_purse";
pub const CANCEL_FUNC: &str = "cancel_bid";
pub const FINALIZE_FUNC: &str = "finalize";
pub const CANCEL_AUCTION_FUNC: &str = "cancel_auction";
pub const AUCTION_CONTRACT_HASH: &str = "auction_contract_package_hash";
pub const AUCTION_ACCESS_TOKEN: &str = "auction_access_token";
pub const EVENTS: &str = "auction_events";
pub const EVENTS_COUNT: &str = "auction_events_count";
pub const COMMISSIONS: &str = "commissions";
pub const KYC_HASH: &str = "kyc_package_hash";
pub const BIDDER_NUMBER_CAP: &str = "bidder_count_cap";
pub const AUCTION_TIMER_EXTENSION: &str = "auction_timer_extension";
pub const MINIMUM_BID_STEP: &str = "minimum_bid_step";
pub const MARKETPLACE_COMMISSION: &str = "marketplace_commission";
pub const MARKETPLACE_ACCOUNT: &str = "marketplace_account";
pub const AUCTION_COUNT: &str = "auction_count";
pub const AUCTION_ACTIVE_STATE: &str = "auction_active_state";

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

    pub fn get_auction_package_hash() -> ContractPackageHash {
        if let casper_types::system::CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } = get_call_stack().last().unwrap_or_revert()
        {
            *contract_package_hash
        } else {
            revert(ApiError::User(700));
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

    pub fn get_auction_count() -> u8 {
        read_named_key_value(AUCTION_COUNT)
    }

    pub fn get_auction_active_state() -> bool {
        read_named_key_value(AUCTION_ACTIVE_STATE)
    }

    pub fn set_auction_count(count: u8) {
        write_named_key_value(AUCTION_COUNT, count)
    }

    pub fn set_auction_active_state(state: bool) {
        write_named_key_value(AUCTION_ACTIVE_STATE, state)
    }

    pub fn get_commission_shares() -> BTreeMap<AccountHash, u16> {
        let commissions = Self::get_commissions();
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
                "account" => {
                    let rate = commissions
                        .get(&format!("{}_rate", actor))
                        .unwrap_or_revert();
                    let share_rate = string_to_u16(rate);
                    share_sum += share_rate;
                    converted_commissions.insert(string_to_account_hash(value), share_rate);
                }
                "rate" => {
                    let account = commissions
                        .get(&format!("{}_account", actor))
                        .unwrap_or_revert();
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
            write_named_key_value(END, AuctionData::get_end() + increment);
            write_named_key_value(CANCEL, AuctionData::get_cancel_time() + increment);
        }
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
    let finalized = false;
    let bidder_count_cap = runtime::get_named_arg::<Option<u64>>(BIDDER_NUMBER_CAP);
    // Get commissions from nft

    let commissions_ret: Option<BTreeMap<String, String>> = runtime::call_versioned_contract(
        ContractPackageHash::from(token_contract_hash),
        None,
        "token_commission",
        runtime_args! {
            "token_id" => token_id.clone(),
            "property" => "".to_string(),
        },
    );

    let commissions = match commissions_ret {
        Some(com) => com,
        None => BTreeMap::new(),
    };

    let auction_timer_extension = runtime::get_named_arg::<Option<u64>>(AUCTION_TIMER_EXTENSION);
    let minimum_bid_step = runtime::get_named_arg::<Option<U512>>(MINIMUM_BID_STEP);
    let marketplace_commission = runtime::get_named_arg::<u32>(MARKETPLACE_COMMISSION);
    let marketplace_account = runtime::get_named_arg::<AccountHash>(MARKETPLACE_ACCOUNT);

    let mut named_keys = named_keys!(
        (OWNER, token_owner),
        (BENEFICIARY_ACCOUNT, beneficiary_account),
        (NFT_HASH, Key::Hash(token_contract_hash)),
        (KYC_HASH, kyc_contract_hash),
        (ENGLISH_FORMAT, english_format),
        (TOKEN_ID, token_id),
        (START, start_time),
        (CANCEL, cancellation_time),
        (END, end_time),
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
        (MARKETPLACE_ACCOUNT, marketplace_account),
        (AUCTION_ACTIVE_STATE, true),
        (AUCTION_COUNT, 1_u8)
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
        Err(_e) => revert(AuctionError::CommissionAccountIncorrectSerialization),
    }
}

fn string_to_u16(ustr: &str) -> u16 {
    match ustr.parse::<u16>() {
        Ok(u) => u,
        Err(_e) => revert(AuctionError::CommissionRateIncorrectSerialization),
    }
}

pub fn push_auction_history() {
    let mut history_item: BTreeMap<String, String> = BTreeMap::new();
    history_item.insert(
        "nft_package_contract".to_string(),
        AuctionData::get_nft_hash().to_formatted_string(),
    );
    history_item.insert(
        "kyc_package_contract".to_string(),
        AuctionData::get_kyc_hash().to_formatted_string(),
    );
    history_item.insert("token_id".to_string(), AuctionData::get_token_id());
    history_item.insert(
        "winner".to_string(),
        match AuctionData::get_winner() {
            Some(winner) => winner.to_formatted_string(),
            None => "unclaimed".to_string(),
        },
    );
    history_item.insert(
        "winning_price".to_string(),
        match AuctionData::get_price() {
            Some(winning_price) => winning_price.to_string(),
            None => "unclaimed".to_string(),
        },
    );
    let (market_acc, market_share) = AuctionData::get_marketplace_data();
    history_item.insert(
        "marketplace_account".to_string(),
        market_acc.to_formatted_string(),
    );
    history_item.insert(
        "marketplace_commission".to_string(),
        market_share.to_string(),
    );
    runtime::put_key(
        &format!(
            "auction_{}_data",
            AuctionData::get_auction_count().to_string()
        ),
        storage::new_uref(history_item).into(),
    )
}

pub fn initialize_auction() {
    if AuctionData::get_auction_active_state() {
        revert(ApiError::User(800))
    }
    if AuctionData::get_token_owner() != Key::Account(runtime::get_caller()) {
        revert(ApiError::User(801))
    }
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
    let finalized = false;
    let bidder_count_cap = runtime::get_named_arg::<Option<u64>>(BIDDER_NUMBER_CAP);
    // Get commissions from nft

    let commissions_ret: Option<BTreeMap<String, String>> = runtime::call_versioned_contract(
        ContractPackageHash::from(token_contract_hash),
        None,
        "token_commission",
        runtime_args! {
            "token_id" => token_id.clone(),
            "property" => "".to_string(),
        },
    );

    let commissions = match commissions_ret {
        Some(com) => com,
        None => BTreeMap::new(),
    };

    let auction_timer_extension = runtime::get_named_arg::<Option<u64>>(AUCTION_TIMER_EXTENSION);
    let minimum_bid_step = runtime::get_named_arg::<Option<U512>>(MINIMUM_BID_STEP);
    let marketplace_commission = runtime::get_named_arg::<u32>(MARKETPLACE_COMMISSION);
    let marketplace_account = runtime::get_named_arg::<AccountHash>(MARKETPLACE_ACCOUNT);

    write_named_key_value(BENEFICIARY_ACCOUNT, beneficiary_account);
    write_named_key_value(NFT_HASH, Key::Hash(token_contract_hash));
    write_named_key_value(KYC_HASH, kyc_contract_hash);
    write_named_key_value(ENGLISH_FORMAT, english_format);
    write_named_key_value(TOKEN_ID, token_id);
    write_named_key_value(START, start_time);
    write_named_key_value(CANCEL, cancellation_time);
    write_named_key_value(END, end_time);
    write_named_key_value(START_PRICE, start_price);
    write_named_key_value(RESERVE, reserve_price);
    write_named_key_value(PRICE, winning_bid);
    write_named_key_value(WINNER, current_winner);
    write_named_key_value(FINALIZED, finalized);
    write_named_key_value(COMMISSIONS, commissions);
    write_named_key_value(BIDDER_NUMBER_CAP, bidder_count_cap);
    write_named_key_value(AUCTION_TIMER_EXTENSION, auction_timer_extension);
    write_named_key_value(MINIMUM_BID_STEP, minimum_bid_step);
    write_named_key_value(MARKETPLACE_COMMISSION, marketplace_commission);
    write_named_key_value(MARKETPLACE_ACCOUNT, marketplace_account);

    AuctionData::set_auction_active_state(true);
    AuctionData::set_auction_count(AuctionData::get_auction_count() + 1);
}
