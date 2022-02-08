#![no_std]

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, Key, URef,
};
use error::AuctionError;

extern crate alloc;

pub mod auction;
pub mod error;
#[macro_use]
pub mod data;
pub mod bids;
pub mod events;

pub trait AuctionLogic {
    fn auction_bid();
    fn auction_cancel_bid();
    fn auction_allocate(winner: Option<AccountHash>);
    fn auction_transfer(winner: Option<AccountHash>);
    fn auction_finalize(time_check: bool);
    fn cancel_auction();
}

struct Dict {
    uref: URef,
}

impl Dict {
    pub fn at(name: &str) -> Dict {
        let key: Key =
            runtime::get_key(name).unwrap_or_revert_with(AuctionError::DictionaryKeyNotFound);
        let uref: URef = *key
            .as_uref()
            .unwrap_or_revert_with(AuctionError::DictionaryKeyNotURef);
        Dict { uref }
    }

    pub fn _get<T: CLTyped + FromBytes>(&self, key: &str) -> Option<T> {
        storage::dictionary_get(self.uref, key)
            .unwrap_or_revert_with(AuctionError::DictionaryGetFail)
            .unwrap_or_default()
    }

    pub fn set<T: CLTyped + ToBytes>(&self, key: &str, value: T) {
        storage::dictionary_put(self.uref, key, Some(value));
    }

    pub fn _remove<T: CLTyped + ToBytes>(&self, key: &str) {
        storage::dictionary_put(self.uref, key, Option::<T>::None);
    }
}
