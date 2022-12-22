// Wasm names
pub const CONTRACT_KYC: &str = "kyc-contract.wasm";
pub const CONTRACT_CEP_47_TOKEN: &str = "cep_47.wasm";
pub const CONTRACT_CEP_78_TOKEN: &str = "cep_78.wasm";
pub const CONTRACT_AUCTION: &str = "casper-private-auction-installer.wasm";
pub const SESSION_BID_PURSE: &str = "bid-purse.wasm";
pub const SESSION_EXTENDED_BID_PURSE: &str = "extend-bid-purse.wasm";
pub const SESSION_DELTA_BID_PURSE: &str = "delta-bid-purse.wasm";

// Named Keys
pub const KEY_CEP_47_CONTRACT_NAME: &str = "cep_47_token";
pub const KEY_CEP_47_CONTRACT_HASH: &str = "cep_47_token_contract_hash";
pub const KEY_CEP_47_PACKAGE_HASH: &str = "cep_47_token_package_hash";

pub const KEY_CEP_78_CONTRACT_HASH: &str = "nft_contract";
pub const KEY_CEP_78_PACKAGE_HASH: &str = "nft_contract_package";

pub const KEY_KYC_CONTRACT_NAME: &str = "kyc";
pub const KEY_KYC_CONTRACT_HASH: &str = "kyc_contract_hash";
pub const KEY_KYC_PACKAGE_HASH: &str = "kyc_package_hash";

pub const KEY_AUCTION_CONTRACT_NAME: &str = "auction";
pub const KEY_AUCTION_CONTRACT_HASH: &str = "auction_contract_hash";
pub const KEY_AUCTION_PACKAGE_HASH: &str = "auction_package_hash";

pub const TOKEN_CEP_47_NAME: &str = "CEP_47_TOKEN";
pub const TOKEN_CEP_47_SYMBOL: &str = "CEP_47";
pub const TOKEN_CEP_78_NAME: &str = "CEP_78_TOKEN";
pub const TOKEN_CEP_78_SYMBOL: &str = "CEP_78";
pub const TOKEN_KYC_NAME: &str = "KYC_TOKEN";
pub const TOKEN_KYC_SYMBOL: &str = "KYC";

// Runtime Args name
pub const ARG_AUCTION_CONTRACT: &str = "auction_contract";
pub const ARG_NAME: &str = "name";
pub const ARG_SYMBOL: &str = "symbol";
pub const ARG_META: &str = "meta";
pub const ARG_ADMIN: &str = "admin";
pub const ARG_CONTRACT_NAME: &str = "contract_name";
pub const ARG_RECIPIENT: &str = "recipient";
pub const ARG_COLLECTION_NAME: &str = "collection_name";
pub const ARG_COLLECTION_SYMBOL: &str = "collection_symbol";
pub const ARG_TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub const ARG_OWNERSHIP_MODE: &str = "ownership_mode";
pub const ARG_NFT_KIND: &str = "nft_kind";
pub const ARG_JSON_SCHEMA: &str = "json_schema";
pub const ARG_IDENTIFIER_MODE: &str = "identifier_mode";
pub const ARG_NFT_METADATA_KIND: &str = "nft_metadata_kind";
pub const ARG_METADATA_MUTABILITY: &str = "metadata_mutability";
pub const ARG_TOKEN_META_DATA: &str = "token_meta_data";

// TODO fetch those const from core constants
pub const TOKEN_ID: &str = "token_id";
pub const TOKEN_META: &str = "token_meta";
// TOKEN_HASH = base16::encode_lower(&runtime::blake2b(TOKEN_META));
pub const TOKEN_HASH: &str = "14e3a85361c4b486208896334e391b627fc9f62b92766c7d52893881c679676d";
pub const ARG_TOKEN_OWNER: &str = "token_owner";
pub const START_TIME: &str = "start_time";
pub const BID: &str = "bid";

// Specific
pub const ENGLISH: &str = "ENGLISH";
pub const DUTCH: &str = "DUTCH";
pub const WRAPPED: &str = "wrapped";
pub const STATUS: &str = "status";
pub const ORIGIN: &str = "origin";
pub const ACTIVE: &str = "active";
pub const TOKEN_IDS: &str = "token_ids";
pub const TOKEN_METAS: &str = "token_metas";
pub const TOKEN_GAUGES: &str = "token_gauges";
pub const TOKEN_WAREHOUSES: &str = "token_warehouses";
pub const TOKEN_COMISSIONS: &str = "token_commissions";
pub const ARG_PURSE_NAME: &str = "purse_name";
pub const PURSE_NAME_VALUE: &str = "my_auction_purse";
pub const BENEFICIARY_ACCOUNT: &str = "beneficiary_account";
pub const TOKEN_CONTRACT_HASH: &str = "token_contract_hash";
pub const FORMAT: &str = "format";
pub const STARTING_PRICE: &str = "starting_price";
pub const RESERVE_PRICE: &str = "reserve_price";
pub const CANCELLATION_TIME: &str = "cancellation_time";
pub const END_TIME: &str = "end_time";
pub const BIDDER_COUNT_CAP: &str = "bidder_count_cap";
pub const AUCTION_TIMER_EXTENSION: &str = "auction_timer_extension";
pub const MINIMUM_BID_STEP: &str = "minimum_bid_step";
pub const MARKETPLACE_ACCOUNT: &str = "marketplace_account";
pub const MARKETPLACE_COMMISSION: &str = "marketplace_commission";
pub const HAS_ENHANCED_NFT: &str = "has_enhanced_nft";
