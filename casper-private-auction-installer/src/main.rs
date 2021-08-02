#![no_std]
#![no_main]

extern crate alloc;
use casper_types::{EntryPoints, runtime_args, RuntimeArgs, EntryPoint, Parameter, CLType, EntryPointAccess, EntryPointType, Key};
use casper_contract::{contract_api::{system, runtime, storage}};
use alloc::{vec, string::String};
use casper_private_auction_core::{AUCTION_PURSE, BID, BID_PURSE, BID_FUNC, CANCEL_FUNC, FINALIZE_FUNC, AUCTION_ACCESS_TOKEN, auction_bid, auction_cancel_bid, auction_finalize, create_auction_named_keys, auction_receive_token, AUCTION_CONTRACT_HASH};

#[no_mangle]
pub extern "C" fn bid() {
    auction_bid();
}

#[no_mangle]
pub extern "C" fn cancel_bid() {
    auction_cancel_bid();
}

#[no_mangle]
pub extern "C" fn finalize() {
    auction_finalize(true);
}

#[no_mangle]
pub extern "C" fn add_auction_purse() {
    let purse = system::create_purse();
    runtime::put_key(AUCTION_PURSE, purse.into());
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    // TODO: Change back to Session type and use a call to a getter entry point to obtain auction purse URef (maybe?)
    entry_points.add_entry_point(EntryPoint::new(
        String::from(BID_FUNC),
        vec![
            Parameter::new(BID, CLType::U512),
            Parameter::new(BID_PURSE, CLType::URef)
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from(CANCEL_FUNC),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from(FINALIZE_FUNC),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // TODO: This needs to be one-time use only
    entry_points.add_entry_point(EntryPoint::new(
        String::from("add_auction_purse"),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let auction_named_keys = create_auction_named_keys();

    // TODO: Verify whether this is a contract or a package hash
    let (auction_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(auction_named_keys),
        Some(String::from(AUCTION_CONTRACT_HASH)),
        Some(String::from(AUCTION_ACCESS_TOKEN)),
    );

    // Create purse in the contract's context
    runtime::call_contract::<()>(
        auction_hash,
        "add_auction_purse",
        runtime_args! {}
    );

    let auction_hash_as_key = Key::Hash(auction_hash.value());
    auction_receive_token(auction_hash_as_key);
}