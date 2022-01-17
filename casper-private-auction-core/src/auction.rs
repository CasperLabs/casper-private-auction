use casper_contract::{
    contract_api::{
        runtime::{self, get_call_stack},
        system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::account::AccountHash;
pub use casper_types::bytesrepr::ToBytes;
pub use casper_types::{
    bytesrepr::FromBytes, contracts::NamedKeys, runtime_args, system::CallStackElement, ApiError,
    CLTyped, ContractHash, Key, RuntimeArgs, URef, U512,
};

use crate::error::AuctionError;
use crate::{
    data::AuctionData,
    events::{emit, AuctionEvent},
};

pub struct Auction;

impl Auction {
    fn add_bid(bidder: AccountHash, bidder_purse: URef, new_bid: U512) {
        if !AuctionData::is_auction_live() || AuctionData::is_finalized() {
            runtime::revert(AuctionError::BadState)
        }
        if !AuctionData::is_kyc_proved() {
            runtime::revert(AuctionError::KYCError);
        }
        // Get the existing bid, if any
        let mut bids = AuctionData::get_bids();
        let auction_purse = AuctionData::get_auction_purse();
        if bids.get(&bidder).is_none() {
            if let Some(bidder_cap) = AuctionData::get_bidder_count_cap() {
                if bidder_cap <= bids.len() {
                    if let Some((lowest_bidder, lowest_bid)) = bids.get_spot(new_bid) {
                        bids.remove_by_key(&lowest_bidder);
                        system::transfer_from_purse_to_account(
                            auction_purse,
                            lowest_bidder,
                            lowest_bid,
                            None,
                        )
                        .unwrap_or_revert();
                    }
                }
            }
        }
        let bid_amount = if let Some(current_bid) = bids.get(&bidder) {
            if new_bid <= current_bid {
                runtime::revert(AuctionError::NewBidLower)
            }
            new_bid - current_bid
        } else {
            new_bid
        };
        system::transfer_from_purse_to_purse(bidder_purse, auction_purse, bid_amount, None)
            .unwrap_or_revert();
        bids.replace(&bidder, new_bid);
    }

    fn find_new_winner() -> Option<(AccountHash, U512)> {
        let bids = AuctionData::get_bids();
        let winning_pair = bids.max_by_key();
        match winning_pair {
            Some((key, bid)) => Some((key, bid)),
            _ => None,
        }
    }

    fn get_bidder() -> AccountHash {
        // Figure out who is trying to bid and what their bid is
        let call_stack = runtime::get_call_stack();
        // if call_stack.len() == 2 {runtime::revert(AuctionError::InvalidCallStackLenght)}
        if let Some(CallStackElement::Session { account_hash }) = call_stack.first() {
            *account_hash
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
                    contract_package_hash,
                    contract_hash: _,
                } => Key::Hash(contract_package_hash.value()),
                _ => runtime::revert(AuctionError::InvalidCaller),
            }
        };
        let mut token_ids = alloc::vec::Vec::new();
        token_ids.push(AuctionData::get_token_id());
        runtime::call_versioned_contract(
            AuctionData::get_nft_hash(),
            None,
            "transfer",
            runtime_args! {
              "sender" => auction_key,
              "recipient" => recipient,
              "token_ids" => token_ids,
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
        fn return_bids(auction_purse: URef) {
            let mut bids = AuctionData::get_bids();
            for (bidder, bid) in &bids.to_map() {
                system::transfer_from_purse_to_account(auction_purse, *bidder, *bid, None)
                    .unwrap_or_revert();
            }
            bids.clear();
        }
        let auction_purse = AuctionData::get_auction_purse();
        match winner {
            Some(key) => {
                let mut bids = AuctionData::get_bids();
                match bids.get(&key) {
                    Some(bid) => {
                        // Every actor receives x one-thousandth of the winning bid, the surplus goes to the designated beneficiary account.
                        let share_piece = bid / 1000;
                        let mut given_as_shares = U512::zero();
                        for (account, share) in AuctionData::get_commission_shares() {
                            let actor_share = share_piece * share;
                            system::transfer_from_purse_to_account(
                                auction_purse,
                                account,
                                actor_share,
                                None,
                            )
                            .unwrap_or_revert();
                            given_as_shares += actor_share;
                        }
                        system::transfer_from_purse_to_account(
                            auction_purse,
                            AuctionData::get_beneficiary(),
                            bid - given_as_shares,
                            None,
                        )
                        .unwrap_or_revert();
                        bids.remove_by_key(&key);
                        return_bids(auction_purse);
                    }
                    // Something went wrong, so better return everyone's money
                    _ => return_bids(auction_purse),
                }
            }
            _ => {
                return_bids(auction_purse);
            }
        }
    }

    fn auction_bid() {
        if !AuctionData::is_auction_live() || AuctionData::is_finalized() {
            runtime::revert(AuctionError::BadState)
        }
        if !AuctionData::is_kyc_proved() {
            runtime::revert(AuctionError::KYCError);
        }
        if get_call_stack().len() != 2 {
            runtime::revert(AuctionError::DisallowedMiddleware);
        }
        // We do not check times here because we do that in Auction::add_bid
        // Figure out who is trying to bid and what their bid is
        let bidder = Self::get_bidder();
        let bid = runtime::get_named_arg::<U512>(crate::data::BID);
        if bid < AuctionData::get_reserve() {
            runtime::revert(AuctionError::BidBelowReserve);
        }
        let bidder_purse = runtime::get_named_arg::<URef>(crate::data::BID_PURSE);

        // Adding the bid, doing the purse transfer and resetting the winner if necessary, as well as possibly ending a Dutch auction
        let winner = AuctionData::get_winner();
        let price = AuctionData::get_price();
        if !AuctionData::is_english_format() {
            if let (None, None) = (winner, price) {
                let current_price = AuctionData::get_current_price();
                if bid < current_price {
                    runtime::revert(AuctionError::BidTooLow);
                }
                Self::add_bid(bidder, bidder_purse, current_price);
                AuctionData::set_winner(Some(bidder), Some(bid));
                Self::auction_finalize(false);
            } else {
                runtime::revert(AuctionError::BadState);
            }
        } else {
            Self::add_bid(bidder, bidder_purse, bid);
            if let (Some(_), Some(current_price)) = (winner, price) {
                let min_step = AuctionData::get_minimum_bid_step().unwrap_or_default();
                if bid > current_price && bid - current_price >= min_step {
                    AuctionData::set_winner(Some(bidder), Some(bid));
                } else {
                    runtime::revert(AuctionError::BidTooLow)
                }
            } else if let (None, None) = (winner, price) {
                AuctionData::set_winner(Some(bidder), Some(bid));
            } else {
                runtime::revert(AuctionError::BadState)
            }
        }

        AuctionData::increase_auction_times();
        emit(&AuctionEvent::Bid { bidder, bid })
    }

    fn auction_cancel_bid() {
        let bidder = Self::get_bidder();

        if u64::from(runtime::get_blocktime()) < AuctionData::get_cancel_time() {
            let mut bids = AuctionData::get_bids();

            match bids.get(&bidder) {
                Some(current_bid) => {
                    system::transfer_from_purse_to_account(
                        AuctionData::get_auction_purse(),
                        bidder,
                        current_bid,
                        None,
                    )
                    .unwrap_or_revert();
                    bids.remove_by_key(&bidder);
                    match Self::find_new_winner() {
                        Some((winner, bid)) => AuctionData::set_winner(Some(winner), Some(bid)),
                        _ => AuctionData::set_winner(None, None),
                    }
                }
                None => runtime::revert(AuctionError::NoBid),
            }
        } else {
            runtime::revert(AuctionError::LateCancellation)
        }
        emit(&AuctionEvent::BidCancelled { bidder })
    }

    fn auction_finalize(time_check: bool) {
        // Get finalization and check if we're done
        if AuctionData::is_finalized() {
            runtime::revert(AuctionError::AlreadyFinal)
        };

        // We're not finalized, so let's get all the other arguments, as well as time to make sure we're not too early
        if time_check && u64::from(runtime::get_blocktime()) < AuctionData::get_end() {
            runtime::revert(AuctionError::EarlyFinalize)
        }

        // TODO: Figure out how to gracefully finalize if the keys are bad
        let winner = match (AuctionData::get_price(), AuctionData::get_winner()) {
            (Some(winning_bid), Some(winner)) => {
                Self::auction_allocate(Some(winner));
                Self::auction_transfer(Some(winner));
                AuctionData::set_finalized();
                Some((winner, winning_bid))
            }
            _ => {
                Self::auction_allocate(None);
                Self::auction_transfer(None);
                AuctionData::set_finalized();
                None
            }
        };
        emit(&AuctionEvent::Finalized { winner })
    }

    fn cancel_auction() {
        if AuctionData::get_token_owner() != Key::Account(runtime::get_caller()) {
            runtime::revert(AuctionError::InvalidCaller);
        }
        if !AuctionData::get_bids().is_empty() && AuctionData::get_winner().is_some() {
            runtime::revert(AuctionError::CannotCancelAuction);
        }

        Self::auction_allocate(None);
        Self::auction_transfer(None);
        AuctionData::set_finalized();
    }
}
