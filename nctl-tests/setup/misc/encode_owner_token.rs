#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! hex = "*"
//! casper-types = "*"
//! ```

// ------------------------------------
// Encoding owner and token number
// ------------------------------------
// Alexander Limonov
// alimonov@casperlabs.io
// ------------------------------------
// CHANGELOG:
// 8/28/2021 - created
#![allow(unused_doc_comments)]

use std::env;
use casper_types::{
    account, bytesrepr::ToBytes, CLTyped, Key, U256,
};

pub fn key_and_value_to_str<T: CLTyped + ToBytes>(key: &Key, value: &T) -> String {
    let bytes_a = key.to_bytes().ok();
    let bytes_b = value.to_bytes().ok();

    let mut bytes_a_val =
        match bytes_a {
            Some(a) => a.clone(),
            None => panic!("Error encountered converting key to bytes."),
        };

    let mut bytes_b_val =
        match bytes_b {
            Some(b) => b.clone(),
            None => panic!("Error encountered converting value to bytes."),
        };

    bytes_a_val.append(&mut bytes_b_val);

    let bytes = account::blake2b(bytes_a_val);
    hex::encode(bytes)
}

let args: Vec<String> = env::args().collect();

println!("Received the following arguments:");
for argument in env::args() {
    println!("{}", &argument);
}

let owner =
    match Key::from_formatted_str(&args[1]) {
        Ok(key) => key,
        Err(err) => panic!("Error encountered parsing owner key: {}.", err)
    };

let token_number =
    match U256::from_dec_str(&args[2]) {
        Ok(res) => res,
        Err(err) => panic!("Error encountered parsing token number: {}.", err)
    };

let result = key_and_value_to_str(&owner, &token_number);

println!("{}", result);