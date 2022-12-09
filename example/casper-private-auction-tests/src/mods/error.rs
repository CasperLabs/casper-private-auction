use casper_types::ApiError;

pub enum PrivateAuctionTestError {
    InvalidContractHash,
    InvalidSessionFile,
}

impl From<PrivateAuctionTestError> for ApiError {
    fn from(code: PrivateAuctionTestError) -> Self {
        ApiError::User(code as u16)
    }
}
