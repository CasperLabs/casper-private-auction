mod mods;

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic = "User(19)"]
    fn ecp47_english_bids() {
        let mut test_auction = AuctionContract::default();
        test_auction.deploy_contracts(TypeAuction::English, TypeECP::ECP47);
        const BID_ENGLISH_BUYER_ALI: TypeDeploy = TypeDeploy::Bid(TypeAccount::Ali, 400_u16);
        const BID_ENGLISH_BUYER_BOB: TypeDeploy = TypeDeploy::Bid(TypeAccount::Bob, 600_u16);
        test_auction.deploy(BID_ENGLISH_BUYER_BOB);
        test_auction.deploy(BID_ENGLISH_BUYER_ALI);
    }

    #[test]
    fn ecp47_dutch_bid() {
        let mut test_auction = AuctionContract::default();
        test_auction.deploy_contracts(TypeAuction::Dutch, TypeECP::ECP47);
        const BID_ENGLISH_BUYER_BOB: TypeDeploy = TypeDeploy::Bid(TypeAccount::Bob, 800_u16);
        test_auction.deploy(BID_ENGLISH_BUYER_BOB);
    }

    #[test]
    fn ecp78_english_bids() {
        let mut test_auction = AuctionContract::default();
        test_auction.deploy_contracts(TypeAuction::English, TypeECP::ECP78);
        const BID_ENGLISH_BUYER_BOB: TypeDeploy = TypeDeploy::Bid(TypeAccount::Bob, 600_u16);
        //test_auction.deploy(BID_ENGLISH_BUYER_BOB);
    }

    #[test]
    fn ecp78_dutch_bid() {
        let mut test_auction = AuctionContract::default();
        test_auction.deploy_contracts(TypeAuction::Dutch, TypeECP::ECP78);
        const BID_ENGLISH_BUYER_BOB: TypeDeploy = TypeDeploy::Bid(TypeAccount::Bob, 800_u16);
        test_auction.deploy(BID_ENGLISH_BUYER_BOB);
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
