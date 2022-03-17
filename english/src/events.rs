use crate::data::{EVENTS, EVENTS_COUNT};
use crate::error::AuctionError;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use casper_contract::contract_api::runtime::{self, revert};
use casper_contract::contract_api::storage;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::{account::AccountHash, U512};
use casper_types::{CLTyped, Key, URef};
pub enum AuctionEvent {
    Bid {
        bidder: AccountHash,
        bid: U512,
    },
    SetWinner {
        bidder: Option<AccountHash>,
        bid: Option<U512>,
    },
    BidCancelled {
        bidder: AccountHash,
    },
    Finalized {
        winner: Option<(AccountHash, U512)>,
    },
}

struct Dict {
    uref: URef,
}

impl Dict {
    pub fn at(name: &str) -> Dict {
        let key: Key = runtime::get_key(name).unwrap_or_revert();
        let uref: URef = *key.as_uref().unwrap_or_revert();
        Dict { uref }
    }

    pub fn _get<T: CLTyped + FromBytes>(&self, key: &str) -> Option<T> {
        storage::dictionary_get(self.uref, key)
            .unwrap_or_revert()
            .unwrap_or_default()
    }

    pub fn set<T: CLTyped + ToBytes>(&self, key: &str, value: T) {
        storage::dictionary_put(self.uref, key, Some(value));
    }

    pub fn _remove<T: CLTyped + ToBytes>(&self, key: &str) {
        storage::dictionary_put(self.uref, key, Option::<T>::None);
    }
}

pub fn emit(event: &AuctionEvent) {
    let mut events_count: u32 = if let Some(Key::URef(uref)) = runtime::get_key(EVENTS_COUNT) {
        storage::read(uref).unwrap_or_revert().unwrap_or_revert()
    } else {
        revert(AuctionError::BadKey)
    };

    let (emit_event, event_id): (BTreeMap<&str, String>, String) = match event {
        AuctionEvent::Bid { bidder, bid } => {
            let mut event = BTreeMap::new();
            let event_id = events_count.to_string();
            event.insert("event_id", event_id.clone());
            event.insert("bidder", bidder.to_string());
            event.insert("event_type", "Bid".to_string());
            event.insert("bid", bid.to_string());
            (event, event_id)
        }
        AuctionEvent::SetWinner { bidder, bid } => {
            let mut event = BTreeMap::new();
            let event_id = events_count.to_string();
            event.insert("event_id", event_id.clone());
            if bidder.is_some() {
                event.insert("winner", bidder.unwrap().to_string());
            }
            event.insert("event_type", "Bid".to_string());
            if bid.is_some() {
                event.insert("bid", bid.unwrap().to_string());
            }
            (event, event_id)
        }
        AuctionEvent::BidCancelled { bidder } => {
            let mut event = BTreeMap::new();
            let event_id = events_count.to_string();
            event.insert("event_id", event_id.clone());
            event.insert("bidder", bidder.to_string());
            event.insert("event_type", "BidCancelled".to_string());
            (event, event_id)
        }
        AuctionEvent::Finalized { winner } => {
            let mut event = BTreeMap::new();
            let event_id = events_count.to_string();
            event.insert("event_id", event_id.clone());
            event.insert(
                "winner",
                match winner {
                    Some((bidder, bid)) => {
                        format!("{}:{}", bidder, bid)
                    }
                    None => "None".to_string(),
                },
            );
            event.insert("event_type", "Finalized".to_string());
            (event, event_id)
        }
    };
    events_count += 1;

    let events_dict = Dict::at(EVENTS);
    events_dict.set(&event_id, emit_event);
    set_events_count(events_count);
}

pub fn get_events_count() -> u32 {
    if let Some(Key::URef(uref)) = runtime::get_key(EVENTS_COUNT) {
        return storage::read(uref).unwrap_or_revert().unwrap_or_revert();
    }
    revert(AuctionError::BadKey)
}

pub fn set_events_count(events_count: u32) {
    match runtime::get_key(EVENTS_COUNT) {
        Some(key) => {
            if let Key::URef(uref) = key {
                storage::write(uref, events_count);
            }
        }
        None => {
            let key = storage::new_uref(events_count).into();
            runtime::put_key(EVENTS_COUNT, key);
        }
    }
}
