// Wasm names
pub const CONTRACT_KYC: &'static str = "kyc-contract.wasm";
pub const CONTRACT_ECP47_TOKEN: &'static str = "ecp47_token.wasm";
pub const CONTRACT_ECP78_TOKEN: &'static str = "ecp78_token.wasm";
pub const CONTRACT_AUCTION: &'static str = "casper-private-auction-installer.wasm";
pub const SESSION_BID_PURSE: &'static str = "bid-purse.wasm";

// Named Keys
pub const KEY_ECP47_CONTRACT_NAME: &'static str = "ecp47_token";
pub const KEY_ECP47_CONTRACT_HASH: &'static str = "ecp47_token_contract_hash";
pub const KEY_ECP47_PACKAGE_HASH: &'static str = "ecp47_token_package_hash";
pub const KEY_ECP78_CONTRACT_NAME: &'static str = "ecp78_token";
pub const KEY_ECP78_CONTRACT_HASH: &'static str = "ecp78_token_contract_hash";
pub const KEY_ECP78_PACKAGE_HASH: &'static str = "ecp78_token_package_hash";
pub const KEY_KYC_CONTRACT_NAME: &'static str = "kyc_token";
pub const KEY_KYC_CONTRACT_HASH: &'static str = "kyc_token_contract_hash";
pub const KEY_KYC__PACKAGE_HASH: &'static str = "kyc_token_package_hash";
pub const KEY_AUCTION_CONTRACT_HASH: &'static str = "AUCTION_auction_contract_hash";
pub const KEY_AUCTION_PACKAGE_HASH: &'static str = "AUCTION_auction_contract_package_hash";

// Entry points
pub const ENTRY_POINT_GRANT_GATEKEEPER: &'static str = "grant_gatekeeper";

// Config
pub const AUCTION_NAME: &'static str = "AUCTION";
pub const TOKEN_ECP47_NAME: &'static str = "ECP47_TOKEN";
pub const TOKEN_ECP47_SYMBOL: &'static str = "ECP47";
pub const TOKEN_ECP78_NAME: &'static str = "ECP78_TOKEN";
pub const TOKEN_ECP78_SYMBOL: &'static str = "ECP78";
pub const TOKEN_KYC_NAME: &'static str = "KYC_TOKEN";
pub const TOKEN_KYC_SYMBOL: &'static str = "KYC";

// Runtime Args name
pub const RUNTIME_ARG_NAME_NAME: &'static str = "name";
pub const RUNTIME_ARG_NAME_SYMBOL: &'static str = "symbol";
pub const RUNTIME_ARG_NAME_META: &'static str = "meta";
pub const RUNTIME_ARG_ADMIN: &'static str = "admin";
pub const RUNTIME_ARG_CONTRACT_NAME: &'static str = "contract_name";
pub const RUNTIME_ARG_GATEKEEPER: &'static str = "gatekeeper";
pub const RUNTIME_ARG_RECIPIENT: &'static str = "recipient";

// This const can be found in data.rs
pub const TOKEN_ID: &str = "token_id";
pub const TOKEN_META: &str = "token_meta";

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
