use std::{collections::BTreeMap, path::PathBuf};

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractPackageHash,
    HashAddr, Key, PublicKey, RuntimeArgs, SecretKey, URef, U256, U512,
};

pub mod meta {
    use super::Meta;
    use maplit::btreemap;

    pub fn red_dragon() -> Meta {
        btreemap! {
            "color".to_string() => "red".to_string()
        }
    }

    pub fn blue_dragon() -> Meta {
        btreemap! {
            "color".to_string() => "blue".to_string()
        }
    }

    pub fn black_dragon() -> Meta {
        btreemap! {
            "color".to_string() => "black".to_string()
        }
    }

    pub fn gold_dragon() -> Meta {
        btreemap! {
            "color".to_string() => "gold".to_string()
        }
    }
}

pub mod token_cfg {
    use super::Meta;
    use maplit::btreemap;

    pub const NAME: &str = "DragonsNFT";
    pub const SYMBOL: &str = "DRAG";

    pub fn contract_meta() -> Meta {
        btreemap! {
            "origin".to_string() => "fire".to_string()
        }
    }
}

pub const CONTRACT_KEY: &str = "NFT_contract_hash";
pub const CONTRACT_HASH_KEY: &str = "NFT_contract_hash_wrapped";

const BALANCES_DICT: &str = "balances";
const OWNED_TOKENS_DICT: &str = "owned_tokens";
const TOKEN_OWNERS_DICT: &str = "owners";
const METADATA_DICT: &str = "metadata";

use crate::Hash;

pub struct CasperCEP47Contract {
    pub builder: InMemoryWasmTestBuilder,
    pub nft_hash: Hash,
    pub nft_package: Hash,
    pub kyc_hash: Hash,
    pub kyc_package_hash: Hash,
    pub admin: AccountHash,
    pub ali: AccountHash,
    pub bob: AccountHash,
}

pub type TokenId = String;
pub type Meta = BTreeMap<String, String>;

use crate::utils::{deploy, fund_account, query, DeploySource, query_dictionary_item};

impl CasperCEP47Contract {
    pub fn deploy() -> Self {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

        let admin_pk: PublicKey = (&admin_secret).into();
        let admin = admin_pk.to_account_hash();
        let ali_pk: PublicKey = (&ali_secret).into();
        let ali = ali_pk.to_account_hash();
        let bob_pk: PublicKey = (&bob_secret).into();
        let bob = bob_pk.to_account_hash();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(fund_account(&admin)).expect_success().commit();
        builder.exec(fund_account(&ali)).expect_success().commit();
        builder.exec(fund_account(&bob)).expect_success().commit();

        let kyc_code = PathBuf::from("civic-token.wasm");
        let mut meta = BTreeMap::new();
        meta.insert("origin".to_string(), "kyc".to_string());

        let kyc_args = runtime_args! {
            "name" => "kyc",
            "contract_name" => "kyc",
            "symbol" => "symbol",
            "meta" => meta,
            "admin" => Key::Account(admin)
        };

        deploy(
            &mut builder,
            &admin,
            &DeploySource::Code(kyc_code),
            kyc_args,
            true,
            None,
        );

        let kyc_hash = query(
            &builder,
            Key::Account(admin),
            &["kyc_contract_hash_wrapped".to_string()],
        );

        let kyc_package_hash = query(
            &builder,
            Key::Account(admin),
            &["kyc_package_hash_wrapped".to_string()],
        );
        
        let token_code = PathBuf::from("cask-token.wasm");
        let token_args = runtime_args! {
            "name" => token_cfg::NAME,
            "symbol" => token_cfg::SYMBOL,
            "meta" => token_cfg::contract_meta(),
            "admin" => Key::Account(admin),
            "kyc_package_hash" => Key::Hash(kyc_package_hash),
            "contract_name" => "NFT".to_string()
        };
        
        deploy(
            &mut builder,
            &admin,
            &DeploySource::Code(token_code),
            token_args,
            true,
            None,
        );
        let nft_hash = query(
            &builder,
            Key::Account(admin),
            &[CONTRACT_HASH_KEY.to_string()],
        );
        let nft_package = query(
            &builder,
            Key::Account(admin),
            &["NFT_package_hash_wrapped".to_string()],
        );

        Self {
            builder,
            nft_hash,
            nft_package,
            kyc_hash,
            kyc_package_hash,
            admin,
            ali,
            bob,
        }
    }

    pub fn add_kyc(&mut self, recipient: AccountHash) {
        let mut token_meta = BTreeMap::new();
        token_meta.insert("status".to_string(), "active".to_string());
        let args = runtime_args! {
            "recipient" => Key::Account(recipient),
            "token_id" => Some(recipient.to_string()),
            "token_meta" => token_meta
        };
        deploy(
            &mut self.builder,
            &self.admin,
            &DeploySource::ByHash {
                hash: self.kyc_package_hash,
                method: "mint".to_string(),
            },
            args,
            true,
            None,
        );
        
    }

    fn call(&mut self, sender: &AccountHash, method: &str, args: RuntimeArgs) {
        deploy(
            &mut self.builder,
            &sender,
            &DeploySource::ByHash {
                hash: self.nft_package,
                method: method.to_string(),
            },
            args,
            true,
            None,
        );
    }

    fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        query(
            &self.builder,
            Key::Account(self.admin),
            &[CONTRACT_KEY.to_string(), name.to_string()],
        )
    }

    fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        query_dictionary_item(
            &self.builder,
            Key::Hash(self.nft_hash),
            Some(dict_name.to_string()),
            key,
        ).expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t()
        .expect("Wrong type in query result.")
    }

    pub fn get_event(&self, index: u32) -> BTreeMap<String, String> {
        self.query_dictionary_value("events", index.to_string())
            .unwrap()
    }

    pub fn get_events(&self) -> Vec<BTreeMap<String, String>> {
        let mut events = Vec::new();
        for i in 0..self.get_events_count() {
            events.push(self.get_event(i));
        }
        events
    }

    pub fn get_events_count(&self) -> u32 {
        self.query_contract("events_count").unwrap()
    }

    pub fn mint(
        &mut self,
        recipient: &Key,
        token_id: &str,
        token_meta: &Meta,
        sender: &AccountHash,
        commissions: BTreeMap<String, String>,
    ) {
        let mut gauge: BTreeMap<String, String> = BTreeMap::new();
        gauge.insert("gauge".to_string(), "is_gaugy".to_string());
        let mut warehouse: BTreeMap<String, String> = BTreeMap::new();
        warehouse.insert("ware".to_string(), "house".to_string());

        self.call(
            sender,
            "mint",
            runtime_args! {
                "recipient" => *recipient,
                "token_ids" => Some(vec![token_id.to_string()]),
                "token_metas" => vec![token_meta.clone()],
                "token_gauges" => vec![gauge],
                "token_warehouses" => vec![warehouse],
                "token_commissions" => vec![commissions],
            },
        );
    }

    pub fn mint_copies(
        &mut self,
        recipient: &AccountHash,
        token_ids: Option<&Vec<TokenId>>,
        token_meta: &Meta,
        count: u32,
        sender: &AccountHash,
    ) {
        self.call(
            sender,
            "mint_copies",
            runtime_args! {
                "recipient" => Key::from(*recipient),
                "token_ids" => token_ids.cloned(),
                "token_meta" => token_meta.clone(),
                "count" => count
            },
        );
    }

    pub fn mint_many(
        &mut self,
        recipient: &AccountHash,
        token_ids: Option<&Vec<TokenId>>,
        token_metas: &[Meta],
        sender: &AccountHash,
    ) {
        self.call(
            sender,
            "mint_many",
            runtime_args! {
                "recipient" => Key::from(*recipient),
                "token_ids" => token_ids.cloned(),
                "token_metas" => token_metas.to_owned(),
            },
        );
    }

    pub fn transfer_token(&mut self, recipient: &Key, token_id: TokenId, sender: &AccountHash) {
        self.call(
            sender,
            "transfer_token",
            runtime_args! {
                "recipient" => *recipient,
                "token_id" => vec![token_id]
            },
        );
    }

    fn key_to_str(key: &Key) -> String {
        match key {
            Key::Account(account) => account.to_string(),
            Key::Hash(package) => hex::encode(package),
            _ => panic!(),
        }
    }
}
