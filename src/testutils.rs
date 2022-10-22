#![cfg(any(test, feature = "testutils"))]

use crate::AuctionContractClient;
use soroban_auth::Identifier;

use soroban_sdk::{BigInt, BytesN, Env};

pub fn register_test_contract(e: &Env, contract_id: &[u8; 32]) {
    let contract_id = BytesN::from_array(e, contract_id);
    e.register_contract(&contract_id, crate::AuctionContract {});
}

pub struct AuctionContract {
    env: Env,
    contract_id: BytesN<32>,
}

impl AuctionContract {
    fn client(&self) -> AuctionContractClient {
        AuctionContractClient::new(&self.env, &self.contract_id)
    }

    pub fn new(env: &Env, contract_id: &[u8; 32]) -> Self {
        Self {
            env: env.clone(),
            contract_id: BytesN::from_array(env, contract_id),
        }
    }

    pub fn initialize(
        &self,
        admin: &Identifier,
        token_id: &[u8; 32],
        //        item_id: &[u8; 32],
        starting_price: BigInt,
        minimum_price: BigInt,
        slope: BigInt,
    ) {
        self.client().initialize(
            admin,
            &BytesN::from_array(&self.env, token_id),
            //            &BytesN::from_array(&self.env, item_id),
            &starting_price,
            &minimum_price,
            &slope,
        );
    }

    pub fn nonce(&self) -> BigInt {
        self.client().nonce()
    }

    pub fn buy(&self, from: Identifier) -> bool {
        self.client().buy(&from)
    }

    pub fn get_price(&self) -> BigInt {
        self.client().get_price()
    }
}
