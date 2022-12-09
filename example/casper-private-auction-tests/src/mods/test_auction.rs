use crate::mods::constants::AUCTION_NAME;

use super::constants::{
    BID, BID_PURSE, CONTRACT_AUCTION, CONTRACT_ECP47_TOKEN, CONTRACT_KYC,
    ENTRY_POINT_GRANT_GATEKEEPER, ENTRY_POINT_MINT, KEY_AUCTION_CONTRACT_HASH,
    KEY_AUCTION_PACKAGE_HASH, KEY_ECP47_CONTRACT_HASH, KEY_ECP47_CONTRACT_NAME,
    KEY_ECP47_PACKAGE_HASH, KEY_KYC_CONTRACT_HASH, KEY_KYC_CONTRACT_NAME, KEY_KYC__PACKAGE_HASH,
    RUNTIME_ARG_ADMIN, RUNTIME_ARG_CONTRACT_NAME, RUNTIME_ARG_GATEKEEPER, RUNTIME_ARG_NAME_META,
    RUNTIME_ARG_NAME_NAME, RUNTIME_ARG_NAME_SYMBOL, RUNTIME_ARG_RECIPIENT, TOKEN_COMISSIONS,
    TOKEN_ECP47_NAME, TOKEN_ECP47_SYMBOL, TOKEN_GAUGES, TOKEN_ID, TOKEN_IDS, TOKEN_KYC_NAME,
    TOKEN_KYC_SYMBOL, TOKEN_META, TOKEN_METAS, TOKEN_WAREHOUSES,
};
use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_PAYMENT, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::{
    core::engine_state::ExecuteRequest, storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash, runtime_args, system::mint, ContractHash, ContractPackageHash, Key,
    PublicKey, RuntimeArgs, SecretKey, URef, U512,
};
use core::fmt;
use std::collections::HashMap;
use tests::auction_args::AuctionArgsBuilder;

#[non_exhaustive]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeAccount {
    Admin,
    Ali,
    Bob,
}

#[non_exhaustive]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TypeDeploy {
    Kyc,
    GrantGateKeeper,
    GrantBuyer(TypeAccount),
    Nft,
    Mint,
    Auction,
    Bid(TypeAccount, u16),
}

impl fmt::Display for TypeDeploy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

const KYC: TypeDeploy = TypeDeploy::Kyc;
const NFT: TypeDeploy = TypeDeploy::Nft;
const MINT: TypeDeploy = TypeDeploy::Mint;
const AUCTION: TypeDeploy = TypeDeploy::Auction;
const GRANT_GATE_KEEPER: TypeDeploy = TypeDeploy::GrantGateKeeper;
const GRANT_BUYER_ALI: TypeDeploy = TypeDeploy::GrantBuyer(TypeAccount::Ali);
const GRANT_BUYER_BOB: TypeDeploy = TypeDeploy::GrantBuyer(TypeAccount::Bob);
const BID_ENGLISH_BUYER_ALI: TypeDeploy = TypeDeploy::Bid(TypeAccount::Ali, 400_u16);
const BID_ENGLISH_BUYER_BOB: TypeDeploy = TypeDeploy::Bid(TypeAccount::Bob, 600_u16);

pub struct AuctionContract {
    builder: WasmTestBuilder<InMemoryGlobalState>,
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
                &self.contract_hashes.get(&KYC).unwrap(),
            )
            .field(
                get_contracts_name_constants(NFT).0,
                &self.contract_hashes.get(&NFT).unwrap(),
            )
            .field(
                get_contracts_name_constants(KYC).0,
                &self.package_hashes.get(&KYC).unwrap(),
            )
            .field(
                get_contracts_name_constants(NFT).0,
                &self.package_hashes.get(&NFT).unwrap(),
            )
            .field(
                get_contracts_name_constants(AUCTION).0,
                &self.contract_hashes.get(&AUCTION),
            )
            .field(
                get_contracts_name_constants(AUCTION).0,
                &self.package_hashes.get(&AUCTION),
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

impl AuctionContract {
    fn deploy_contract(&mut self, type_deploy: TypeDeploy) {
        let admin_account_hash: AccountHash = self.get_admin_account_hash();
        let session_file = get_session_file(type_deploy);
        let args = self.get_runtime_args(type_deploy);
        let request = match type_deploy {
            KYC | NFT | AUCTION => {
                ExecuteRequestBuilder::standard(admin_account_hash, session_file.unwrap(), args)
            }
            MINT => ExecuteRequestBuilder::contract_call_by_hash(
                admin_account_hash,
                *self.contract_hashes.get(&NFT).unwrap(),
                ENTRY_POINT_MINT,
                args,
            ),
            GRANT_GATE_KEEPER | GRANT_BUYER_ALI | GRANT_BUYER_BOB => {
                let entry_point = if type_deploy == GRANT_GATE_KEEPER {
                    ENTRY_POINT_GRANT_GATEKEEPER
                } else {
                    ENTRY_POINT_MINT
                };
                ExecuteRequestBuilder::contract_call_by_hash(
                    admin_account_hash,
                    *self.contract_hashes.get(&KYC).unwrap(),
                    entry_point,
                    args,
                )
            }
            BID_ENGLISH_BUYER_ALI | BID_ENGLISH_BUYER_BOB => {
                let request = if let TypeDeploy::Bid(type_account, amount) = type_deploy {
                    dbg!(*self.contract_hashes.get(&AUCTION).unwrap());
                    dbg!(*self.account_hashes.get(&type_account).unwrap());
                    Some(ExecuteRequestBuilder::contract_call_by_hash(
                        *self.account_hashes.get(&type_account).unwrap(),
                        *self.contract_hashes.get(&AUCTION).unwrap(),
                        BID,
                        args,
                    ))
                } else {
                    None
                };
                request.unwrap()
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

    pub fn deploy_contracts(&mut self) {
        self.deploy_contract(KYC);
        self.deploy_contract(GRANT_GATE_KEEPER);
        self.deploy_contract(NFT);
        self.deploy_contract(MINT);
        self.deploy_contract(GRANT_BUYER_ALI);
        self.deploy_contract(GRANT_BUYER_BOB);
        self.deploy_contract(AUCTION);
        dbg!(&self);
        self.deploy_contract(BID_ENGLISH_BUYER_ALI);
        // self.deploy_contract(BID_ENGLISH_BUYER_BOB);
        dbg!(&self);
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

    pub fn get_account_purse(&self, account_hash: AccountHash) -> URef {
        let account = self
            .builder
            .get_account(account_hash)
            .expect("could not get account purse");
        account.main_purse()
    }

    fn get_admin_account_hash(&self) -> AccountHash {
        *self.account_hashes.get(&TypeAccount::Admin).unwrap()
    }

    fn get_auction_runtime_args(&self, type_deploy: TypeDeploy) -> Option<RuntimeArgs> {
        let admin_account_hash: AccountHash = self.get_admin_account_hash();
        let runtime_args = if TypeDeploy::Auction == type_deploy {
            let english = true;
            let now = AuctionArgsBuilder::get_now_u64();
            let mut auction_args = AuctionArgsBuilder::default();
            if !english {
                auction_args.set_dutch();
            }
            auction_args.set_name(AUCTION_NAME);
            auction_args.set_cancellation_time(now + 30000);
            auction_args.set_end_time(now + 600000);
            auction_args.set_beneficiary(&admin_account_hash);
            auction_args.set_token_contract_hash(self.package_hashes.get(&NFT).unwrap());
            auction_args.set_kyc_package_hash(self.package_hashes.get(&KYC).unwrap());
            auction_args.set_token_id(TOKEN_ECP47_NAME);
            dbg!(&auction_args);
            Some(auction_args.build())
        } else {
            None
        };
        runtime_args
    }

    fn get_call_args(&self, type_deploy: TypeDeploy) -> Option<RuntimeArgs> {
        let runtime_args = match type_deploy {
            TypeDeploy::GrantBuyer(type_account) => {
                let recipient_account_hash = *self.account_hashes.get(&type_account).unwrap();
                Some(runtime_args! {
                    RUNTIME_ARG_RECIPIENT => Key::Account(recipient_account_hash),
                    TOKEN_ID => Some(format!("{TOKEN_KYC_NAME}_{type_deploy}")),
                    TOKEN_META => ""
                })
            }
            TypeDeploy::Bid(type_account, amount) => Some(runtime_args! {
                BID => U512::from(amount),
                BID_PURSE => self.get_account_purse(*self.account_hashes.get(&type_account).unwrap()),
            }),
            _ => None,
        };
        runtime_args
    }

    fn get_contract_hash_from_named_key(
        &self,
        type_deploy: TypeDeploy,
    ) -> (Option<ContractHash>, Option<ContractPackageHash>) {
        let (contract_hash, package_hash) = if let Some(KYC | NFT | AUCTION) = Some(type_deploy) {
            let account = self
                .builder
                .get_expected_account(self.account_hashes[&TypeAccount::Admin]);
            let (contract_hash_name, package_hash_name) = get_contracts_name_constants(type_deploy);
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
            NFT => runtime_args! {
                RUNTIME_ARG_NAME_NAME => TOKEN_ECP47_NAME,
                RUNTIME_ARG_NAME_SYMBOL => TOKEN_ECP47_SYMBOL,
                RUNTIME_ARG_NAME_META => "",
                RUNTIME_ARG_ADMIN => Key::Account(admin_account_hash),
                RUNTIME_ARG_CONTRACT_NAME => KEY_ECP47_CONTRACT_NAME
            },
            MINT => runtime_args! {
                RUNTIME_ARG_RECIPIENT => Key::Account(admin_account_hash),
                TOKEN_IDS => Some(vec![TOKEN_ECP47_NAME]),
                TOKEN_METAS => vec![""],
                TOKEN_GAUGES => vec![""],
                TOKEN_WAREHOUSES => vec![""],
                TOKEN_COMISSIONS => vec![""],
            },
            GRANT_BUYER_ALI | GRANT_BUYER_BOB | BID_ENGLISH_BUYER_ALI | BID_ENGLISH_BUYER_BOB => {
                self.get_call_args(type_deploy).unwrap()
            }
            AUCTION => self.get_auction_runtime_args(type_deploy).unwrap(),
            _ => todo!(),
        }
    }

    pub fn new(builder: WasmTestBuilder<InMemoryGlobalState>) -> Self {
        let contract_hashes = HashMap::new();
        let package_hashes = HashMap::new();
        let account_hashes = HashMap::new();
        let mut test_auction = Self {
            builder,
            contract_hashes,
            account_hashes,
            package_hashes,
        };
        test_auction.account_hashes = test_auction.get_account_hashes();
        test_auction
    }
}

fn get_session_file(type_deploy: TypeDeploy) -> Option<&'static str> {
    match type_deploy {
        KYC => Some(CONTRACT_KYC),
        NFT => Some(CONTRACT_ECP47_TOKEN),
        AUCTION => Some(CONTRACT_AUCTION),
        _ => None,
    }
}

fn get_contracts_name_constants(type_deploy: TypeDeploy) -> (&'static str, &'static str) {
    match type_deploy {
        KYC => (KEY_KYC_CONTRACT_HASH, KEY_KYC__PACKAGE_HASH),
        NFT => (KEY_ECP47_CONTRACT_HASH, KEY_ECP47_PACKAGE_HASH),
        AUCTION => (KEY_AUCTION_CONTRACT_HASH, KEY_AUCTION_PACKAGE_HASH),
        _ => unimplemented!(),
    }
}

pub fn fund_account(account: &AccountHash) -> ExecuteRequest {
    let deploy_item = DeployItemBuilder::new()
        .with_address(*DEFAULT_ACCOUNT_ADDR)
        .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
        .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
        .with_transfer_args(runtime_args! {
            mint::ARG_AMOUNT => U512::from(MINIMUM_ACCOUNT_CREATION_BALANCE),
            mint::ARG_TARGET => *account,
            mint::ARG_ID => <Option::<u64>>::None
        })
        .with_deploy_hash([1; 32])
        .build();

    ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
}
