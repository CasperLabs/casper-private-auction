#![no_std]
#![no_main]

use casper_contract::{
    contract_api::{
        account::get_main_purse,
        runtime,
        system::{create_purse, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, RuntimeArgs, U512};

#[no_mangle]
pub extern "C" fn call() {
    let bid: U512 = runtime::get_named_arg("bid");
    let purse = create_purse();
    transfer_from_purse_to_purse(get_main_purse(), purse, bid, None).unwrap_or_revert();
    runtime::call_contract(
        runtime::get_named_arg("auction_contract"),
        "bid",
        runtime_args! {
          "bid" => bid,
          "bid_purse" => purse,
        },
    )
}
