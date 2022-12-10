use super::{
    constants::{
        ACTIVE, AUCTION_CONTRACT, AUCTION_NAME, ENTRY_POINT_GRANT_GATEKEEPER,
        KEY_ECP47_CONTRACT_NAME, KEY_ECP78_CONTRACT_NAME, KEY_KYC_CONTRACT_NAME, PURSE_NAME,
        PURSE_NAME_VALUE, RUNTIME_ARG_ADMIN, RUNTIME_ARG_CONTRACT_NAME, RUNTIME_ARG_GATEKEEPER,
        RUNTIME_ARG_NAME_META, RUNTIME_ARG_NAME_NAME, RUNTIME_ARG_NAME_SYMBOL,
        RUNTIME_ARG_RECIPIENT, STATUS, TOKEN_COMISSIONS, TOKEN_ECP47_NAME, TOKEN_ECP47_SYMBOL,
        TOKEN_ECP78_NAME, TOKEN_ECP78_SYMBOL, TOKEN_GAUGES, TOKEN_ID, TOKEN_IDS, TOKEN_KYC_NAME,
        TOKEN_KYC_SYMBOL, TOKEN_META, TOKEN_METAS, TOKEN_WAREHOUSES,
    },
    enums::{TypeAccount, TypeAuction, TypeDeploy, TypeECP},
    structs::AuctionContract,
    utils::{fund_account, get_contracts_name_constants, get_session_file},
};
use casper_engine_test_support::{ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT};
use casper_execution_engine::{
    core::engine_state::ExecuteRequest, storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash,
    runtime_args,
    system::mint::{self, METHOD_MINT},
    ContractHash, ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};

use std::collections::{BTreeMap, HashMap};
use tests::auction_args::AuctionArgsBuilder;

pub const KYC: TypeDeploy = TypeDeploy::Kyc;
pub const MINT: TypeDeploy = TypeDeploy::Mint;
pub const NFT_ECP47: TypeDeploy = TypeDeploy::Nft(TypeECP::ECP47);
pub const NFT_ECP78: TypeDeploy = TypeDeploy::Nft(TypeECP::ECP78);

const GRANT_GATE_KEEPER: TypeDeploy = TypeDeploy::GrantGateKeeper;
const GRANT_BUYER_ALI: TypeDeploy = TypeDeploy::GrantBuyer(TypeAccount::Ali);
const GRANT_BUYER_BOB: TypeDeploy = TypeDeploy::GrantBuyer(TypeAccount::Bob);

impl AuctionContract {
    pub fn deploy(&mut self, type_deploy: TypeDeploy) {
        let admin_account_hash: AccountHash = self.get_admin_account_hash();
        let session_file = get_session_file(type_deploy);
        let args = self.get_runtime_args(type_deploy);
        let request = match type_deploy {
            KYC | TypeDeploy::Nft(_) | TypeDeploy::Auction(_) | TypeDeploy::Bid(_, _) => {
                let account_hash = if let TypeDeploy::Bid(type_account, _) = type_deploy {
                    *self.account_hashes.get(&type_account).unwrap()
                } else {
                    admin_account_hash
                };
                ExecuteRequestBuilder::standard(account_hash, session_file.unwrap(), args)
            }
            MINT => ExecuteRequestBuilder::contract_call_by_hash(
                admin_account_hash,
                *self
                    .contract_hashes
                    .get(&TypeDeploy::Nft(self.type_ecp))
                    .unwrap(),
                METHOD_MINT,
                args,
            ),
            GRANT_GATE_KEEPER | TypeDeploy::GrantBuyer(_) => {
                ExecuteRequestBuilder::versioned_contract_call_by_hash(
                    admin_account_hash,
                    *self.package_hashes.get(&KYC).unwrap(),
                    None,
                    if type_deploy == GRANT_GATE_KEEPER {
                        ENTRY_POINT_GRANT_GATEKEEPER
                    } else {
                        METHOD_MINT
                    },
                    args,
                )
            }
            _ => unimplemented!(),
        };
        self.exec_request(request.build());
        let contract_hash_tuple = self.get_contract_hash_from_named_key(type_deploy);
        // contract_call_by_hash have no contract hash back
        let (Some(contract_hash), Some(package_hash)) = contract_hash_tuple
        else {
            //dbg!(format!("no contract_hash || package_hash for {type_deploy}"));
            return
        };
        self.contract_hashes.insert(type_deploy, contract_hash);
        self.package_hashes.insert(type_deploy, package_hash);
    }

    pub fn deploy_contracts(&mut self, type_auction: TypeAuction, type_ecp: TypeECP) {
        self.type_auction = type_auction;
        self.type_ecp = type_ecp;
        self.deploy(KYC);
        self.deploy(GRANT_GATE_KEEPER);
        match type_ecp {
            TypeECP::ECP47 => self.deploy(NFT_ECP47),
            TypeECP::ECP78 => self.deploy(NFT_ECP78),
        }
        self.deploy(MINT);
        self.deploy(GRANT_BUYER_ALI);
        self.deploy(GRANT_BUYER_BOB);
        self.deploy(TypeDeploy::Auction(type_auction));
        //dbg!(&self);
    }

    fn exec_request(&mut self, exec_request: ExecuteRequest) {
        self.builder.exec(exec_request).expect_success().commit();
    }

    fn get_account_hashes(&mut self) -> HashMap<TypeAccount, AccountHash> {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

        let admin = PublicKey::from(&admin_secret).to_account_hash();
        let ali = PublicKey::from(&ali_secret).to_account_hash();
        let bob = PublicKey::from(&bob_secret).to_account_hash();

        self.exec_request(fund_account(&admin));
        self.exec_request(fund_account(&ali));
        self.exec_request(fund_account(&bob));

        HashMap::from([
            (TypeAccount::Admin, admin),
            (TypeAccount::Ali, ali),
            (TypeAccount::Bob, bob),
        ])
    }

    fn get_admin_account_hash(&self) -> AccountHash {
        *self.account_hashes.get(&TypeAccount::Admin).unwrap()
    }

    fn get_auction_runtime_args(&self, type_deploy: TypeDeploy) -> RuntimeArgs {
        let admin_account_hash: AccountHash = self.get_admin_account_hash();
        let english = type_deploy == TypeDeploy::Auction(TypeAuction::English);
        //let now = AuctionArgsBuilder::get_now_u64();
        let mut auction_args = AuctionArgsBuilder::default();
        if !english {
            auction_args.set_dutch();
        }
        auction_args.set_name(AUCTION_NAME);
        auction_args.set_start_time(0);
        auction_args.set_reserve_price(U512::from(500_u16));
        auction_args.set_starting_price(
            if type_deploy == TypeDeploy::Auction(TypeAuction::Dutch) {
                Some(U512::from(500_u16))
            } else {
                None
            },
        );
        // auction_args.set_cancellation_time(3000);
        // auction_args.set_end_time(now);
        auction_args.set_beneficiary(&admin_account_hash);
        auction_args.set_token_contract_hash(
            self.package_hashes
                .get(&TypeDeploy::Nft(self.type_ecp))
                .unwrap(),
        );
        auction_args.set_kyc_package_hash(self.package_hashes.get(&KYC).unwrap());
        auction_args.set_token_id(TOKEN_ECP47_NAME);
        //dbg!(&auction_args);
        auction_args.build()
    }

    fn get_call_args(&self, type_deploy: TypeDeploy) -> Option<RuntimeArgs> {
        let runtime_args = match type_deploy {
            TypeDeploy::GrantBuyer(type_account) => {
                let recipient_account_hash = *self.account_hashes.get(&type_account).unwrap();
                let mut token_meta = BTreeMap::new();

                // This precise meta value is compulsory
                token_meta.insert(STATUS.to_string(), ACTIVE.to_string());

                Some(runtime_args! {
                    RUNTIME_ARG_RECIPIENT => Key::Account(recipient_account_hash),
                    TOKEN_ID => Some(format!("{TOKEN_KYC_NAME}_{type_deploy}")),
                    TOKEN_META => token_meta
                })
            }
            TypeDeploy::Bid(_, amount) => Some(runtime_args! {
                ARG_AMOUNT => U512::from(amount),
                PURSE_NAME => PURSE_NAME_VALUE,
                AUCTION_CONTRACT => *self.contract_hashes.get(&TypeDeploy::Auction(self.type_auction)).unwrap(),
            }),
            _ => None,
        };
        runtime_args
    }

    fn get_contract_hash_from_named_key(
        &self,
        type_deploy: TypeDeploy,
    ) -> (Option<ContractHash>, Option<ContractPackageHash>) {
        let (contract_hash, package_hash) =
            if let Some(KYC | TypeDeploy::Nft(_) | TypeDeploy::Auction(_)) = Some(type_deploy) {
                let account = self
                    .builder
                    .get_expected_account(self.account_hashes[&TypeAccount::Admin]);
                let (contract_hash_name, package_hash_name) =
                    get_contracts_name_constants(type_deploy);
                let contract_hash = account
                    .named_keys()
                    .get(contract_hash_name)
                    .expect("must have contract hash key as part of contract creation")
                    .into_hash()
                    .map(ContractHash::new)
                    .expect("must be contract hash");
                let package_hash = account
                    .named_keys()
                    .get(package_hash_name)
                    .expect("must have package hash key as part of contract creation")
                    .into_hash()
                    .map(ContractPackageHash::new)
                    .expect("must be contract hash");
                (Some(contract_hash), Some(package_hash))
            } else {
                (None, None)
            };
        (contract_hash, package_hash)
    }

    fn get_runtime_args(&self, type_deploy: TypeDeploy) -> RuntimeArgs {
        let admin_account_hash: AccountHash = self.get_admin_account_hash();
        match type_deploy {
            KYC => runtime_args! {
                RUNTIME_ARG_NAME_NAME => TOKEN_KYC_NAME,
                RUNTIME_ARG_NAME_SYMBOL => TOKEN_KYC_SYMBOL,
                RUNTIME_ARG_NAME_META => "",
                RUNTIME_ARG_ADMIN => Key::Account(admin_account_hash),
                RUNTIME_ARG_CONTRACT_NAME => KEY_KYC_CONTRACT_NAME
            },
            GRANT_GATE_KEEPER => runtime_args! {
                RUNTIME_ARG_GATEKEEPER => Key::Account(admin_account_hash)
            },
            NFT_ECP47 => runtime_args! {
                RUNTIME_ARG_NAME_NAME => TOKEN_ECP47_NAME,
                RUNTIME_ARG_NAME_SYMBOL => TOKEN_ECP47_SYMBOL,
                RUNTIME_ARG_NAME_META => "",
                RUNTIME_ARG_ADMIN => Key::Account(admin_account_hash),
                RUNTIME_ARG_CONTRACT_NAME => KEY_ECP47_CONTRACT_NAME
            },
            NFT_ECP78 => runtime_args! {
                RUNTIME_ARG_NAME_NAME => TOKEN_ECP78_NAME,
                RUNTIME_ARG_NAME_SYMBOL => TOKEN_ECP78_SYMBOL,
                RUNTIME_ARG_NAME_META => "",
                RUNTIME_ARG_ADMIN => Key::Account(admin_account_hash),
                RUNTIME_ARG_CONTRACT_NAME => KEY_ECP78_CONTRACT_NAME
            },
            MINT => runtime_args! {
                RUNTIME_ARG_RECIPIENT => Key::Account(admin_account_hash),
                TOKEN_IDS => Some(vec![TOKEN_ECP47_NAME]),
                TOKEN_METAS => vec![""],
                TOKEN_GAUGES => vec![""],
                TOKEN_WAREHOUSES => vec![""],
                TOKEN_COMISSIONS => vec![""],
            },
            TypeDeploy::GrantBuyer(_) | TypeDeploy::Bid(_, _) => {
                self.get_call_args(type_deploy).unwrap()
            }
            TypeDeploy::Auction(_) => self.get_auction_runtime_args(type_deploy),
            _ => unimplemented!(),
        }
    }

    pub fn new(builder: WasmTestBuilder<InMemoryGlobalState>) -> Self {
        let contract_hashes = HashMap::new();
        let package_hashes = HashMap::new();
        let account_hashes = HashMap::new();
        let mut test_auction = Self {
            builder,
            contract_hashes,
            type_auction: TypeAuction::English,
            type_ecp: TypeECP::ECP47,
            account_hashes,
            package_hashes,
        };
        test_auction.account_hashes = test_auction.get_account_hashes();
        test_auction
    }
}
