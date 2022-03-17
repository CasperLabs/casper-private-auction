use alloc::string::String;
use casper_contract::{
    contract_api::{
        runtime::{self, get_call_stack},
        system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
pub use casper_types::bytesrepr::ToBytes;
use casper_types::{account::AccountHash, ContractPackageHash};
pub use casper_types::{
    bytesrepr::FromBytes, contracts::NamedKeys, runtime_args, system::CallStackElement, ApiError,
    CLTyped, ContractHash, Key, RuntimeArgs, URef, U512,
};

use crate::{
    bids::Bids,
    data::{
        read_named_key_uref, read_named_key_value, write_named_key_value, AUCTION_PURSE,
        BENEFICIARY_ACCOUNT, BIDDER_NUMBER_CAP, CANCEL, END, FINALIZED, MINIMUM_BID_STEP, NFT_HASH,
        OWNER, PRICE, RESERVE, TOKEN_ID, WINNER,
    },
    error::AuctionError,
};
use crate::{
    data::AuctionData,
    events::{emit, AuctionEvent},
};

pub struct Auction;

impl Auction {
    fn add_bid(bidder: AccountHash, bidder_purse: URef, new_bid: U512) {
        if !AuctionData::is_auction_live() || read_named_key_value::<bool>(FINALIZED) {
            runtime::revert(AuctionError::BadState)
        }
        if !AuctionData::is_kyc_proved() {
            runtime::revert(AuctionError::KYCError);
        }
        // Get the existing bid, if any
        let mut bids = Bids::at();
        let auction_purse = read_named_key_uref(AUCTION_PURSE);
        if bids.get(&bidder).is_none() {
            if let Some(bidder_cap) = read_named_key_value::<Option<u64>>(BIDDER_NUMBER_CAP) {
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
        let bids = Bids::at();
        let winning_pair = bids.max_by_key();
        match winning_pair {
            Some((key, bid)) => Some((key, bid)),
            _ => None,
        }
    }

    fn get_bidder() -> AccountHash {
        // Figure out who is trying to bid and what their bid is
        let call_stack = runtime::get_call_stack();
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
        token_ids.push(read_named_key_value::<String>(TOKEN_ID));
        runtime::call_versioned_contract(
            ContractPackageHash::new(
                read_named_key_value::<Key>(NFT_HASH)
                    .into_hash()
                    .unwrap_or_revert(),
            ),
            None,
            "transfer",
            runtime_args! {
              "sender" => auction_key,
              "recipient" => recipient,
              "token_ids" => token_ids,
            },
        )
    }

    fn auction_allocate(winner: Option<AccountHash>) {
        match winner {
            Some(acct) => Self::auction_transfer_token(Key::Account(acct)),
            _ => Self::auction_transfer_token(read_named_key_value::<Key>(OWNER)),
        }
    }

    fn auction_transfer(winner: Option<AccountHash>) {
        fn return_bids(auction_purse: URef) {
            let mut bids = Bids::at();
            for (bidder, bid) in &bids.to_map() {
                system::transfer_from_purse_to_account(auction_purse, *bidder, *bid, None)
                    .unwrap_or_revert();
            }
            bids.clear();
        }
        let auction_purse = read_named_key_uref(AUCTION_PURSE);
        match winner {
            Some(key) => {
                let mut bids = Bids::at();
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
                            read_named_key_value::<Key>(BENEFICIARY_ACCOUNT)
                                .into_account()
                                .unwrap_or_revert(),
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

    pub(crate) fn auction_bid() {
        if !AuctionData::is_auction_live() || read_named_key_value::<bool>(FINALIZED) {
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
        if bid < read_named_key_value::<U512>(RESERVE) {
            runtime::revert(AuctionError::BidBelowReserve);
        }
        let bidder_purse = runtime::get_named_arg::<URef>(crate::data::BID_PURSE);

        // Adding the bid, doing the purse transfer and resetting the winner if necessary, as well as possibly ending a Dutch auction
        let winner = read_named_key_value::<Option<AccountHash>>(WINNER);
        let price = read_named_key_value::<Option<U512>>(PRICE);

        Self::add_bid(bidder, bidder_purse, bid);
        if let (Some(_), Some(current_price)) = (winner, price) {
            let min_step =
                read_named_key_value::<Option<U512>>(MINIMUM_BID_STEP).unwrap_or_default();
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

        AuctionData::increase_auction_times();
        emit(&AuctionEvent::Bid { bidder, bid })
    }

    pub(crate) fn auction_cancel_bid() {
        let bidder = Self::get_bidder();

        if u64::from(runtime::get_blocktime()) < read_named_key_value::<u64>(CANCEL) {
            let mut bids = Bids::at();

            match bids.get(&bidder) {
                Some(current_bid) => {
                    system::transfer_from_purse_to_account(
                        read_named_key_uref(AUCTION_PURSE),
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

    pub(crate) fn auction_finalize(time_check: bool) {
        // Get finalization and check if we're done
        if read_named_key_value::<bool>(FINALIZED) {
            runtime::revert(AuctionError::AlreadyFinal)
        };

        // We're not finalized, so let's get all the other arguments, as well as time to make sure we're not too early
        if time_check && u64::from(runtime::get_blocktime()) < read_named_key_value::<u64>(END) {
            runtime::revert(AuctionError::EarlyFinalize)
        }

        // TODO: Figure out how to gracefully finalize if the keys are bad
        let winner = match (
            read_named_key_value::<Option<U512>>(PRICE),
            read_named_key_value::<Option<AccountHash>>(WINNER),
        ) {
            (Some(winning_bid), Some(winner)) => {
                Self::auction_allocate(Some(winner));
                Self::auction_transfer(Some(winner));
                write_named_key_value(FINALIZED, true);
                Some((winner, winning_bid))
            }
            _ => {
                Self::auction_allocate(None);
                Self::auction_transfer(None);
                write_named_key_value(FINALIZED, true);
                None
            }
        };
        emit(&AuctionEvent::Finalized { winner })
    }

    pub(crate) fn cancel_auction() {
        if read_named_key_value::<Key>(OWNER) != Key::Account(runtime::get_caller()) {
            runtime::revert(AuctionError::InvalidCaller);
        }
        if !Bids::at().is_empty() && read_named_key_value::<Option<AccountHash>>(WINNER).is_some() {
            runtime::revert(AuctionError::CannotCancelAuction);
        }

        Self::auction_allocate(None);
        Self::auction_transfer(None);
        write_named_key_value(FINALIZED, true);
    }
}
