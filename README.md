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

## Disclaimer

The `nctl-tests` are for a separate version of this codebase where both auctions are implemented in the same contract. These tests have not been adjusted for this scheme where the auctions reside in separate contracts.

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
This contract was compiled and ran during development using `1.59.0-nightly`

## Casper contract sdk version
casper-types = "1.4.3"
casper-contract = "1.4.4"
casper-engine-test-support = "2.0.3"

### Date 17 March 2022