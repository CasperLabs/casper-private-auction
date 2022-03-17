# English Auction smart-contract
Implementation of an english auction of casper-cep47-nft tokens written in `rust` for Casper blockchain.
In and english auction the highest bid at the times of auction close wins, until then anyone can place a higher bid.
In this implementation the highest 10 bids are kept at a time, if your bid is "knocked off" by having 10 higher bids, your motes are returned to you account.
If you already have a bid in the auction you are free to increase your bid. You can also cancel your bid at any time.

## Entrypoints

1. `bid` 
    - applies a bid to the auction. If the bid is above curve, and the auction is live, the bid wins and the token is transferred to the bidder while the bid motes are transferred to the appropriate parties. If the bid is below curve the call is reverted.
    - parameters:
        - `bid` : `U512`
        - `bid_purse` : purse holding `URef` with `write` access 
2. `cancel_auction`
    - can be called by the token owner. Returns the token to the owner and closes the auction.
3. `cancel_bid`
    - cancels your bid and returns your bid motes to your account.
4. `finalize`
    - manually finalizes the contract. Sends the token to the winner, and the motes to the appropriate parties. In case there were no winners, returns to token to the original owner.
    - If you are here from the dutch auction or are going there, this is automatically done in the dutch auction when a winning bid is placed. This cannot be done in the english token since it is only active when the contract is called and so it is unable to be closed automatically based on time.

## Installation arguments

- `beneficiary_account`: `AccountHash` - account address where all non-allocated motes will be sent after the auction
- `token_contract_hash`: `ContractPackageHash` - of the nft token contract
- `kyc_package_hash`: `ContractPackageHash` - of the used kyc contract*
- `reserve_price`: `U512` - lowest price that is acceptable in the auction (starting price)
- `token_id`: `String` - id of the nft token put up for auction
- `start_time`: `u64` - unix timestamp from whence bid may be made
- `cancellation_time`: `u64` - unix timestamp before which bids may be cancelled (cancelled bid motes are returned to the `main_purse` of the account of the bidder)
- `end_time`: `u64` - designated unix timestamp from when the auction becomes unbiddable
- `name`: `String` - name for the deployed auction to differentiate between concurrent instances
- `starting_price`: `U512` - value where the price curve starts at the beginning of the auction
- `bidder_count_cap`: `u64` - (optional) the number of distinct bidders that are accounted for in the auction. Any more above this will result in the automatic cancellation of the lowest bidder
- `auction_timer_extension`: `u64` - (optional) number of milliseconds the auction is extended by when a new bid is made (countermeasure against auction snipes)
- `minimum_bid_step`: `U512` - (optional) number of motes a bidder has to overbid the current highest amount

### *kyc contract

For some countries and some items that might be on sale there is an obligation to use a KYC (know-your-customer) service provider.
If your use-case does not need this, you are free to remove the relevant call from the auction. 
The compatibility with a kyc contract only has a single requirement from the auction side.
The kyc contract needs to provide an entrypoint as such:
```
EntryPoint::new(
    "is_kyc_proved",
    vec![
        Parameter::new("account", Key::cl_type()),
        Parameter::new("index", CLType::Option(Box::new(U256::cl_type()))),
    ],
    CLType::Bool,
    EntryPointAccess::Public,
    EntryPointType::Contract,
)
```