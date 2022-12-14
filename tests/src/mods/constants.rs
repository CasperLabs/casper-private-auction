// Wasm names
pub const CONTRACT_KYC: &str = "kyc-contract.wasm";
pub const CONTRACT_CEP_47_TOKEN: &str = "cep_47.wasm";
pub const CONTRACT_CEP_78_TOKEN: &str = "cep_78.wasm";
pub const CONTRACT_AUCTION: &str = "casper-private-auction-installer.wasm";
pub const SESSION_BID_PURSE: &str = "bid-purse.wasm";

// Named Keys
pub const KEY_CEP_47_CONTRACT_NAME: &str = "cep_47_token";
pub const KEY_CEP_47_CONTRACT_HASH: &str = "cep_47_token_contract_hash";
pub const KEY_CEP_47_PACKAGE_HASH: &str = "cep_47_token_package_hash";

pub const KEY_CEP_78_CONTRACT_HASH: &str = "nft_contract";
pub const KEY_CEP_78_PACKAGE_HASH: &str = "nft_contract_package";

pub const KEY_KYC_CONTRACT_NAME: &str = "kyc_token";
pub const KEY_KYC_CONTRACT_HASH: &str = "kyc_token_contract_hash";
pub const KEY_KYC__PACKAGE_HASH: &str = "kyc_token_package_hash";

pub const KEY_AUCTION_CONTRACT_HASH: &str = "AUCTION_auction_contract_hash";
pub const KEY_AUCTION_PACKAGE_HASH: &str = "AUCTION_auction_contract_package_hash";

// Entry points
pub const ENTRY_POINT_GRANT_GATEKEEPER: &str = "grant_gatekeeper";

// Config
pub const AUCTION_NAME: &str = "AUCTION";
pub const TOKEN_CEP_47_NAME: &str = "CEP_47_TOKEN";
pub const TOKEN_CEP_47_SYMBOL: &str = "CEP_47";
pub const TOKEN_CEP_78_NAME: &str = "CEP_78_TOKEN";
pub const TOKEN_CEP_78_SYMBOL: &str = "CEP_78";
pub const TOKEN_KYC_NAME: &str = "KYC_TOKEN";
pub const TOKEN_KYC_SYMBOL: &str = "KYC";

// Runtime Args name
pub const ARG_NAME: &str = "name";
pub const ARG_SYMBOL: &str = "symbol";
pub const ARG_META: &str = "meta";
pub const ARG_ADMIN: &str = "admin";
pub const ARG_CONTRACT_NAME: &str = "contract_name";
pub const ARG_GATEKEEPER: &str = "gatekeeper";
pub const ARG_RECIPIENT: &str = "recipient";
pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub const ARG_ALLOW_MINTING: &str = "allow_minting";
pub const ARG_MINTING_MODE: &str = "minting_mode";
pub const ARG_OWNERSHIP_MODE: &str = "ownership_mode";
pub const ARG_NFT_KIND: &str = "nft_kind";
pub const ARG_HOLDER_MODE: &str = "holder_mode";
pub const ARG_WHITELIST_MODE: &str = "whitelist_mode";
pub const ARG_CONTRACT_WHITELIST: &str = "contract_whitelist";
pub const ARG_JSON_SCHEMA: &str = "json_schema";
pub const ARG_IDENTIFIER_MODE: &str = "identifier_mode";
pub const ARG_BURN_MODE: &str = "burn_mode";
pub const ARG_NFT_METADATA_KIND: &str = "nft_metadata_kind";
pub const ARG_METADATA_MUTABILITY: &str = "metadata_mutability";
pub const ARG_TOKEN_META_DATA: &str = "token_meta_data";

// This const can be found in data.rs
pub const TOKEN_ID: &str = "token_id";
pub const TOKEN_META: &str = "token_meta";
pub const ARG_TOKEN_OWNER: &str = "token_owner";

// Specific
pub const STATUS: &str = "status";
pub const ACTIVE: &str = "active";
pub const TOKEN_IDS: &str = "token_ids";
pub const TOKEN_METAS: &str = "token_metas";
pub const TOKEN_GAUGES: &str = "token_gauges";
pub const TOKEN_WAREHOUSES: &str = "token_warehouses";
pub const TOKEN_COMISSIONS: &str = "token_commissions";
pub const PURSE_NAME: &str = "purse_name";
pub const PURSE_NAME_VALUE: &str = "my_auction_purse";
pub const AUCTION_CONTRACT: &str = "auction_contract";
