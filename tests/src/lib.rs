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
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
    assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
        auction_contract.get_winning_bid().unwrap()
    );
}

#[test]
fn english_auction_bid_cancel_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1);
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now + 2);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 3);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
    assert_eq!(auction_contract.ali, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(30000),
        auction_contract.get_winning_bid().unwrap()
    );
}

/*
pub enum AuctionError {
    EarlyFinalize = 0,
    InvalidCaller = 1,
    LateBid = 2,
    BidTooLow = 3,
    AlreadyFinal = 4,
    BadState = 5,
    NoBid = 6,
    LateCancellation = 7,
    UnknownFormat = 8,
    InvalidTimes = 9,
    InvalidPrices = 10,
    EarlyBid = 11,
    InvalidBeneficiary = 12,
    BadKey = 13,
    Internal = 14,
    InternalTwo = 15,
}
*/

// Finalizing the auction before it ends results in User(0) error
#[test]
#[should_panic = "User(0)"]
fn english_auction_early_finalize_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 300);
}

/*
// User error 1 happens if not the correct user is trying to interact with the auction.
// More precisely, if a) the bidder is a contract. b) someone other than a stored contact is trying to transfer out the auctioned token
#[test]
#[should_panic = "User(1)"]
fn english_auction_early_finalize_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.finalize(&auction_contract.admin.clone(),now + 300);
}
*/

// Trying to bid after the end of the auction results in User(2) error
#[test]
#[should_panic = "User(2)"]
fn english_auction_bid_too_late_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(
        &auction_contract.bob.clone(),
        U512::from(40000),
        now + 10000,
    );
}

// Trying to bid an amount below the reserve results in User(3) error
#[test]
#[should_panic = "User(3)"]
fn english_auction_bid_too_low_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(1), now + 10000);
}

// Finalizing after finalizing is User(4) error.
#[test]
#[should_panic = "User(4)"]
fn english_auction_bid_after_finalized_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3501);
}

// User(5) is BadState. Either the auction managed to be finalized before expiring, or Dutch contract was initialized without starting price.
// TODO: simulate User(5), cannot currently do it as auction init throws back the state with User(10)

// User(6) error -> trying to cancel a bid that wasn't placed
#[test]
#[should_panic = "User(6)"]
fn english_auction_no_bid_cancel_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 2000);
}

#[test]
#[should_panic = "User(7)"]
fn english_auction_bid_late_cancel_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 3000);
}

// Deploying an auction with neither ENGLISH nor DUTCH format results in User(8) error
#[test]
#[should_panic = "User(8)"]
fn auction_unknown_format_test() {
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
    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(hash),
        "format"=> "WOLOLO",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => 1,
        "cancellation_time" => 2,
        "end_time" => 3,
    };
    let session_code = Code::from("casper-private-auction-installer.wasm");
    let session = SessionBuilder::new(session_code, auction_args)
        .with_address(admin)
        .with_authorization_keys(&[admin])
        .with_block_time(0)
        .build();
    context.run(session);
}

// Any combination of bad prices on auction deployment returns User(10)
#[test]
#[should_panic = "User(10)"]
fn dutch_auction_no_starting_price_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_starting_price(None);
    auction_args.set_dutch();
    let mut auction_contract = auction::AuctionContract::deploy(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
}

#[test]
#[should_panic = "User(11)"]
fn english_auction_bid_early_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now - 1000);
}

// #[test]
fn dutch_auction_bid_finalize_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(false, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
    assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
        auction_contract.get_winning_bid().unwrap()
    );
}

// #[test]
fn dutch_auction_bid_cancel_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(false, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    println!(
        "{:?}",
        auction_contract.get_events(auction_contract.contract_hash)
    );
    assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
        auction_contract.get_winning_bid().unwrap()
    );
}
