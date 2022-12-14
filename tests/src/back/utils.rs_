use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT,
    MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::core::engine_state::ExecuteRequest;
use casper_types::{
    account::AccountHash,
    runtime_args,
    system::mint::{ARG_ID, ARG_TARGET},
    RuntimeArgs, U512,
};

use super::{
    constants::{
        CONTRACT_AUCTION, CONTRACT_ECP47_TOKEN, CONTRACT_ECP78_TOKEN, CONTRACT_KYC,
        KEY_AUCTION_CONTRACT_HASH, KEY_AUCTION_PACKAGE_HASH, KEY_ECP47_CONTRACT_HASH,
        KEY_ECP47_PACKAGE_HASH, KEY_ECP78_CONTRACT_HASH, KEY_ECP78_PACKAGE_HASH,
        KEY_KYC_CONTRACT_HASH, KEY_KYC__PACKAGE_HASH, SESSION_BID_PURSE,
    },
    enums::TypeDeploy,
    test_auction::{KYC, NFT_ECP47, NFT_ECP78},
};

pub fn get_session_file(type_deploy: TypeDeploy) -> Option<&'static str> {
    match type_deploy {
        KYC => Some(CONTRACT_KYC),
        NFT_ECP47 => Some(CONTRACT_ECP47_TOKEN),
        NFT_ECP78 => Some(CONTRACT_ECP78_TOKEN),
        TypeDeploy::Auction(_) => Some(CONTRACT_AUCTION),
        TypeDeploy::Bid(_, _) => Some(SESSION_BID_PURSE),
        _ => None,
    }
}

pub fn get_contracts_name_constants(type_deploy: TypeDeploy) -> (&'static str, &'static str) {
    match type_deploy {
        KYC => (KEY_KYC_CONTRACT_HASH, KEY_KYC__PACKAGE_HASH),
        NFT_ECP47 => (KEY_ECP47_CONTRACT_HASH, KEY_ECP47_PACKAGE_HASH),
        NFT_ECP78 => (KEY_ECP78_CONTRACT_HASH, KEY_ECP78_PACKAGE_HASH),
        TypeDeploy::Auction(_) => (KEY_AUCTION_CONTRACT_HASH, KEY_AUCTION_PACKAGE_HASH),
        _ => unimplemented!(),
    }
}

pub fn fund_account(account: &AccountHash) -> ExecuteRequest {
    let deploy_item = DeployItemBuilder::new()
        .with_address(*DEFAULT_ACCOUNT_ADDR)
        .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
        .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
        .with_transfer_args(runtime_args! {
            ARG_AMOUNT => U512::from(MINIMUM_ACCOUNT_CREATION_BALANCE),
            ARG_TARGET => *account,
            ARG_ID => <Option::<u64>>::None
        })
        .with_deploy_hash([1; 32])
        .build();

    ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
}
