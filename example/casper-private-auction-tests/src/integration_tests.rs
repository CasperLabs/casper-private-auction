mod mods;

#[cfg(test)]
mod tests {
    use crate::mods::test_auction::AuctionContract;
    use casper_engine_test_support::{InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST};

    #[test]
    fn seller_erc47_token_setup() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        let mut test_auction = AuctionContract::new(builder);
        test_auction.deploy_contracts();
        // assert_eq!(fixture.token_name(), AuctionContract::TOKEN_NAME);
        // assert_eq!(fixture.token_symbol(), AuctionContract::TOKEN_SYMBOL);
        // assert_eq!(fixture.token_decimals(), AuctionContract::TOKEN_DECIMALS);
        // assert_eq!(
        //     fixture.balance_of(Key::from(fixture.ali)),
        //     Some(AuctionContract::token_total_supply())
        // );
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
