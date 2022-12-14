use std::{collections::HashMap, fmt};

use casper_engine_test_support::WasmTestBuilder;
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{account::AccountHash, ContractHash, ContractPackageHash};

use super::{
    enums::{TypeAccount, TypeAuction, TypeDeploy, TypeECP},
    test_auction::{KYC, NFT_ECP47, NFT_ECP78},
    utils::get_contracts_name_constants,
};

pub struct AuctionContract {
    pub builder: WasmTestBuilder<InMemoryGlobalState>,
    pub type_auction: TypeAuction,
    pub type_ecp: TypeECP,
    pub contract_hashes: HashMap<TypeDeploy, ContractHash>,
    pub package_hashes: HashMap<TypeDeploy, ContractPackageHash>,
    pub account_hashes: HashMap<TypeAccount, AccountHash>,
}

// Test debug
impl fmt::Debug for AuctionContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuctionContract")
            .field(
                get_contracts_name_constants(KYC).0,
                &self.contract_hashes.get(&KYC),
            )
            .field(
                get_contracts_name_constants(NFT_ECP47).0,
                &self.contract_hashes.get(&NFT_ECP47),
            )
            .field(
                get_contracts_name_constants(KYC).0,
                &self.package_hashes.get(&KYC),
            )
            .field(
                get_contracts_name_constants(NFT_ECP47).0,
                &self.package_hashes.get(&NFT_ECP47),
            )
            .field(
                get_contracts_name_constants(NFT_ECP78).0,
                &self.contract_hashes.get(&NFT_ECP78),
            )
            .field(
                get_contracts_name_constants(NFT_ECP78).0,
                &self.package_hashes.get(&NFT_ECP78),
            )
            .field(
                get_contracts_name_constants(TypeDeploy::Auction(TypeAuction::English)).0,
                &self
                    .contract_hashes
                    .get(&TypeDeploy::Auction(TypeAuction::English)),
            )
            .field(
                get_contracts_name_constants(TypeDeploy::Auction(TypeAuction::English)).0,
                &self
                    .package_hashes
                    .get(&TypeDeploy::Auction(TypeAuction::English)),
            )
            .field(
                get_contracts_name_constants(TypeDeploy::Auction(TypeAuction::Dutch)).0,
                &self
                    .contract_hashes
                    .get(&TypeDeploy::Auction(TypeAuction::Dutch)),
            )
            .field(
                get_contracts_name_constants(TypeDeploy::Auction(TypeAuction::Dutch)).0,
                &self
                    .package_hashes
                    .get(&TypeDeploy::Auction(TypeAuction::Dutch)),
            )
            .field(
                "Admin",
                &self.account_hashes.get(&TypeAccount::Admin).unwrap(),
            )
            .field("Ali", &self.account_hashes.get(&TypeAccount::Ali).unwrap())
            .field("Bob", &self.account_hashes.get(&TypeAccount::Bob).unwrap())
            .finish()
    }
}
