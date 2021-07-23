#![no_std]
#![no_main]

extern crate alloc;
use casper_types::{ApiError, EntryPoints, ContractHash, Key, EntryPoint, Parameter, CLType, EntryPointAccess, EntryPointType, runtime_args, RuntimeArgs, U256, U512};
use casper_contract::{unwrap_or_revert::UnwrapOrRevert, contract_api::{runtime, storage}};
use alloc::{vec, string::String};
use casper_private_auction_core::create_auction_named_keys;

#[no_mangle]
pub extern "C" fn bid() {

}

#[no_mangle]
pub extern "C" fn cancel_bid() {

}

#[no_mangle]
pub extern "C" fn finalize() {

}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        String::from("transfer_back"),
        vec![
            Parameter::new("token_contract_hash", CLType::Key),
            Parameter::new("sender", CLType::Key),
            Parameter::new("recipient", CLType::Key),
            Parameter::new("token_id", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let (_, _) = storage::new_locked_contract(
        entry_points,
        Some(create_auction_named_keys()),
        Some(String::from("auction_package_hash")),
        Some(String::from("auction_access_token")),
    );
}