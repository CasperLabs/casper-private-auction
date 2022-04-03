#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::String;
use casper_contract::{
    contract_api::{
        account::get_main_purse,
        runtime::{self, revert},
        system::{create_purse, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ApiError, ContractHash, RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    // You are required to use an argument called "amount" for the amount that you transfer our of a main_purse
    let amount: U512 = runtime::get_named_arg("amount");
    let auction_contract: ContractHash = runtime::get_named_arg("auction_contract");
    let purse_name: String = runtime::get_named_arg("purse_name");
    let bidder_purse: URef = match runtime::get_key(&purse_name) {
        Some(existing_purse) => existing_purse.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = create_purse();
            runtime::put_key(&purse_name, new_purse.into());
            new_purse
        }
    };
    transfer_from_purse_to_purse(get_main_purse(), bidder_purse, amount, None).unwrap_or_revert();
    let bidder_purse_out = bidder_purse.into_read_write();
    if !bidder_purse_out.is_writeable() || !bidder_purse_out.is_readable() {
        revert(ApiError::User(101));
    }
    let pre_bid =
        match runtime::call_contract::<Option<U512>>(auction_contract, "get_bid", runtime_args! {})
        {
            Some(bid) => bid,
            None => U512::zero(),
        };
    runtime::call_contract::<()>(
        auction_contract,
        "bid",
        runtime_args! {
            "bid_purse" => bidder_purse_out,
            "bid" => amount+pre_bid
        },
    );
}
