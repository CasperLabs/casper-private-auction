# casper-private-auction

## Preparation

The Auction Contract requires to additional contracts to function.

- An NFT contract with

  - a `"transfer"` endpoint that has the following arguments:
    - `"sender"`
    - `"recipient"`
    - `"token_ids"`
  - a `"token_commission"` endpoint with the following arguments:
    - `"token_id"`
    - `"property"`

- A KYC contract with
  - `"is_kyc_proved"` endpoint that has the arguments:
    - `"account"`
    - `"index"`

Upon deploying the Auction Contract you need to supply the `contract package hash` of the two listed (NFT, KYC) and deployed contracts.

## Auction Contract Deployment Arguments

- `"beneficiary_account"`: Key-account type of the account you wish to receive any motes that weren't distributed after commissions were paid
- `"token_contract_hash"`: Key-hash type of the contract package hash of the NFT contract
- `"kyc_package_hash"`: Key-hash type of the contract package hash of the KYC contract
- `"format"`: String/Text data to tell the contract to run an english or a dutch auction (only ENGLISH or DUTCH are valid data (capitalization is necessary))
- `"starting_price"`: Option<U512> type. Dutch auction starting price. English auction doesn't use this so it requires this to be None or will fail to deploy.
- `"reserve_price"`: U512 type of reserve price aka smallest permitted selling price.
- `"token_id"`: String/Text id of the NFT token put up for auction.
- `"start_time"`: u64 UNIX timestamp of auctions starting time.
- `"cancellation_time"`: u64 UNIX timestamp of the latest time bids on the auction can be cancelled.
- `"end_time"`: u64 UNIX timestamp of the time the auction will end.

## Usage

The three possible endpoints of the contract are:

- `"bid"`: self-explanatory.
- `"cancel_bid"`: anyone can cancel their bid in an english auction until the cancellation deadline. Not usable with dutch auctions, as with those the first valid bid wins immidiately.
- `"finalize"`: called after the end of an english auction to finish up the auction the distribute motes and token according to result. Dutch auctions finalize themselves upon the first valid bid.

Dutch auctions will end immidiately when a valid bid has been made.
English auctions will run their course and must be finalized after the end time to distribute motes and the NFT token.

When the auction end
a) there were no winning bids or an error has occured, every bid will be returned to the bidder, and the NFT token to the owner.
b) the auction was won by a bidder and as such they will receive the NFT token. Bids that did not win will be returned to their respective bidders.
The winning bid will be sliced into bid/1000 pieces and these pieces will be forwarded to those listed in the NFT tokens commission meta data. All remaining motes will be sent to the beneficiary account provided by the auctions deployer.

## Make commands

### prepare

Adds wasm to the cargo compilation targets.

### build-contract

Builds the contracts using the nightly toolchain with wasm as the compilation target.

### test-only

Run all tests inside the workspace.

### copy-wasm-file-to-test

Copies the `.wasm` files into `/tests/wasm` folder, where the test framework is set to look for them.

### test

Executes the `build-contract` and `copy-wasm-file-to-test`, then `test-only` commands.

### clippy

Executes the clippy linter on the contract and test codes.

### lint

Runs `clippy` and `format`

### check-lint

Runs `clippy` then executes `fmt` with `--check` enabled. Gives errors for clippy warnings.

### format

Applies formatting to the codes.

### clean

Artifact removal command. (`.wasm` files, `target` folders)

## Rust version

This contract was compiled and ran during development using `1.55.0-nightly (868c702d0 2021-06-30)`

## Casper contract sdk version

casper-types = "1.3.3"
casper-contract = "1.3.3"
casper-engine-test-support = "1.3.3"

### Date 6 September 2021

### Contact

Smart contracts for private NFT auctions. Please contact Alexander Limonov (alimonov@casperlabs.io) with any questions.
