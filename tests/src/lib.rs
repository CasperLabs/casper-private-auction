#![allow(unused)]
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use casper_contract::contract_api::runtime;
use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, system::mint, CLTyped, ContractHash,
    ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};
use utils::{deploy, query, DeploySource};

use crate::auction_args::AuctionArgsBuilder;

type Hash = [u8; 32];

pub mod auction;
pub mod auction_args;
pub mod nft;
pub mod utils;

#[test]
fn english_auction_bid_finalize_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
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
    assert_eq!(auction_contract.ali, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(30000),
        auction_contract.get_winning_bid().unwrap()
    );
}
#[test]
fn dutch_auction_bid_finalize_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_starting_price(Some(U512::from(40000)));
    auction_args.set_dutch();
    let mut auction_contract = auction::AuctionContract::deploy(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    assert!(auction_contract.is_finalized());
    assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
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

// User error 1 happens if not the correct user is trying to interact with the auction.
// More precisely, if a) the bidder is a contract. b) someone other than a stored contact is trying to transfer out the auctioned token

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
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(1), now + 1000);
}

#[test]
#[should_panic = "User(3)"]
fn dutch_auction_bid_too_low_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_starting_price(Some(U512::from(40000)));
    auction_args.set_dutch();
    let mut auction_contract = auction::AuctionContract::deploy(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
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

// Fails with BadState (User(5)) error since on bidding the contract notices that it was already finalized.
// User(5) might also be either that the auction managed to be finalized before expiring, or Dutch contract was initialized without starting price.
#[test]
#[should_panic = "User(5)"]
fn dutch_auction_already_taken_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_starting_price(Some(U512::from(40000)));
    auction_args.set_dutch();
    let mut auction_contract = auction::AuctionContract::deploy(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1001);
}

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
    let mut commissions = BTreeMap::new();
    cep47.mint(
        &Key::Account(cep47.admin),
        &token_id,
        &token_meta,
        &(cep47.admin.clone()),
        commissions,
    );

    let nft::CasperCEP47Contract {
        mut builder,
        nft_hash,
        kyc_hash,
        kyc_package_hash,
        nft_package,
        admin,
        ali,
        bob,
    } = cep47;
    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package),
        "kyc_package_hash" => Key::Hash(kyc_package_hash),
        "format"=> "WOLOLO",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => 1,
        "cancellation_time" => 2,
        "end_time" => 3,
        "name" => "test"
    };

    let auction_code = PathBuf::from("casper-private-auction-installer.wasm");
    deploy(
        &mut builder,
        &admin,
        &DeploySource::Code(auction_code),
        auction_args,
        true,
        Some(0),
    );
}

// Deploying with wrong times reverts with User(9) error
#[test]
#[should_panic = "User(9)"]
fn auction_bad_times_test() {
    let mut cep47 = nft::CasperCEP47Contract::deploy();
    let token_id = String::from("custom_token_id");
    let token_meta = nft::meta::red_dragon();
    let mut commissions = BTreeMap::new();
    cep47.mint(
        &Key::Account(cep47.admin),
        &token_id,
        &token_meta,
        &(cep47.admin.clone()),
        commissions,
    );
    let nft::CasperCEP47Contract {
        mut builder,
        nft_hash,
        kyc_hash,
        kyc_package_hash,
        nft_package,
        admin,
        ali,
        bob,
    } = cep47;
    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package),
        "kyc_package_hash" => Key::Hash(kyc_package_hash),
        "format"=> "ENGLISH",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => 1000_u64,
        "cancellation_time" => 20_u64,
        "end_time" => 11_u64,
        "name" => "test"
    };
    let auction_code = PathBuf::from("casper-private-auction-installer.wasm");
    deploy(
        &mut builder,
        &admin,
        &DeploySource::Code(auction_code),
        auction_args,
        true,
        Some(0),
    );
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
}

#[test]
#[should_panic = "User(11)"]
fn english_auction_bid_early_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now - 1000);
}

// Deploying with wrong times reverts with User(9) error
#[test]
#[should_panic = "User(18)"]
fn auction_bid_no_kyc_token_test() {
    let mut cep47 = nft::CasperCEP47Contract::deploy();
    let token_id = String::from("custom_token_id");
    let token_meta = nft::meta::red_dragon();
    let mut commissions = BTreeMap::new();
    cep47.mint(
        &Key::Account(cep47.admin),
        &token_id,
        &token_meta,
        &(cep47.admin.clone()),
        commissions,
    );
    let nft::CasperCEP47Contract {
        mut builder,
        nft_hash,
        kyc_hash,
        kyc_package_hash,
        nft_package,
        admin,
        ali,
        bob,
    } = cep47;
    let now: u64 = AuctionArgsBuilder::get_now_u64();
    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package),
        "kyc_package_hash" => Key::Hash(kyc_package_hash),
        "format"=> "ENGLISH",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => now+500,
        "cancellation_time" => now+3500,
        "end_time" => now+4000,
        "name" => "test"
    };
    //deploy auction
    let auction_code = PathBuf::from("casper-private-auction-installer.wasm");
    deploy(
        &mut builder,
        &admin,
        &DeploySource::Code(auction_code),
        auction_args,
        true,
        Some(0),
    );
    let contract_hash: Hash = query(
        &builder,
        Key::Account(admin),
        &["test_auction_contract_hash_wrapped".to_string()],
    );
    //bid
    let session_code = PathBuf::from("bid-purse.wasm");
    deploy(
        &mut builder,
        &admin,
        &DeploySource::Code(session_code),
        runtime_args! {
            "bid" => U512::from(40000),
            "auction_contract" => contract_hash
        },
        true,
        Some(now + 1500),
    );
}
