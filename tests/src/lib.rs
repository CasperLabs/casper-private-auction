#![allow(unused)]
use std::time::Duration;

use casper_contract::contract_api::runtime;
use casper_engine_test_support::{
    internal::TIMESTAMP_MILLIS_INCREMENT, Code, Hash, SessionBuilder, TestContext,
    TestContextBuilder,
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};

pub mod auction;
pub mod auction_args;
pub mod nft;

#[test]
fn english_auction_bid_finalize_test() {
    let mut cep47 = nft::CasperCEP47Contract::deploy();
    let token_id = String::from("custom_token_id");
    let token_meta = nft::meta::red_dragon();
    cep47.mint_one(
        &Key::Account(cep47.admin),
        Some(&token_id),
        &token_meta,
        &(cep47.admin.clone()),
    );
    let nft::CasperCEP47Contract {
        mut context,
        hash,
        admin,
        ali,
        bob,
    } = cep47;
    let mut auction_contract = auction::AuctionContract::deploy(context, hash, &token_id, true);
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&ali, U512::from(30000), now);
    println!("bidded");
    std::thread::sleep(Duration::from_millis(3500));
    auction_contract.finalize();
    assert!(auction_contract.is_finalized());
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
    assert_eq!(ali, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(30000),
        auction_contract.get_winning_bid().unwrap()
    );
}
