use super::{
    auction_args::AuctionArgsBuilder,
    constants::{
        ARG_ADMIN, ARG_COLLECTION_NAME, ARG_COLLECTION_SYMBOL, ARG_CONTRACT_NAME,
        ARG_IDENTIFIER_MODE, ARG_JSON_SCHEMA, ARG_META, ARG_METADATA_MUTABILITY, ARG_NAME,
        ARG_NFT_KIND, ARG_NFT_METADATA_KIND, ARG_OWNERSHIP_MODE, ARG_RECIPIENT, ARG_SYMBOL,
        ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ARG_TOTAL_TOKEN_SUPPLY, AUCTION_CONTRACT,
        CONTRACT_AUCTION, CONTRACT_CEP_47_TOKEN, CONTRACT_CEP_78_TOKEN, CONTRACT_KYC,
        KEY_CEP_47_CONTRACT_HASH, KEY_CEP_47_CONTRACT_NAME, KEY_CEP_47_PACKAGE_HASH,
        KEY_CEP_78_CONTRACT_HASH, KEY_CEP_78_PACKAGE_HASH, KEY_KYC_CONTRACT_HASH,
        KEY_KYC_CONTRACT_NAME, KEY_KYC_PACKAGE_HASH, PURSE_NAME, PURSE_NAME_VALUE,
        SESSION_BID_PURSE, SESSION_DELTA_BID_PURSE, SESSION_EXTENDED_BID_PURSE, TOKEN_CEP_47_NAME,
        TOKEN_CEP_47_SYMBOL, TOKEN_CEP_78_NAME, TOKEN_CEP_78_SYMBOL, TOKEN_COMISSIONS,
        TOKEN_GAUGES, TOKEN_ID, TOKEN_IDS, TOKEN_KYC_NAME, TOKEN_KYC_SYMBOL, TOKEN_META,
        TOKEN_METAS, TOKEN_WAREHOUSES,
    },
    utils::{deploy, fund_account, query, query_dictionary_item, DeploySource},
};
use casper_engine_test_support::{
    InMemoryWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, system::mint::METHOD_MINT, CLTyped,
    ContractHash, ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};
use maplit::btreemap;
use std::{collections::BTreeMap, path::PathBuf};

pub struct AuctionContract {
    pub builder: InMemoryWasmTestBuilder,
    pub auction_hash: ContractHash,
    pub auction_package: ContractPackageHash,
    pub nft_hash: ContractHash,
    pub nft_package: ContractPackageHash,
    pub kyc_hash: ContractHash,
    pub kyc_package: ContractPackageHash,
    pub admin: AccountHash,
    pub ali: AccountHash,
    pub bob: AccountHash,
}

impl AuctionContract {
    pub fn deploy_with_default_args(english: bool, start_time: u64) -> Self {
        let mut auction_args = AuctionArgsBuilder::default();
        if !english {
            auction_args.set_dutch();
        }
        auction_args.set_start_time(start_time);
        Self::deploy_contracts(auction_args)
    }

    pub fn deploy(auction_args: AuctionArgsBuilder) -> Self {
        if auction_args.has_enhanced_nft() {
            Self::deploy_contracts_with_enhanced_nft(auction_args)
        } else {
            Self::deploy_contracts(auction_args)
        }
    }

    pub fn get_accounts(
        builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    ) -> (AccountHash, AccountHash, AccountHash) {
        let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let ali_secret = SecretKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob_secret = SecretKey::ed25519_from_bytes([5u8; 32]).unwrap();

        let admin_pk: PublicKey = PublicKey::from(&admin_secret);
        let admin = admin_pk.to_account_hash();
        let ali_pk: PublicKey = PublicKey::from(&ali_secret);
        let ali = ali_pk.to_account_hash();
        let bob_pk: PublicKey = PublicKey::from(&bob_secret);
        let bob = bob_pk.to_account_hash();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(fund_account(&admin)).expect_success().commit();
        builder.exec(fund_account(&ali)).expect_success().commit();
        builder.exec(fund_account(&bob)).expect_success().commit();
        (admin, ali, bob)
    }

    fn deploy_contracts(mut auction_args: AuctionArgsBuilder) -> Self {
        let mut builder = InMemoryWasmTestBuilder::default();
        let (admin, ali, bob) = Self::get_accounts(&mut builder);
        let (kyc_hash, kyc_package) = Self::deploy_kyc(&mut builder, &admin);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &admin);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &ali);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &bob);

        let (nft_hash, nft_package) = Self::deploy_nft(&mut builder, &admin);

        let token_id = String::from("custom_token_id");
        let token_meta = btreemap! {
            "origin".to_string() => "fire".to_string()
        };
        let commissions = BTreeMap::new();

        Self::mint_nft(
            &mut builder,
            &nft_package,
            &Key::Account(admin),
            &token_id,
            &token_meta,
            &admin,
            commissions,
        );

        auction_args.set_beneficiary(&admin);
        auction_args.set_token_contract_hash(&nft_package);
        auction_args.set_kyc_package_hash(&kyc_package);
        auction_args.set_token_id(&token_id);

        let (auction_hash, auction_package) =
            Self::deploy_auction(&mut builder, &admin, auction_args.build());

        Self {
            builder,
            auction_hash,
            auction_package,
            nft_hash,
            nft_package,
            kyc_hash,
            kyc_package,
            admin,
            ali,
            bob,
        }
    }

    fn deploy_contracts_with_enhanced_nft(mut auction_args: AuctionArgsBuilder) -> Self {
        let mut builder = InMemoryWasmTestBuilder::default();
        let (admin, ali, bob) = Self::get_accounts(&mut builder);
        let (kyc_hash, kyc_package) = Self::deploy_kyc(&mut builder, &admin);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &admin);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &ali);
        Self::add_kyc(&mut builder, &kyc_package, &admin, &bob);

        let (enhanced_nft_hash, enhanced_nft_package) =
            Self::deploy_enhanced_nft(&mut builder, &admin);

        Self::mint_enhanced_nft(
            &mut builder,
            &enhanced_nft_package,
            &admin,
            &Key::Account(admin),
        );
        auction_args.set_token_id(TOKEN_META);
        auction_args.set_beneficiary(&admin);
        auction_args.set_token_contract_hash(&enhanced_nft_package);
        auction_args.set_kyc_package_hash(&kyc_package);

        let (auction_hash, auction_package) =
            Self::deploy_auction(&mut builder, &admin, auction_args.build());

        Self {
            builder,
            auction_hash,
            auction_package,
            nft_hash: enhanced_nft_hash,
            nft_package: enhanced_nft_package,
            kyc_hash,
            kyc_package,
            admin,
            ali,
            bob,
        }
    }

    pub fn deploy_kyc(
        builder: &mut InMemoryWasmTestBuilder,
        admin: &AccountHash,
    ) -> (ContractHash, ContractPackageHash) {
        let mut meta = BTreeMap::new();
        meta.insert("origin".to_string(), "kyc".to_string());

        let kyc_args = runtime_args! {
            ARG_NAME => TOKEN_KYC_NAME,
            ARG_CONTRACT_NAME => KEY_KYC_CONTRACT_NAME,
            ARG_SYMBOL => TOKEN_KYC_SYMBOL,
            ARG_META => meta,
            ARG_ADMIN => Key::Account(*admin)
        };
        let auction_code = PathBuf::from(CONTRACT_KYC);
        deploy(
            builder,
            admin,
            &DeploySource::Code(auction_code),
            kyc_args,
            true,
            None,
        );

        let contract_hash = query(
            builder,
            Key::Account(*admin),
            &[[KEY_KYC_CONTRACT_HASH, "wrapped"].join("_")],
        );
        let contract_package = query(
            builder,
            Key::Account(*admin),
            &[[KEY_KYC_PACKAGE_HASH, "wrapped"].join("_")],
        );

        (contract_hash, contract_package)
    }

    pub fn deploy_nft(
        builder: &mut InMemoryWasmTestBuilder,
        admin: &AccountHash,
    ) -> (ContractHash, ContractPackageHash) {
        let token_args = runtime_args! {
            ARG_NAME => TOKEN_CEP_47_NAME,
            ARG_SYMBOL => TOKEN_CEP_47_SYMBOL,
            ARG_META => "",
            ARG_ADMIN => Key::Account(*admin),
            ARG_CONTRACT_NAME => KEY_CEP_47_CONTRACT_NAME
        };
        let nft_code = PathBuf::from(CONTRACT_CEP_47_TOKEN);
        deploy(
            builder,
            admin,
            &DeploySource::Code(nft_code),
            token_args,
            true,
            None,
        );

        let contract_hash: ContractHash = query(
            builder,
            Key::Account(*admin),
            &[[KEY_CEP_47_CONTRACT_HASH, "wrapped"].join("_")],
        );
        let contract_package: ContractPackageHash = query(
            builder,
            Key::Account(*admin),
            &[[KEY_CEP_47_PACKAGE_HASH, "wrapped"].join("_")],
        );
        (contract_hash, contract_package)
    }

    fn deploy_enhanced_nft(
        builder: &mut InMemoryWasmTestBuilder,
        admin: &AccountHash,
    ) -> (ContractHash, ContractPackageHash) {
        let token_args = runtime_args! {
            ARG_COLLECTION_NAME => TOKEN_CEP_78_NAME,
            ARG_COLLECTION_SYMBOL => TOKEN_CEP_78_SYMBOL,
            ARG_TOTAL_TOKEN_SUPPLY => 1000_u64,
            ARG_OWNERSHIP_MODE => 2_u8, // transferable
            ARG_NFT_KIND => 1_u8, // virtual good
            ARG_NFT_METADATA_KIND => 2_u8,
            ARG_JSON_SCHEMA => ARG_META,
            ARG_IDENTIFIER_MODE => 1_u8,
            ARG_METADATA_MUTABILITY => 0_u8,
        };
        let nft_code = PathBuf::from(CONTRACT_CEP_78_TOKEN);

        deploy(
            builder,
            admin,
            &DeploySource::Code(nft_code),
            token_args,
            true,
            None,
        );

        let account = builder.get_expected_account(*admin);
        let contract_hash = account
            .named_keys()
            .get(KEY_CEP_78_CONTRACT_HASH)
            .expect("must have contract hash key as part of contract creation")
            .into_hash()
            .map(ContractHash::new)
            .expect("must be contract hash");
        let package_hash = account
            .named_keys()
            .get(KEY_CEP_78_PACKAGE_HASH)
            .expect("must have package hash key as part of contract creation")
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("must be contract hash");

        (contract_hash, package_hash)
    }

    pub fn deploy_auction(
        builder: &mut InMemoryWasmTestBuilder,
        admin: &AccountHash,
        auction_args: RuntimeArgs,
    ) -> (ContractHash, ContractPackageHash) {
        let auction_code = PathBuf::from(CONTRACT_AUCTION);
        deploy(
            builder,
            admin,
            &DeploySource::Code(auction_code),
            auction_args,
            true,
            None,
        );

        // TODO refacto "test" name value
        let contract_hash: ContractHash = query(
            builder,
            Key::Account(*admin),
            &["test_auction_contract_hash_wrapped".to_string()],
        );
        let contract_package: ContractPackageHash = query(
            builder,
            Key::Account(*admin),
            &["test_auction_contract_package_hash_wrapped".to_string()],
        );
        (contract_hash, contract_package)
    }

    pub fn mint_nft(
        builder: &mut InMemoryWasmTestBuilder,
        nft_package: &ContractPackageHash,
        recipient: &Key,
        token_id: &str,
        token_meta: &BTreeMap<String, String>,
        sender: &AccountHash,
        mut commissions: BTreeMap<String, String>,
    ) {
        let mut gauge: BTreeMap<String, String> = BTreeMap::new();
        gauge.insert("gauge".to_string(), "is_gaugy".to_string());
        let mut warehouse: BTreeMap<String, String> = BTreeMap::new();
        warehouse.insert("ware".to_string(), "house".to_string());
        commissions.insert(
            "comm_account".to_string(),
            "Key::Account(7de52a3013f609faa38ae99af4350da6aa6b69bec0e4087ecae87c2b9486a265)"
                .to_string(),
        );
        commissions.insert("comm_rate".to_string(), "55".to_string());
        let args = runtime_args! {
            ARG_RECIPIENT => *recipient,
            TOKEN_IDS => Some(vec![token_id.to_string()]),
            TOKEN_METAS => vec![token_meta.clone()],
            TOKEN_GAUGES => vec![gauge],
            TOKEN_WAREHOUSES => vec![warehouse],
            TOKEN_COMISSIONS => vec![commissions],
        };
        deploy(
            builder,
            sender,
            &DeploySource::ByPackageHash {
                package_hash: *nft_package,
                method: METHOD_MINT.to_string(),
            },
            args,
            true,
            None,
        );
    }

    pub fn mint_enhanced_nft(
        builder: &mut InMemoryWasmTestBuilder,
        package_hash: &ContractPackageHash,
        sender: &AccountHash,
        recipient: &Key,
    ) {
        let args = runtime_args! {
            ARG_TOKEN_OWNER => *recipient,
            ARG_TOKEN_META_DATA => TOKEN_META,
        };
        deploy(
            builder,
            sender,
            &DeploySource::ByPackageHash {
                package_hash: *package_hash,
                method: METHOD_MINT.to_string(),
            },
            args,
            true,
            None,
        );
    }

    pub fn add_kyc_token(&mut self, recipient: &AccountHash) {
        Self::add_kyc(&mut self.builder, &self.kyc_package, &self.admin, recipient)
    }

    pub fn add_kyc(
        builder: &mut InMemoryWasmTestBuilder,
        kyc_package: &ContractPackageHash,
        admin: &AccountHash,
        recipient: &AccountHash,
    ) {
        let mut token_meta = BTreeMap::new();
        token_meta.insert("status".to_string(), "active".to_string());
        let args = runtime_args! {
            ARG_RECIPIENT => Key::Account(*recipient),
            TOKEN_ID => Some(recipient.to_string()),
            TOKEN_META => token_meta,
        };

        deploy(
            builder,
            admin,
            &DeploySource::ByPackageHash {
                package_hash: *kyc_package,
                method: METHOD_MINT.to_string(),
            },
            args,
            true,
            None,
        );
    }

    pub fn bid(&mut self, bidder: &AccountHash, amount: U512, block_time: u64) {
        let session_code = PathBuf::from(SESSION_BID_PURSE);
        deploy(
            &mut self.builder,
            bidder,
            &DeploySource::Code(session_code),
            runtime_args! {
                ARG_AMOUNT => amount,
                PURSE_NAME => PURSE_NAME_VALUE,
                AUCTION_CONTRACT => self.auction_hash
            },
            true,
            Some(block_time),
        );
    }

    pub fn extend_bid(&mut self, bidder: &AccountHash, amount: U512, block_time: u64) {
        let session_code = PathBuf::from(SESSION_EXTENDED_BID_PURSE);
        deploy(
            &mut self.builder,
            bidder,
            &DeploySource::Code(session_code),
            runtime_args! {
                ARG_AMOUNT => amount,
                PURSE_NAME => PURSE_NAME_VALUE,
                AUCTION_CONTRACT => self.auction_hash
            },
            true,
            Some(block_time),
        );
    }

    pub fn delta_bid(&mut self, bidder: &AccountHash, amount: U512, block_time: u64) {
        let session_code = PathBuf::from(SESSION_DELTA_BID_PURSE);
        deploy(
            &mut self.builder,
            bidder,
            &DeploySource::Code(session_code),
            runtime_args! {
                ARG_AMOUNT => amount,
                PURSE_NAME => PURSE_NAME_VALUE,
                AUCTION_CONTRACT => self.auction_hash
            },
            true,
            Some(block_time),
        );
    }

    pub fn cancel_auction(&mut self, caller: &AccountHash, time: u64) {
        self.call(caller, "cancel_auction", runtime_args! {}, time)
    }

    pub fn cancel_bid(&mut self, caller: &AccountHash, time: u64) {
        self.call(caller, "cancel_bid", runtime_args! {}, time)
    }

    pub fn finalize(&mut self, caller: &AccountHash, time: u64) {
        self.call(caller, "finalize", runtime_args! {}, time)
    }

    pub fn is_finalized(&self) -> bool {
        self.query_contract(self.auction_hash.value(), "finalized")
    }

    pub fn get_end(&self) -> u64 {
        self.query_contract(self.auction_hash.value(), "end_time")
    }

    pub fn get_winner(&self) -> Option<AccountHash> {
        self.query_contract(self.auction_hash.value(), "current_winner")
    }

    pub fn get_winning_bid(&self) -> Option<U512> {
        self.query_contract(self.auction_hash.value(), "winning_bid")
    }

    pub fn get_event(&self, contract_hash: [u8; 32], index: u32) -> BTreeMap<String, String> {
        self.query_dictionary_value(
            Key::Hash(contract_hash),
            if contract_hash != self.auction_hash.value() {
                "events"
            } else {
                "auction_events"
            },
            index.to_string(),
        )
        .unwrap()
    }

    pub fn get_events(&self, contract_hash: [u8; 32]) -> Vec<BTreeMap<String, String>> {
        let mut events = Vec::new();
        for i in 0..self.get_events_count(contract_hash) {
            events.push(self.get_event(contract_hash, i));
        }
        events
    }

    pub fn get_events_count(&self, contract_hash: [u8; 32]) -> u32 {
        self.query_contract(
            contract_hash,
            if contract_hash != self.auction_hash.value() {
                "events_count"
            } else {
                "auction_events_count"
            },
        )
    }

    /// Wrapper function for calling an entrypoint on the contract with the access rights of the deployer.
    fn call(&mut self, caller: &AccountHash, method: &str, args: RuntimeArgs, time: u64) {
        deploy(
            &mut self.builder,
            caller,
            &DeploySource::ByPackageHash {
                package_hash: self.auction_package,
                method: method.to_string(),
            },
            args,
            true,
            Some(time),
        );
    }

    fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        base: Key,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        query_dictionary_item(&self.builder, base, Some(dict_name.to_string()), key)
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t()
            .expect("Wrong type in query result.")
    }

    // TODO remove hardcoded names DragonsNFT / test
    fn query_contract<T: CLTyped + FromBytes>(&self, contract_hash: [u8; 32], name: &str) -> T {
        query(
            &self.builder,
            Key::Account(self.admin),
            &[
                if contract_hash != self.auction_hash.value() {
                    "DragonsNFT_contract".to_string()
                } else {
                    "test_auction_contract_hash".to_string()
                },
                name.to_string(),
            ],
        )
    }

    /// Getter function for the balance of an account.
    pub fn get_account_balance(&self, account_key: &AccountHash) -> U512 {
        let account = self
            .builder
            .get_account(*account_key)
            .expect("should get genesis account");
        self.builder.get_purse_balance(account.main_purse())
    }

    /// Shorthand to get the balances of all 3 accounts in order.
    pub fn get_all_accounts_balance(&self) -> (U512, U512, U512) {
        (
            self.get_account_balance(&self.admin),
            self.get_account_balance(&self.ali),
            self.get_account_balance(&self.bob),
        )
    }

    pub fn get_marketplace_balance(&self) -> U512 {
        let account = self
            .builder
            .get_account(AccountHash::new([11_u8; 32]))
            .expect("should get genesis account");
        self.builder.get_purse_balance(account.main_purse())
    }

    pub fn get_comm_balance(&self) -> U512 {
        let account = self
            .builder
            .get_account(
                AccountHash::from_formatted_str(
                    "account-hash-7de52a3013f609faa38ae99af4350da6aa6b69bec0e4087ecae87c2b9486a265",
                )
                .unwrap(),
            )
            .expect("should get genesis account");
        self.builder.get_purse_balance(account.main_purse())
    }
}
