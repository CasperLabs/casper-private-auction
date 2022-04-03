use core::iter::FromIterator;

use alloc::{collections::BTreeMap, string::ToString, vec::Vec};
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, Key, URef, U512};

use crate::error::AuctionError;

pub struct Bids {
    key_uref: URef,
    index_uref: URef,
    len: u64,
}

impl Bids {
    // Constructor for Bids. Should be used only once. For using the Bids use the `at` function.
    pub fn init() -> (URef, URef) {
        let key_uref = storage::new_dictionary("bids_key")
            .unwrap_or_revert_with(AuctionError::CannotCreateDictionary);
        let index_uref = storage::new_dictionary("bids_index")
            .unwrap_or_revert_with(AuctionError::CannotCreateDictionary);
        storage::dictionary_put(index_uref, "len", Some(0_u64));
        (key_uref, index_uref)
    }

    // Fetches the dictionary system user the argument `name`.
    pub fn at() -> Bids {
        let key_uref_key: Key =
            runtime::get_key("bids_key").unwrap_or_revert_with(AuctionError::DictionaryKeyNotFound);
        let index_uref_key: Key = runtime::get_key("bids_index")
            .unwrap_or_revert_with(AuctionError::DictionaryKeyNotFound);

        let key_uref: URef = *key_uref_key
            .as_uref()
            .unwrap_or_revert_with(AuctionError::DictionaryKeyNotURef);
        let index_uref: URef = *index_uref_key
            .as_uref()
            .unwrap_or_revert_with(AuctionError::DictionaryKeyNotURef);

        let len: Option<u64> = storage::dictionary_get(index_uref, "len")
            .unwrap_or_revert_with(AuctionError::DictionaryGetFailLen)
            .unwrap_or_revert_with(AuctionError::DictionaryGetNoValueLen);
        Bids {
            key_uref,
            index_uref,
            len: len.unwrap_or_default(),
        }
    }

    // Get a key corresponding to an index.
    pub fn get_key_by_index(&self, index: u64) -> Option<AccountHash> {
        storage::dictionary_get(self.index_uref, &index.to_string())
            .unwrap_or_revert_with(AuctionError::DictionaryGetFailGetByIndex)
            .unwrap_or_default()
    }

    // Return the index a key is stored under.
    pub fn get_index_by_key(&self, key: &AccountHash) -> Option<u64> {
        storage::dictionary_get(self.index_uref, &key.to_string())
            .unwrap_or_revert_with(AuctionError::DictionaryGetFailGetByKey)
            .unwrap_or_default()
    }

    // If exists, returns the value stored under a key.
    pub fn get(&self, key: &AccountHash) -> Option<U512> {
        storage::dictionary_get(self.key_uref, &key.to_string())
            .unwrap_or_revert_with(AuctionError::DictionaryGetFailBidsGet)
            .unwrap_or_default()
    }

    // Getter for the value at the nth place in the dictionary.
    pub fn nth(&mut self, n: u64) -> Option<U512> {
        if let Some(key) = self.get_key_by_index(n) {
            let ret = self.get(&key);
            return ret;
        }
        None
    }

    // Public method for adding new entry to the Bids. If key already exists, does nothing.
    pub fn insert(&mut self, key: &AccountHash, value: U512) {
        if self.get(key).is_none() {
            self.set_value_to_key(key, Some(value));
            self.set_key_to_index(self.len, key);
            self.set_len(self.len + 1);
        }
    }

    // Replaces as existing entry, or if one is not present, inserts a new one.
    pub fn replace(&mut self, key: &AccountHash, value: U512) {
        if self.get(key).is_some() {
            self.set_value_to_key(key, Some(value));
            return;
        }
        self.insert(key, value);
    }

    // Switch key at index. Can also replace value at that key.
    pub fn replace_index(&mut self, index: u64, key: &AccountHash, value: Option<U512>) {
        if self.nth(index).is_some() {
            self.set_key_to_index(index, key);
            if let Some(v) = value {
                self.set_value_to_key(key, Some(v))
            }
            return;
        }
        if let Some(value) = value {
            self.insert(key, value);
        }
    }

    // If an index is associated to the key, calls `remove_by_index`, and in case an index is not present still tries to remove the value at the key.
    pub fn remove_by_key(&mut self, key: &AccountHash) {
        match self.get_index_by_key(key) {
            Some(index) => {
                self.remove_by_index(index);
            }
            None => {
                if self.get(key).is_some() {
                    self.set_value_to_key(key, Option::<U512>::None);
                }
            }
        }
    }

    // Remove all data under a certain index. Index, key, value included.
    // If the index is the last one, `pop`s it, if index is smaller than that, moves the last data to index,
    // and removes the key and value previously stored at that index.
    // Decreased `len`.
    pub fn remove_by_index(&mut self, index: u64) {
        if let Some(key) = self.get_key_by_index(index) {
            match self.len.cmp(&(index + 1)) {
                core::cmp::Ordering::Equal => {
                    self.pop();
                }
                core::cmp::Ordering::Greater => {
                    if let Some(last_key) = self.get_key_by_index(self.len - 1) {
                        self.set_key_to_index(index, &last_key);
                        self.set_key_to_index(self.len - 1, &key);
                    }
                    self.pop();
                }
                core::cmp::Ordering::Less => (),
            }
        }
    }

    // Removes and returns the data and key at the last index from the dictionary.
    pub fn pop(&mut self) -> Option<(AccountHash, U512)> {
        let index = self.len - 1;
        if let Some(key) = self.get_key_by_index(index) {
            if let Some(value) = self.get(&key) {
                storage::dictionary_put(
                    self.index_uref,
                    &index.to_string(),
                    Option::<AccountHash>::None,
                );
                storage::dictionary_put(self.index_uref, &key.to_string(), Option::<u64>::None);
                self.set_value_to_key(&key, Option::<U512>::None);
                self.set_len(index);
                return Some((key, value));
            }
        }
        None
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn set_value_to_key(&self, key: &AccountHash, value: Option<U512>) {
        storage::dictionary_put(self.key_uref, &key.to_string(), value);
    }

    fn set_key_to_index(&self, index: u64, key: &AccountHash) {
        storage::dictionary_put(self.index_uref, &index.to_string(), Some(*key));
        storage::dictionary_put(self.index_uref, &key.to_string(), Some(index));
    }

    fn set_len(&mut self, length: u64) {
        storage::dictionary_put(self.index_uref, "len", Some(length));
        self.len = length;
    }

    pub fn to_map(&self) -> BTreeMap<AccountHash, U512> {
        let mut ret = BTreeMap::new();
        for i in 0..self.len {
            let key = self
                .get_key_by_index(i)
                .unwrap_or_revert_with(AuctionError::DictionaryGetNoValueGetByKey);
            let value = self
                .get(&key)
                .unwrap_or_revert_with(AuctionError::DictionaryGetFailBidsGet);
            ret.insert(key, value);
        }
        ret
    }

    pub fn clear(&mut self) {
        for _ in 0..self.len {
            self.pop();
        }
    }

    pub fn max_by_key(&self) -> Option<(AccountHash, U512)> {
        if !self.is_empty() {
            let mut max_key = self
                .get_key_by_index(0)
                .unwrap_or_revert_with(AuctionError::DictionaryGetNoValueGetByIndex);
            let mut max_value = self
                .get(&max_key)
                .unwrap_or_revert_with(AuctionError::DictionaryGetFailBidsGet);
            for i in 1..self.len {
                let key = self
                    .get_key_by_index(i)
                    .unwrap_or_revert_with(AuctionError::DictionaryGetNoValueGetByIndex);
                let value = self
                    .get(&key)
                    .unwrap_or_revert_with(AuctionError::DictionaryGetFailBidsGet);
                if value > max_value {
                    max_key = key;
                    max_value = value;
                }
            }
            return Some((max_key, max_value));
        }
        None
    }

    /// Returns the account hash of the lowest bidder if the new bid is higher
    pub fn get_spot(&self, new_item: U512) -> Option<(AccountHash, U512)> {
        let mut bidders = Vec::from_iter(self.to_map());
        bidders.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        let (lowest_bidder, lowest_bid) = bidders
            .pop()
            .unwrap_or_revert_with(AuctionError::UnreachableDeadEnd);
        if lowest_bid < new_item {
            Some((lowest_bidder, lowest_bid))
        } else {
            None
        }
    }
}
