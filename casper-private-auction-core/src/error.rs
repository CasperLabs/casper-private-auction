use casper_types::ApiError;

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
    InvalidcommissionProperty = 14,
    CommissionAccountIncorrectSerialization = 15,
    CommissionRateIncorrectSerialization = 16,
    CommissionTooManyShares = 17,
    KYCError = 18,
}

impl From<AuctionError> for ApiError {
    fn from(err: AuctionError) -> ApiError {
        ApiError::User(err as u16)
    }
}
