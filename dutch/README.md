# Dutch Auction smart-contract
Implementation of a dutch auction of casper-cep47-nft tokens written in `rust` for Casper blockchain.
Dutch auction in short is a sale where the price is a declining curve from a starting point to an endpoint where the time is step.
The first bid that is placed and is above this curve wins.

The price curve is calculated as follows:

```
let price_range = start_price - end_price;
let duration = end_time - start_time;

let step = price_range / duration;
let time_passed = block_time - start_time;
start_price - (step * time_passed)
```

## Entrypoints

1. `bid` 
    - applies a bid to the auction. If the bid is above curve, and the auction is live, the bid wins and the token is transferred to the bidder while the bid motes are transferred to the appropriate parties. If the bid is below curve the call is reverted.
    - parameters:
        - `bid` : `U512`
        - `bid_purse` : purse holding `URef` with `write` access 
2. `cancel_auction`
    - can be called by the token owner. Returns the token to the owner and closes the auction.

## Installation arguments

name : type - description

- `beneficiary_account`: `AccountHash` - account address where all non-allocated motes will be sent after the auction
- `token_contract_hash`: `ContractPackageHash` - of the nft token contract
- `kyc_package_hash`: `ContractPackageHash` - of the used kyc contract*
- `reserve_price`: `U512` - lowest price on the price curve
- `token_id`: `String` - id of the nft token put up for auction
- `start_time`: `u64` - unix timestamp from whence bid may be made
- `end_time`: `u64` - designated unix timestamp from when the auction becomes unbiddable
- `name`: `String` - name for the deployed auction to differentiate between concurrent instances
- `starting_price`: `U512` - value where the price curve starts at the beginning of the auction

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