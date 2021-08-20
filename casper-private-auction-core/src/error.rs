use casper_types::ApiError;

pub enum AuctionError {
    Early = 0,
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

impl From<AuctionError> for ApiError {
    fn from(err: AuctionError) -> ApiError {
        ApiError::User(err as u16)
    }
}
