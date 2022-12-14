beneficiary_account: Key::Account(AccountHash), account address where all cspr motes will go that were not distributed as commissions
token_contract_hash: Key::Hash(ContractPackageHash),
kyc_package_hash: Key::Hash(ContractPackageHash),
format: either `ENGLISH` or `DUTCH`
starting_price: Option<U512>, None if format is `ENGLISH`
reserve_price: U512,
token_id: String,
start_time: u64, Unix timestamp
cancellation_time: u64, Unix timestamp
end_time: u64, Unix timestamp
name: String, name of this particular account
bidder_count_cap: Option<u64>, argument to limit the number of distinct bidder.
auction_timer_extension: Option<u64>, on successful bids extends the end and cancellation times of the auction.
minimum_bid_step: Option<U512>, if a value is given the next successful bid needs to be at least step higher than the previous.
marketplace_account: AccountHash,
marketplace_commission: u32,
