#![allow(unused)]
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use auction::AuctionContract;
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
use maplit::btreemap;
use utils::{deploy, fund_account, query, DeploySource};

use crate::auction_args::AuctionArgsBuilder;

pub mod auction;
pub mod auction_args;
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
fn english_auction_bid_cancel_only_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    assert!(now < auction_contract.get_end());
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 3);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    assert!(auction_contract.get_winner().is_none());
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
    assert!(auction_contract.get_winner().is_some());
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
    let mut auction_contract = auction::AuctionContract::deploy_contracts(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    assert!(auction_contract.is_finalized());
    assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
        auction_contract.get_winning_bid().unwrap()
    );
}

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
#[should_panic = "User(19)"]
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
    let mut auction_contract = auction::AuctionContract::deploy_contracts(auction_args);
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
    let mut auction_contract = auction::AuctionContract::deploy_contracts(auction_args);
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
    let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

    let admin_pk: PublicKey = (&admin_secret).into();
    let admin = admin_pk.to_account_hash();
    let ali_pk: PublicKey = (&ali_secret).into();
    let ali = ali_pk.to_account_hash();
    let bob_pk: PublicKey = (&bob_secret).into();
    let bob = bob_pk.to_account_hash();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
    builder.exec(fund_account(&admin)).expect_success().commit();
    builder.exec(fund_account(&ali)).expect_success().commit();
    builder.exec(fund_account(&bob)).expect_success().commit();

    let (kyc_hash, kyc_package) = AuctionContract::deploy_kyc(&mut builder, &admin);

    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &admin);
    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &ali);
    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &bob);

    let (nft_hash, nft_package) = AuctionContract::deploy_nft(&mut builder, &admin, kyc_package);
    let token_id = String::from("custom_token_id");

    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package.value()),
        "kyc_package_hash" => Key::Hash(kyc_package.value()),
        "format"=> "WOLOLO",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => 1,
        "cancellation_time" => 2,
        "end_time" => 3,
        "name" => "test",
        "bidder_count_cap" => Some(10_u64)
    };

    let (auction_hash, auction_package) =
        AuctionContract::deploy_auction(&mut builder, &admin, auction_args);
}

// Deploying with wrong times reverts with User(9) error
#[test]
#[should_panic = "User(9)"]
fn auction_bad_times_test() {
    let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

    let admin_pk: PublicKey = (&admin_secret).into();
    let admin = admin_pk.to_account_hash();
    let ali_pk: PublicKey = (&ali_secret).into();
    let ali = ali_pk.to_account_hash();
    let bob_pk: PublicKey = (&bob_secret).into();
    let bob = bob_pk.to_account_hash();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
    builder.exec(fund_account(&admin)).expect_success().commit();
    builder.exec(fund_account(&ali)).expect_success().commit();
    builder.exec(fund_account(&bob)).expect_success().commit();

    let (kyc_hash, kyc_package) = AuctionContract::deploy_kyc(&mut builder, &admin);

    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &admin);
    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &ali);
    AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &bob);

    let (nft_hash, nft_package) = AuctionContract::deploy_nft(&mut builder, &admin, kyc_package);
    let token_id = String::from("custom_token_id");

    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package.value()),
        "kyc_package_hash" => Key::Hash(kyc_package.value()),
        "format"=> "ENGLISH",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => 1000_u64,
        "cancellation_time" => 20_u64,
        "end_time" => 11_u64,
        "name" => "test",
        "bidder_count_cap" => Some(10_u64)
    };

    let (auction_hash, auction_package) =
        AuctionContract::deploy_auction(&mut builder, &admin, auction_args);
}

// Any combination of bad prices on auction deployment returns User(10)
#[test]
#[should_panic = "User(10)"]
fn dutch_auction_no_starting_price_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_starting_price(None);
    auction_args.set_dutch();
    let mut auction_contract = auction::AuctionContract::deploy_contracts(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
}

#[test]
#[should_panic = "User(11)"]
fn english_auction_bid_early_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now - 1000);
}

#[test]
#[should_panic = "User(4)"]
fn auction_bid_no_kyc_token_test() {
    let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

    let admin_pk: PublicKey = (&admin_secret).into();
    let admin = admin_pk.to_account_hash();
    let ali_pk: PublicKey = (&ali_secret).into();
    let ali = ali_pk.to_account_hash();
    let bob_pk: PublicKey = (&bob_secret).into();
    let bob = bob_pk.to_account_hash();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
    builder.exec(fund_account(&admin)).expect_success().commit();
    builder.exec(fund_account(&ali)).expect_success().commit();
    builder.exec(fund_account(&bob)).expect_success().commit();

    let (kyc_hash, kyc_package) = AuctionContract::deploy_kyc(&mut builder, &admin);

    let (nft_hash, nft_package) = AuctionContract::deploy_nft(&mut builder, &admin, kyc_package);
    let token_id = String::from("custom_token_id");

    let now: u64 = AuctionArgsBuilder::get_now_u64();
    let auction_args = runtime_args! {
        "beneficiary_account"=>Key::Account(admin),
        "token_contract_hash"=>Key::Hash(nft_package.value()),
        "kyc_package_hash" => Key::Hash(kyc_package.value()),
        "format"=> "ENGLISH",
        "starting_price"=> None::<U512>,
        "reserve_price"=>U512::from(300),
        "token_id"=>token_id,
        "start_time" => now+500,
        "cancellation_time" => now+3500,
        "end_time" => now+4000,
        "name" => "test",
        "bidder_count_cap" => Some(10_u64)
    };

    let (auction_hash, auction_package) =
        AuctionContract::deploy_auction(&mut builder, &admin, auction_args);
    //bid
    let session_code = PathBuf::from("bid-purse.wasm");
    deploy(
        &mut builder,
        &admin,
        &DeploySource::Code(session_code),
        runtime_args! {
            "bid" => U512::from(40000),
            "auction_contract" => auction_hash
        },
        true,
        Some(now + 1500),
    );
}

#[test]
fn cancel_auction_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1001)
}

#[test]
#[should_panic = "User(22)"]
fn cancel_auction_after_bid_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1001)
}

#[test]
fn cancel_auction_after_cancelled_bid_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_contract = auction::AuctionContract::deploy_with_default_args(true, now);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 1001);
    auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1002)
}

#[test]
#[should_panic = "User(6)"]
fn english_auction_bidder_count_limit_test() {
    let now = auction_args::AuctionArgsBuilder::get_now_u64();
    let mut auction_args = auction_args::AuctionArgsBuilder::default();
    auction_args.set_bidder_count_cap(Some(1));
    let mut auction_contract = auction::AuctionContract::deploy_contracts(auction_args);
    auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
    auction_contract.bid(&auction_contract.ali.clone(), U512::from(40000), now + 1001);
    auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 1002);
    auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
    assert!(auction_contract.is_finalized());
    assert_eq!(auction_contract.ali, auction_contract.get_winner().unwrap());
    assert_eq!(
        U512::from(40000),
        auction_contract.get_winning_bid().unwrap()
    );
}
