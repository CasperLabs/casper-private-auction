NCTL test/demo scripts
======================

These scripts deploy and operate necessary contract to demonstrate and test
operation of private NFT auctions on a running NCTL network.

Assumptions
-----------
- All repos under ~/CasperLabs
- External repo expected: CaskNFT
- running NCTL network with casper-node version 1.3.1 or later
- user 1 with sufficient tokens (normally the case!)
- users 1-5 available
- user 1 always acts as the seller
- `rust-script` installed

Directory structure
-------------------

`nctl-tests/setup`

Contains the critical `client_put_deploy_config.sh` script that sets up the NFT contract and sets relevant variables, must be run first.

`nctl-tests/setup/actions`

Scripts to deploy NFT contracts and auctions.

`nctl-tests/setup/misc` 

Contains the variable setup script `client_put_deploy_config.sh`, invoked in other scripts to guarantee that a certain required variables are correctly set.

`nctl-tests/operation`

Scripts allowing one line interactions with a deployed auction by any of the 5 users.

`nctl-tests/scenarios`

Complex end-to-end testing scenario scripts, deploying NFT contract, auction contract and conducting bidding/finalization.

Typical operation
-------------------

Change directory to `nctl-tests` and invoke 

`. setup/seller_token_setup.sh <token ID>`
 
then the desired scenario in `nctl-tests/scenarios`

Currently, it is recommended to run the setup script before each of the scenarios.