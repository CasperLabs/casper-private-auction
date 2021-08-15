#![no_std]
#![no_main]

extern crate alloc;
use alloc::{string::String, vec};
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage, system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_private_auction_core::{auction::Auction, data, AuctionLogic};
use casper_types::{
    runtime_args, CLType, ContractHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Key, Parameter, RuntimeArgs,
};

#[no_mangle]
pub extern "C" fn bid() {
    Auction::auction_bid();
}

#[no_mangle]
pub extern "C" fn cancel_bid() {
    Auction::auction_cancel_bid();
}

#[no_mangle]
pub extern "C" fn finalize() {
    Auction::auction_finalize(true);
}

#[no_mangle]
pub extern "C" fn add_auction_purse() {
    let purse = system::create_purse();
    runtime::put_key(data::AUCTION_PURSE, purse.into());
}

pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        data::BID,
        vec![
            Parameter::new(data::BID, CLType::U512),
            Parameter::new(data::BID_PURSE, CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        data::CANCEL_FUNC,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        data::FINALIZE_FUNC,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // TODO: This needs to be one-time use only
    entry_points.add_entry_point(EntryPoint::new(
        "add_auction_purse",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = get_entry_points();
    let auction_named_keys = data::create_auction_named_keys();

    // TODO: Store the contract hash, not just the package hash
    let (auction_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(auction_named_keys),
        Some(String::from(data::AUCTION_CONTRACT_HASH)),
        Some(String::from(data::AUCTION_ACCESS_TOKEN)),
    );
    runtime::put_key("auction_contract_hash", auction_hash.into());
    // Test-env wrapper
    runtime::put_key(
        "auction_contract_hash_wrapped",
        storage::new_uref(auction_hash).into(),
    );
    // Create purse in the contract's context
    runtime::call_contract::<()>(auction_hash, "add_auction_purse", runtime_args! {});
    let auction_key = Key::Hash(auction_hash.value());

    let token_owner = Key::Account(runtime::get_caller());
    let token_contract_hash = ContractHash::new(
        runtime::get_named_arg::<Key>(data::NFT_HASH)
            .into_hash()
            .unwrap_or_revert(),
    );
    let token_id_str = runtime::get_named_arg::<String>(data::TOKEN_ID);
    // revert(ApiError::User(666));
    runtime::call_contract(
        token_contract_hash,
        "transfer_token",
        runtime_args! {
          "sender" => token_owner,
          "recipient" => auction_key,
          "token_id" => token_id_str,
        },
    )
}
