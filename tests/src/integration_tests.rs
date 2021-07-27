#[allow(unused)]
#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash,
        bytesrepr::FromBytes,
        CLTyped,
        Key,
        PublicKey,
        RuntimeArgs,
        SecretKey,
        U512,
        //runtime_args, URef,
    };

    pub const CONTRACT_HASH_NAMED_KEY: &str = "contract_hash";

    pub struct AccountInfoContract {
        pub context: TestContext,
        pub contract_hash: Hash,
        pub admin: AccountHash,
        pub user: AccountHash,
    }

    impl AccountInfoContract {
        pub fn deploy() -> Self {
            // Create admin.
            let admin_secret = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
            let admin_key: PublicKey = (&admin_secret).into();
            let admin_addr = AccountHash::from(&admin_key);

            // Create plain user.
            let user_secret = SecretKey::ed25519_from_bytes([2u8; 32]).unwrap();
            let user_key: PublicKey = (&user_secret).into();
            let user_addr = AccountHash::from(&user_key);

            // Create context.
            let mut context = TestContextBuilder::new()
                .with_public_key(admin_key, U512::from(500_000_000_000_000_000u64))
                .with_public_key(user_key, U512::from(500_000_000_000_000_000u64))
                .build();

            // Deploy the main contract onto the context.
            let session_code = Code::from("account-info.wasm");
            let session = SessionBuilder::new(session_code, RuntimeArgs::new())
                .with_address(admin_addr)
                .with_authorization_keys(&[admin_addr])
                .build();
            context.run(session);

            let contract_hash = context
                .query(admin_addr, &[CONTRACT_HASH_NAMED_KEY.to_string()])
                .unwrap()
                .into_t()
                .unwrap();

            Self {
                context,
                contract_hash,
                admin: admin_addr,
                user: user_addr,
            }
        }

        /// Query a non-dictionary entry from the TestContext
        fn query<T: FromBytes + CLTyped>(&self, key: &str) -> T {
            println!("{:?}", key);
            self.context
                .query(
                    self.admin,
                    &[CONTRACT_HASH_NAMED_KEY.to_string(), key.to_string()],
                )
                .unwrap()
                .into_t()
                .unwrap()
        }

        /// Query a dictionary entry from the TestContext
        fn query_dictionary_value<T: CLTyped + FromBytes>(
            &self,
            dict_name: &str,
            key: &str,
        ) -> Option<T> {
            match self.context.query_dictionary_item(
                Key::Hash(self.contract_hash),
                Some(dict_name.to_string()),
                key.to_string(),
            ) {
                Err(_) => None,
                Ok(maybe_value) => {
                    let value = maybe_value
                        .into_t()
                        .unwrap_or_else(|_| panic!("is not expected type."));
                    Some(value)
                }
            }
        }

        /// Call a and entrypoint with the give argument on the TestContest.
        fn call(&mut self, caller: &AccountHash, function: &str, args: RuntimeArgs) {
            let session_code = Code::Hash(self.contract_hash, function.to_string());
            let session = SessionBuilder::new(session_code, args)
                .with_address(*caller)
                .with_authorization_keys(&[*caller])
                .build();
            self.context.run(session);
        }
    }

    #[test]
    fn test_set_deposit() {
        let _contract = AccountInfoContract::deploy();
    }

    #[test]
    #[should_panic]
    fn test_set_deposit_security() {
        let _contract = AccountInfoContract::deploy();
    }
}

fn main() {
    panic!("The main should not be used here");
}
