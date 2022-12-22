use crate::constants::{EVENTS, EVENTS_COUNT};
use crate::error::AuctionError;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use casper_contract::contract_api::runtime::{self, revert};
use casper_contract::contract_api::storage;
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::Key;
use casper_types::{account::AccountHash, U512};
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

pub fn emit(event: &AuctionEvent) {
    let mut events_count = get_events_count();

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
                        format!("{bidder}:{bid}")
                    }
                    None => "None".to_string(),
                },
            );
            event.insert("event_type", "Finalized".to_string());
            (event, event_id)
        }
    };
    events_count += 1;

    let events_dict = crate::Dict::at(EVENTS);
    events_dict.set(&event_id, emit_event);
    set_events_count(events_count);
}

pub fn get_events_count() -> u32 {
    if let Some(Key::URef(uref)) = runtime::get_key(EVENTS_COUNT) {
        return storage::read(uref)
            .unwrap_or_revert_with(AuctionError::CannotReadKey)
            .unwrap_or_revert_with(AuctionError::NamedKeyNotFound);
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
