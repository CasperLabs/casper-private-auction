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
        let ali: PublicKey = (&ali_secret).into();
        let bob: PublicKey = (&bob_secret).into();
        let mut context = TestContextBuilder::new()
            .with_public_key(admin.clone(), U512::from(500_000_000_000_000_000u64))
            .with_public_key(ali.clone(), U512::from(500_000_000_000_000_000u64))
            .with_public_key(bob.clone(), U512::from(500_000_000_000_000_000u64))
            .build();
        let session_code = Code::from("cep47-token.wasm");
        let session_args = runtime_args! {
            "name" => token_cfg::NAME,
            "symbol" => token_cfg::SYMBOL,
            "meta" => token_cfg::contract_meta(),
            "contract_name" => "NFT".to_string()
        };
        let session = SessionBuilder::new(session_code, session_args)
            .with_address(admin.to_account_hash())
            .with_authorization_keys(&[admin.to_account_hash()])
            .build();
        context.run(session);
        let hash = context
            .query(admin.to_account_hash(), &[CONTRACT_HASH_KEY.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        Self {
            context,
            hash,
            admin: admin.to_account_hash(),
            ali: ali.to_account_hash(),
            bob: bob.to_account_hash(),
        }
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
        token_id: Option<&TokenId>,
        token_meta: &Meta,
        sender: &AccountHash,
    ) {
        self.call(
            sender,
            "mint",
            runtime_args! {
                "recipient" => *recipient,
                "token_ids" => Some(vec![token_id.unwrap().to_owned()]),
                "token_metas" => vec![token_meta.clone()]
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
