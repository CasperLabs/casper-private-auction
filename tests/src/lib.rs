pub mod mods;

#[cfg(test)]
mod tests {
    use crate::mods::{
        auction::AuctionContract,
        auction_args::AuctionArgsBuilder,
        constants::{
            ARG_AUCTION_CONTRACT, ARG_NAME, AUCTION_TIMER_EXTENSION, BENEFICIARY_ACCOUNT, BID,
            BIDDER_COUNT_CAP, CANCELLATION_TIME, END_TIME, FORMAT, HAS_ENHANCED_NFT,
            KEY_AUCTION_CONTRACT_NAME, KEY_KYC_PACKAGE_HASH, MARKETPLACE_ACCOUNT,
            MARKETPLACE_COMMISSION, MINIMUM_BID_STEP, RESERVE_PRICE, SESSION_BID_PURSE,
            STARTING_PRICE, START_TIME, TOKEN_CONTRACT_HASH, TOKEN_ID,
        },
        utils::{deploy, fund_account, DeploySource},
    };
    use casper_engine_test_support::{InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST};
    use casper_types::{account::AccountHash, runtime_args, Key, RuntimeArgs, U512};
    use std::path::PathBuf;

    #[test]
    fn english_auction_bid_finalize_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
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
    fn english_auction_cancel_only_bid_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        assert!(now < auction_contract.get_end());
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1);
        auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 3);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
        assert!(auction_contract.is_finalized());
        assert!(auction_contract.get_winner().is_none());
    }

    #[test]
    #[should_panic = "User(3)"]
    fn english_auction_bid_cancel_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
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
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_starting_price(Some(U512::from(40000)));
        auction_args.set_dutch();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
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
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 300);
    }

    // User error 1 happens if not the correct user is trying to interact with the auction.
    // More precisely, if a) the bidder is a contract. b) someone other than a stored contact is trying to transfer out the auctioned token

    // Trying to bid after the end of the auction results in User(2) error
    #[test]
    #[should_panic = "User(2)"]
    fn english_auction_bid_too_late_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
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
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(1), now + 1000);
    }

    #[test]
    #[should_panic = "User(3)"]
    fn dutch_auction_bid_too_low_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_starting_price(Some(U512::from(40000)));
        auction_args.set_dutch();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
    }

    // Finalizing after finalizing is User(4) error.
    #[test]
    #[should_panic = "User(4)"]
    fn english_auction_bid_after_finalized_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
        assert!(auction_contract.is_finalized());
        auction_contract.finalize(&auction_contract.admin.clone(), now + 3501);
    }

    // Fails with BadState (User(5)) error since on bidding the contract notices that it was already finalized.
    // User(5) might also be either that the auction managed to be finalized before expiring, or Dutch contract was initialized without starting price.
    #[test]
    #[should_panic = "User(5)"]
    fn dutch_auction_already_taken_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_starting_price(Some(U512::from(40000)));
        auction_args.set_dutch();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1001);
    }

    // User(6) error -> trying to cancel a bid that wasn't placed
    #[test]
    #[should_panic = "User(6)"]
    fn english_auction_no_bid_cancel_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 2000);
    }

    #[test]
    #[should_panic = "User(7)"]
    fn english_auction_bid_late_cancel_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1);
        auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 3000);
    }

    // Deploying an auction with neither ENGLISH nor DUTCH format results in User(8) error
    #[test]
    #[should_panic = "User(8)"]
    fn auction_unknown_format_test() {
        let mut builder = InMemoryWasmTestBuilder::default();
        let (admin, ali, bob) = AuctionContract::get_accounts(&mut builder);
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(fund_account(&admin)).expect_success().commit();
        builder.exec(fund_account(&ali)).expect_success().commit();
        builder.exec(fund_account(&bob)).expect_success().commit();

        let (_, kyc_package) = AuctionContract::deploy_kyc(&mut builder, &admin);

        AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &admin);
        AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &ali);
        AuctionContract::add_kyc(&mut builder, &kyc_package, &admin, &bob);

        let (_, nft_package) = AuctionContract::deploy_nft(&mut builder, &admin);

        let auction_args = runtime_args! {
            BENEFICIARY_ACCOUNT => Key::Account(admin),
            TOKEN_CONTRACT_HASH => Key::Hash(nft_package.value()),
            KEY_KYC_PACKAGE_HASH => Key::Hash(kyc_package.value()),
            FORMAT => "WOLOLO",
            STARTING_PRICE => None::<U512>,
            RESERVE_PRICE => U512::from(300),
            TOKEN_ID => TOKEN_ID,
            START_TIME => 1,
            CANCELLATION_TIME => 2,
            END_TIME => 3,
            ARG_NAME => KEY_AUCTION_CONTRACT_NAME,
            BIDDER_COUNT_CAP => Some(10_u64),
            AUCTION_TIMER_EXTENSION => None::<u64>,
            MINIMUM_BID_STEP => None::<U512>,
            MARKETPLACE_ACCOUNT => AccountHash::new([11_u8; 32]),
            MARKETPLACE_COMMISSION => 75,
            HAS_ENHANCED_NFT => false
        };

        AuctionContract::deploy(&mut builder, &admin, auction_args);
    }

    // Deploying with wrong times reverts with User(9) error
    #[test]
    #[should_panic = "User(9)"]
    fn auction_bad_times_test() {
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_reserve_price(U512::from(300));
        auction_args.set_start_time(1000_u64);
        auction_args.set_cancellation_time(20_u64);
        auction_args.set_end_time(11_u64);
        auction_args.set_bidder_count_cap(Some(10_u64));
        auction_args.set_marketplace_commission(75);
        auction_args.set_token_id(TOKEN_ID);
        AuctionContract::deploy_auction(auction_args);
    }

    // Any combination of bad prices on auction deployment returns User(10)
    #[test]
    #[should_panic = "User(10)"]
    fn dutch_auction_no_starting_price_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_starting_price(None);
        auction_args.set_dutch();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
    }

    #[test]
    #[should_panic = "User(11)"]
    fn english_auction_bid_early_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now - 1000);
    }

    #[test]
    #[should_panic = "User(4)"]
    fn auction_bid_no_kyc_token_test() {
        let mut builder = InMemoryWasmTestBuilder::default();
        let (admin, ali, bob) = AuctionContract::get_accounts(&mut builder);
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(fund_account(&admin)).expect_success().commit();
        builder.exec(fund_account(&ali)).expect_success().commit();
        builder.exec(fund_account(&bob)).expect_success().commit();

        let (_, kyc_package) = AuctionContract::deploy_kyc(&mut builder, &admin);
        let (_, nft_package) = AuctionContract::deploy_nft(&mut builder, &admin);

        let now: u64 = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_beneficiary(&admin);
        auction_args.set_token_contract_hash(&nft_package);
        auction_args.set_kyc_package_hash(&kyc_package);
        auction_args.set_reserve_price(U512::from(300));
        auction_args.set_start_time(now + 500);
        auction_args.set_cancellation_time(now + 3500);
        auction_args.set_end_time(now + 4000);
        auction_args.set_bidder_count_cap(Some(10_u64));
        auction_args.set_marketplace_commission(75);
        auction_args.set_token_id(TOKEN_ID);

        let (auction_hash, _) = AuctionContract::deploy(&mut builder, &admin, auction_args.build());
        //bid
        let session_code = PathBuf::from(SESSION_BID_PURSE);
        deploy(
            &mut builder,
            &admin,
            &DeploySource::Code(session_code),
            runtime_args! {
                BID => U512::from(40000),
                ARG_AUCTION_CONTRACT => auction_hash
            },
            true,
            Some(now + 1500),
        );
    }

    #[test]
    fn cancel_auction_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1001)
    }

    #[test]
    #[should_panic = "User(22)"]
    fn cancel_auction_after_bid_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
        auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1001)
    }

    #[test]
    fn cancel_auction_after_cancelled_bid_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 1000);
        auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 1001);
        auction_contract.cancel_auction(&auction_contract.admin.clone(), now + 1002)
    }

    #[test]
    #[should_panic = "User(6)"]
    fn english_auction_bidder_count_limit_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_bidder_count_cap(Some(1));
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
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

    #[test]
    fn english_increase_time_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_auction_timer_extension(Some(10000));
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        assert_eq!(auction_contract.get_end(), now + 4000);

        auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
        assert_eq!(auction_contract.get_end(), now + 14000);
        auction_contract.cancel_bid(&auction_contract.bob.clone(), now + 12999);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 14000);
        assert!(auction_contract.is_finalized());
        assert_eq!(None, auction_contract.get_winner());
        assert_eq!(None, auction_contract.get_winning_bid());
    }

    #[test]
    fn english_auction_bid_step_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_minimum_bid_step(Some(U512::from(10000)));
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
        auction_contract.bid(&auction_contract.ali.clone(), U512::from(40000), now + 1001);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 4000);
        assert!(auction_contract.is_finalized());
        assert_eq!(Some(auction_contract.ali), auction_contract.get_winner());
        assert_eq!(Some(U512::from(40000)), auction_contract.get_winning_bid());
    }

    #[test]
    #[should_panic = "User(3)"]
    fn english_auction_bid_step_test_failing() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_minimum_bid_step(Some(U512::from(10001)));
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 1000);
        auction_contract.bid(&auction_contract.ali.clone(), U512::from(40000), now + 1001);
    }

    #[test]
    fn marketplace_commission_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let auction_args = AuctionArgsBuilder::default();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(
            &auction_contract.ali.clone(),
            U512::from(100000),
            now + 1000,
        );
        auction_contract.finalize(&auction_contract.admin.clone(), now + 4000);
        assert!(auction_contract.is_finalized());
        assert_eq!(auction_contract.get_marketplace_balance(), U512::from(7500));
        assert!(auction_contract.get_comm_balance() > U512::from(0));
    }

    #[test]
    fn english_auction_bid_extend_finalize_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        assert!(now < auction_contract.get_end());
        auction_contract.extend_bid(&auction_contract.bob.clone(), U512::from(30000), now);
        auction_contract.extend_bid(&auction_contract.bob.clone(), U512::from(10000), now);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
        assert!(auction_contract.is_finalized());
        assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
        assert_eq!(
            U512::from(40000),
            auction_contract.get_winning_bid().unwrap()
        );
    }

    #[test]
    fn english_auction_bid_delta_finalize_test() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_contract = AuctionContract::deploy_auction_with_default_args(true, now);
        let bob = auction_contract.bob;
        assert!(now < auction_contract.get_end());
        dbg!(auction_contract.get_account_balance(&bob));
        auction_contract.delta_bid(&bob, U512::from(5_000_u64), now);
        dbg!(auction_contract.get_account_balance(&bob));
        auction_contract.delta_bid(&bob, U512::from(8_000_u64), now);
        dbg!(auction_contract.get_account_balance(&bob));
        auction_contract.finalize(&auction_contract.admin.clone(), now + 3500);
        assert!(auction_contract.is_finalized());
        assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
        assert_eq!(
            U512::from(8_000_u64),
            auction_contract.get_winning_bid().unwrap()
        );
    }

    #[test]
    fn english_auction_bid_with_enhanced_nft() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_has_enhanced_nft();
        auction_args.set_start_time(now + 500);
        auction_args.set_end_time(5000);
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        assert!(now < auction_contract.get_end());
        auction_contract.bid(&auction_contract.ali.clone(), U512::from(30000), now + 1000);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(40000), now + 2000);
        auction_contract.finalize(&auction_contract.admin.clone(), now + 5500);
        assert!(auction_contract.is_finalized());
        assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
        assert_eq!(
            U512::from(40000),
            auction_contract.get_winning_bid().unwrap()
        );
    }

    #[test]
    fn dutch_auction_bid_with_enhanced_nft() {
        let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        auction_args.set_dutch();
        auction_args.set_starting_price(Some(U512::from(40000)));
        auction_args.set_reserve_price(U512::from(20000));
        auction_args.set_has_enhanced_nft();
        let mut auction_contract = AuctionContract::deploy_auction(auction_args);
        auction_contract.bid(&auction_contract.bob.clone(), U512::from(30000), now + 3000);
        assert!(auction_contract.is_finalized());
        assert_eq!(auction_contract.bob, auction_contract.get_winner().unwrap());
        assert_eq!(
            U512::from(30000),
            auction_contract.get_winning_bid().unwrap()
        );
    }
}
