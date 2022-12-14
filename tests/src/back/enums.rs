use std::fmt;

#[non_exhaustive]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeAccount {
    Admin,
    Ali,
    Bob,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeAuction {
    English,
    Dutch,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeECP {
    ECP47,
    ECP78,
}

#[non_exhaustive]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeDeploy {
    Kyc,
    GrantGateKeeper,
    GrantBuyer(TypeAccount),
    Nft(TypeECP),
    Mint(TypeECP),
    Auction(TypeAuction),
    Bid(TypeAccount, u16),
}

impl fmt::Display for TypeDeploy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
