use alloc::collections::BTreeMap;
use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::account::AccountHash;
pub use casper_types::bytesrepr::ToBytes;
pub use casper_types::{
    bytesrepr::FromBytes, contracts::NamedKeys, runtime_args, system::CallStackElement, ApiError,
    CLTyped, ContractHash, Key, RuntimeArgs, URef, U512,
};

use crate::data::AuctionData;
use crate::error::AuctionError;

pub struct Auction;

impl Auction {
    fn add_bid(bidder: AccountHash, bidder_purse: URef, bid: U512) {
        if !AuctionData::is_auction_live() || AuctionData::is_finalized() {
            runtime::revert(AuctionError::BadState)
        }
        // Get the existing bid, if any
        let mut bids = AuctionData::get_bids();
        let auction_purse = AuctionData::get_auction_purse();
        match bids.get(&bidder) {
            Some(current_bid) => {
                if &bid <= current_bid {
                    runtime::revert(AuctionError::BidTooLow)
                } else {
                    system::transfer_from_purse_to_purse(
                        bidder_purse,
                        auction_purse,
                        bid - current_bid,
                        None,
                    )
                    .unwrap_or_revert();
                    bids.insert(bidder, bid);
                    AuctionData::update_bids(bids);
                }
            }
            _ => {
                system::transfer_from_purse_to_purse(bidder_purse, auction_purse, bid, None)
                    .unwrap_or_revert();
                bids.insert(bidder, bid);
                AuctionData::update_bids(bids);
            }
        }
    }

    fn find_new_winner() -> Option<(AccountHash, U512)> {
        let bids = AuctionData::get_bids();
        let winning_pair = bids.iter().max_by_key(|p| p.1);
        match winning_pair {
            Some((key, bid)) => Some((*key, *bid)),
            _ => None,
        }
    }

    fn get_bidder() -> Key {
        // Figure out who is trying to bid and what their bid is
        if let Some(CallStackElement::Session { account_hash }) = runtime::get_call_stack().first()
        {
            Key::Account(*account_hash)
        } else {
            runtime::revert(AuctionError::InvalidCaller)
        }
    }

    fn auction_transfer_token(recipient: Key) {
        let auction_key: Key = {
            let call_stack = runtime::get_call_stack();
            let caller: CallStackElement = call_stack.last().unwrap_or_revert().clone();
            match caller {
                CallStackElement::StoredContract {
                    contract_package_hash: _,
                    contract_hash: contract_hash_addr_caller,
                } => Key::Hash(contract_hash_addr_caller.value()),
                _ => runtime::revert(AuctionError::InvalidCaller),
            }
        };

        let token_contract_hash = AuctionData::get_nft_hash();
        let token_id_str = AuctionData::get_token_id();

        runtime::call_contract(
            token_contract_hash,
            "transfer_token",
            runtime_args! {
              "sender" => auction_key,
              "recipient" => recipient,
              "token_id" => token_id_str,
            },
        )
    }
}

impl crate::AuctionLogic for Auction {
    fn auction_allocate(winner: Option<AccountHash>) {
        match winner {
            Some(acct) => Self::auction_transfer_token(Key::Account(acct)),
            _ => Self::auction_transfer_token(AuctionData::get_token_owner()),
        }
    }

    fn auction_transfer(winner: Option<AccountHash>) {
        fn return_bids(mut bids: BTreeMap<AccountHash, U512>, auction_purse: URef) {
            for (bidder, bid) in &bids {
                system::transfer_from_purse_to_account(auction_purse, *bidder, *bid, None)
                    .unwrap_or_revert();
            }
            bids.clear();
            AuctionData::update_bids(bids);
        }
        let auction_purse = AuctionData::get_auction_purse();
        match winner {
            Some(key) => {
                let beneficiary_account = AuctionData::get_beneficiary();
                let mut bids = AuctionData::get_bids();
                match bids.get(&key) {
                    Some(bid) => {
                        system::transfer_from_purse_to_account(
                            auction_purse,
                            beneficiary_account,
                            *bid,
                            None,
                        )
                        .unwrap_or_revert();
                        bids.remove(&key);
                        return_bids(bids, auction_purse);
                    }
                    // Something went wrong, so better return everyone's money
                    _ => return_bids(bids, auction_purse),
                }
            }
            _ => {
                let bids = AuctionData::get_bids();
                return_bids(bids, auction_purse);
            }
        }
    }

    fn auction_bid() {
        // Figure out who is trying to bid and what their bid is
        let bidder = Self::get_bidder()
            .into_account()
            .unwrap_or_revert_with(AuctionError::InvalidCaller);
        let bid = runtime::get_named_arg::<U512>(crate::data::BID);
        if bid < AuctionData::get_reserve() {
            runtime::revert(AuctionError::BidTooLow);
        }
        let bidder_purse = runtime::get_named_arg::<URef>(crate::data::BID_PURSE);

        // Adding the bid, doing the purse transfer and resetting the winner if necessary, as well as possibly ending a Dutch auction
        let winner = AuctionData::get_winner();
        let price = AuctionData::get_price();
        if !AuctionData::is_english_format() {
            if let (None, None) = (winner, price) {
                if bid < AuctionData::get_current_price() {
                    runtime::revert(AuctionError::BidTooLow);
                }
                Self::add_bid(bidder, bidder_purse, AuctionData::get_current_price());
                AuctionData::reset_winner(Some(bidder), Some(bid));
                Self::auction_finalize(false);
            }
        } else {
            Self::add_bid(bidder, bidder_purse, bid);
            if let (Some(_), Some(current_price)) = (winner, price) {
                if bid > current_price {
                    AuctionData::reset_winner(Some(bidder), Some(bid));
                }
            }
        }
    }

    fn auction_cancel_bid() {
        let bidder = Self::get_bidder()
            .into_account()
            .unwrap_or_revert_with(AuctionError::InvalidCaller);
        let block_time = u64::from(runtime::get_blocktime());
        let cancellation_time = AuctionData::get_cancel_time();

        if block_time < cancellation_time {
            let mut bids = AuctionData::get_bids();
            match bids.get(&bidder) {
                Some(current_bid) => {
                    let auction_purse = AuctionData::get_auction_purse();
                    system::transfer_from_purse_to_account(
                        auction_purse,
                        bidder,
                        *current_bid,
                        None,
                    )
                    .unwrap_or_revert();
                    bids.remove(&bidder);
                    AuctionData::update_bids(bids);
                    match Self::find_new_winner() {
                        Some((winner, bid)) => AuctionData::reset_winner(Some(winner), Some(bid)),
                        _ => AuctionData::reset_winner(None, None),
                    }
                }
                _ => runtime::revert(AuctionError::NoBid),
            }
        } else {
            runtime::revert(AuctionError::LateCancellation)
        }
    }

    fn auction_finalize(time_check: bool) {
        // Get finalization and check if we're done
        let finalized = AuctionData::is_finalized();
        if finalized {
            runtime::revert(AuctionError::AlreadyFinal)
        };

        // We're not finalized, so let's get all the other arguments, as well as time to make sure we're not too early
        let end_time = AuctionData::get_end();
        let block_time = u64::from(runtime::get_blocktime());
        if time_check && block_time < end_time {
            runtime::revert(AuctionError::Early)
        }

        // TODO: Figure out how to gracefully finalize if the keys are bad
        match (
            finalized,
            AuctionData::get_price(),
            AuctionData::get_winner(),
        ) {
            (false, Some(_), Some(winner)) => {
                Self::auction_allocate(Some(winner));
                Self::auction_transfer(Some(winner));
                AuctionData::set_finalized()
            }
            _ => {
                Self::auction_allocate(None);
                Self::auction_transfer(None);
                AuctionData::set_finalized()
            }
        }
    }
}
