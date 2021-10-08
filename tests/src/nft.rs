use std::collections::BTreeMap;

use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractPackageHash,
    HashAddr, Key, PublicKey, RuntimeArgs, SecretKey, U256, U512,
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

pub struct CasperCEP47Contract {
    pub context: TestContext,
    pub hash: Hash,
    pub nft_package: Hash,
    pub kyc_hash: Hash,
    pub kyc_package_hash: Hash,
    pub admin: AccountHash,
    pub ali: AccountHash,
    pub bob: AccountHash,
}

pub type TokenId = String;
pub type Meta = BTreeMap<String, String>;

impl CasperCEP47Contract {
    pub fn deploy() -> Self {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

        let admin: PublicKey = (&admin_secret).into();
        let admin_hash = admin.to_account_hash();
        let ali: PublicKey = (&ali_secret).into();
        let ali_hash = ali.to_account_hash();
        let bob: PublicKey = (&bob_secret).into();
        let bob_hash = bob.to_account_hash();
        let mut context = TestContextBuilder::new()
            .with_public_key(admin, U512::from(500_000_000_000_000_000u64))
            .with_public_key(ali, U512::from(500_000_000_000_000_000u64))
            .with_public_key(bob, U512::from(500_000_000_000_000_000u64))
            .build();
        let session_code = Code::from("cask-token.wasm");
        let session_args = runtime_args! {
            "name" => token_cfg::NAME,
            "symbol" => token_cfg::SYMBOL,
            "meta" => token_cfg::contract_meta(),
            "admin" => Key::Account(admin_hash),
            "contract_name" => "NFT".to_string()
        };
        let session = SessionBuilder::new(session_code, session_args)
            .with_address(admin_hash)
            .with_authorization_keys(&[admin_hash])
            .build();
        context.run(session);
        let hash = context
            .query(admin_hash, &[CONTRACT_HASH_KEY.to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        let nft_package = context
            .query(admin_hash, &["NFT_package_hash_wrapped".to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        let kyc_code = Code::from("civic-token.wasm");
        let mut meta = BTreeMap::new();
        meta.insert("origin".to_string(), "kyc".to_string());

        let kyc_args = runtime_args! {
            "name" => "kyc",
            "contract_name" => "kyc",
            "symbol" => "symbol",
            "meta" => meta,
            "admin" => Key::Account(admin_hash)
        };
        let kyc_session = SessionBuilder::new(kyc_code, kyc_args)
            .with_address(admin_hash)
            .with_authorization_keys(&[admin_hash])
            .build();

        context.run(kyc_session);
        let kyc_hash = context
            .query(admin_hash, &["kyc_contract_hash_wrapped".to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        let kyc_package_hash = context
            .query(admin_hash, &["kyc_package_hash_wrapped".to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        Self {
            context,
            hash,
            nft_package,
            kyc_hash,
            kyc_package_hash,
            admin: admin_hash,
            ali: ali_hash,
            bob: bob_hash,
        }
    }

    pub fn add_kyc(&mut self, recipient: AccountHash) {
        let code = Code::Hash(self.kyc_hash, "mint".to_string());
        let mut token_meta = BTreeMap::new();
        token_meta.insert("status".to_string(), "active".to_string());
        let args = runtime_args! {
            "recipient" => Key::Account(recipient),
            "token_id" => Some(recipient.to_string()),
            "token_meta" => token_meta
        };
        let session = SessionBuilder::new(code, args)
            .with_address(self.admin)
            .with_authorization_keys(&[self.admin])
            .build();
        self.context.run(session);
    }

    fn call(&mut self, sender: &AccountHash, method: &str, args: RuntimeArgs) {
        let account = *sender;
        let code = Code::Hash(self.hash, method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(account)
            .with_authorization_keys(&[account])
            .build();
        self.context.run(session);
    }

    fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self
            .context
            .query(self.admin, &[CONTRACT_KEY.to_string(), name.to_string()])
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                Some(value)
            }
        }
    }

    fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        match self.context.query_dictionary_item(
            Key::Hash(self.hash),
            Some(dict_name.to_string()),
            key,
        ) {
            Err(_) => None,
            Ok(maybe_value) => {
                let value: Option<T> = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("is not expected type."));
                value
            }
        }
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
