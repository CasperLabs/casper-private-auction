#![no_std]
#![no_main]

extern crate alloc;

mod auction;
mod data;
mod error;

use alloc::{format, string::String, vec};
use auction::Auction;
use casper_contract::{
    contract_api::{
        runtime::{self},
        storage, system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    runtime_args, CLType, ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Key, Parameter, RuntimeArgs,
};

pub const INIT: &str = "init";
pub const AUCTION_CONTRACT_HASH_NAME_SUFFIX_PART: &str = "_auction_contract_hash";

#[no_mangle]
pub extern "C" fn bid() {
    Auction::auction_bid();
}

#[no_mangle]
pub extern "C" fn cancel_auction() {
    Auction::cancel_auction();
}

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(data::AUCTION_PURSE).is_none() {
        let purse = system::create_purse();
        runtime::put_key(data::AUCTION_PURSE, purse.into());
    }
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
        data::CANCEL_AUCTION_FUNC,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        INIT,
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
    let auction_desig: String = runtime::get_named_arg("name");
    let (auction_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(auction_named_keys),
        Some(format!("{}_{}", auction_desig, data::AUCTION_CONTRACT_HASH)),
        Some(format!("{}_{}", auction_desig, data::AUCTION_ACCESS_TOKEN)),
    );
    let auction_key = Key::Hash(auction_hash.value());
    runtime::put_key(
        &format!(
            "{}{}",
            auction_desig, AUCTION_CONTRACT_HASH_NAME_SUFFIX_PART
        ),
        auction_key,
    );
    runtime::put_key(
        &format!(
            "{}{}_wrapped",
            auction_desig, AUCTION_CONTRACT_HASH_NAME_SUFFIX_PART
        ),
        storage::new_uref(auction_hash).into(),
    );

    // Create purse in the contract's context
    runtime::call_contract::<()>(auction_hash, INIT, runtime_args! {});

    // Hash of the NFT contract put up for auction
    let token_contract_hash = ContractPackageHash::new(
        runtime::get_named_arg::<Key>(data::NFT_HASH)
            .into_hash()
            .unwrap_or_revert(),
    );
    // Transfer the NFT ownership to the auction
    let token_ids = vec![runtime::get_named_arg::<String>(data::TOKEN_ID)];

    let auction_contract_package_hash = runtime::get_key(&format!(
        "{}_{}",
        auction_desig,
        data::AUCTION_CONTRACT_HASH
    ))
    .unwrap_or_revert();
    runtime::put_key(
        &format!("{}_auction_contract_package_hash_wrapped", auction_desig),
        storage::new_uref(ContractPackageHash::new(
            auction_contract_package_hash.into_hash().unwrap_or_revert(),
        ))
        .into(),
    );
    auction::transfer_token(
        token_contract_hash,
        Key::Account(runtime::get_caller()),
        auction_contract_package_hash,
        token_ids,
    );
}
