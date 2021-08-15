#![no_std]

use casper_types::account::AccountHash;

extern crate alloc;

pub mod auction;
pub mod error;
#[macro_use]
pub mod data;

pub trait AuctionLogic {
    fn auction_bid();
    fn auction_cancel_bid();
    fn auction_allocate(winner: Option<AccountHash>);
    fn auction_transfer(winner: Option<AccountHash>);
    fn auction_finalize(time_check: bool);
}
