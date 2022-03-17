use alloc::{string::String, vec};
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

use crate::data::{
    read_named_key_uref, read_named_key_value, write_named_key_value, AuctionData, AUCTION_PURSE,
    BENEFICIARY_ACCOUNT, FINALIZED, NFT_HASH, OWNER, PRICE, RESERVE, TOKEN_ID, WINNER,
};
use crate::error::AuctionError;

pub struct Auction;

impl Auction {
    fn get_bidder() -> AccountHash {
        // Figure out who is trying to bid and what their bid is
        let call_stack = runtime::get_call_stack();
        if call_stack.len() != 2 {
            runtime::revert(AuctionError::InvalidCallStackLength)
        }
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
        let token_ids = vec![read_named_key_value::<String>(TOKEN_ID)];
        let token_contract_hash = ContractPackageHash::new(
            read_named_key_value::<Key>(NFT_HASH)
                .into_hash()
                .unwrap_or_revert(),
        );
        transfer_token(token_contract_hash, auction_key, recipient, token_ids);
    }

    fn auction_allocate() {
        match read_named_key_value::<Option<AccountHash>>(WINNER) {
            Some(acct) => {
                Self::auction_transfer_token(Key::Account(acct));
                let auction_purse = read_named_key_uref(AUCTION_PURSE);
                let bid = read_named_key_value::<Option<U512>>(PRICE).unwrap_or_revert();
                let share_piece = bid / 1000_u16;
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
            }
            _ => Self::auction_transfer_token(read_named_key_value::<Key>(OWNER)),
        }
    }

    pub fn auction_bid() {
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
        if let (None, None) = (winner, price) {
            let current_price = AuctionData::get_current_price();
            if bid < current_price {
                runtime::revert(AuctionError::BidTooLow);
            }
            let auction_purse = read_named_key_uref(AUCTION_PURSE);
            system::transfer_from_purse_to_purse(bidder_purse, auction_purse, bid, None)
                .unwrap_or_revert();

            write_named_key_value(WINNER, Some(bidder));
            write_named_key_value(PRICE, Some(bid));
            Self::auction_allocate();
            write_named_key_value(FINALIZED, true);
        } else {
            runtime::revert(AuctionError::BadState);
        }
    }

    pub fn cancel_auction() {
        if read_named_key_value::<Key>(OWNER) != Key::Account(runtime::get_caller()) {
            runtime::revert(AuctionError::InvalidCaller);
        }
        if read_named_key_value::<Option<AccountHash>>(WINNER).is_some() {
            runtime::revert(AuctionError::CannotCancelAuction);
        }

        Self::auction_allocate();
        write_named_key_value(FINALIZED, true);
    }
}

pub fn transfer_token(
    contract_package_hash: ContractPackageHash,
    auction_key: Key,
    recipient: Key,
    token_ids: alloc::vec::Vec<String>,
) {
    runtime::call_versioned_contract(
        contract_package_hash,
        None,
        "transfer",
        runtime_args! {
          "sender" => auction_key,
          "recipient" => recipient,
          "token_ids" => token_ids,
        },
    )
}
