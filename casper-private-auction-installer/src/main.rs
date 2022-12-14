#![no_std]
#![no_main]

extern crate alloc;
use alloc::{boxed::Box, format, string::String, vec};
use casper_contract::{
    contract_api::{
        runtime::{self, get_caller},
        storage, system,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_private_auction_core::{
    auction::Auction,
    bids::Bids,
    data::{self, HAS_ENHANCED_NFT},
    AuctionLogic,
};
use casper_types::{
    runtime_args, ApiError, CLType, CLValue, ContractPackageHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, RuntimeArgs,
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
pub extern "C" fn cancel_auction() {
    Auction::cancel_auction();
}

#[no_mangle]
pub extern "C" fn get_bid() {
    let bids = Bids::at();
    let bid = bids.get(&get_caller());
    runtime::ret(CLValue::from_t(bid).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(data::AUCTION_PURSE).is_none() {
        let purse = system::create_purse();
        runtime::put_key(data::AUCTION_PURSE, purse.into());
        Bids::init();
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

    entry_points.add_entry_point(EntryPoint::new(
        data::CANCEL_AUCTION_FUNC,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "get_bid",
        vec![],
        CLType::Option(Box::new(CLType::U512)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "init",
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
        &format!("{auction_desig}_auction_contract_hash"),
        auction_key,
    );
    runtime::put_key(
        &format!("{auction_desig}_auction_contract_hash_wrapped"),
        storage::new_uref(auction_hash).into(),
    );

    // Create purse in the contract's context
    runtime::call_contract::<()>(auction_hash, "init", runtime_args! {});

    // Hash of the NFT contract put up for auction
    let token_contract_hash = ContractPackageHash::new(
        runtime::get_named_arg::<Key>(data::NFT_HASH)
            .into_hash()
            .unwrap_or_revert_with(ApiError::User(200)),
    );
    // Transfer the NFT ownership to the auction
    let token_id: String = runtime::get_named_arg::<String>(data::TOKEN_ID);
    let token_hash = base16::encode_lower(&runtime::blake2b(&token_id));
    let token_ids = vec![token_id];

    let auction_contract_package_hash = runtime::get_key(&format!(
        "{}_{}",
        auction_desig,
        data::AUCTION_CONTRACT_HASH
    ))
    .unwrap_or_revert_with(ApiError::User(201));
    runtime::put_key(
        &format!("{auction_desig}_auction_contract_package_hash_wrapped"),
        storage::new_uref(ContractPackageHash::new(
            auction_contract_package_hash
                .into_hash()
                .unwrap_or_revert_with(ApiError::User(202)),
        ))
        .into(),
    );

    let has_enhanced_nft = runtime::get_named_arg::<bool>(HAS_ENHANCED_NFT);
    if !has_enhanced_nft {
        // CEP-47  Transfer
        runtime::call_versioned_contract::<()>(
            token_contract_hash,
            None,
            "transfer",
            runtime_args! {
                "sender" => Key::Account(runtime::get_caller()),
                "recipient" => auction_contract_package_hash,
                "token_ids" => token_ids,
            },
        );
    } else {
        // CEP-78 Transfer
        runtime::call_versioned_contract::<(String, Key)>(
            token_contract_hash,
            None,
            "transfer",
            runtime_args! {
                "source_key" => Key::Account(runtime::get_caller()),
                "target_key" => auction_key,
                "token_hash" => token_hash,
            },
        );
    }
}
