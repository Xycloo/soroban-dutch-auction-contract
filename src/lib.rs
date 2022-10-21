#![no_std]

#[cfg(feature = "testutils")]
extern crate std;

mod test;
pub mod testutils;

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, BigInt, BytesN, Env};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    TokenId,
    ItemId,
    Price,
    MinPrice,
    Timestamp,
    Slope,
    Nonce(Identifier),
}

#[derive(Clone)]
#[contracttype]
pub struct Auth {
    pub sig: Signature,
    pub nonce: BigInt,
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

fn put_minimum_price(e: &Env, price: BigInt) {
    let key = DataKey::MinPrice;
    e.data().set(key, price);
}

fn get_minimum_price(e: &Env) -> BigInt {
    let key = DataKey::MinPrice;
    e.data().get(key).unwrap_or(Ok(BigInt::zero(&e))).unwrap()
}

fn put_starting_price(e: &Env, price: BigInt) {
    let key = DataKey::Price;
    e.data().set(key, price);
}

fn get_starting_price(e: &Env) -> BigInt {
    let key = DataKey::Price;
    e.data().get(key).unwrap_or(Ok(BigInt::zero(&e))).unwrap()
}

fn put_slope(e: &Env, slope: BigInt) {
    let key = DataKey::Slope;
    e.data().set(key, slope);
}

fn get_slope(e: &Env) -> BigInt {
    let key = DataKey::Slope;
    e.data().get(key).unwrap().unwrap()
}

fn put_starting_time(e: &Env, time: u64) {
    let key = DataKey::Timestamp;
    e.data().set(key, time);
}

fn get_starting_time(e: &Env) -> u64 {
    let key = DataKey::Timestamp;
    e.data().get(key).unwrap().unwrap()
}

fn put_token_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::TokenId;
    e.data().set(key, token_id);
}

fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    e.data().get(key).unwrap().unwrap()
}

fn put_item_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::ItemId;
    e.data().set(key, token_id);
}

fn get_item_id(e: &Env) -> BytesN<32> {
    let key = DataKey::ItemId;
    e.data().get(key).unwrap().unwrap()
}

fn transfer_to_admin(e: &Env, from: &Identifier, amount: BigInt) {
    let client = token::Client::new(e, get_token_id(e));
    let admin_id = read_administrator(e);

    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(e),
        from,
        &admin_id,
        &amount,
    )
}

fn empty_contract(e: &Env, to: Identifier) {
    let client = token::Client::new(e, get_item_id(e));
    let amount = client.balance(&get_contract_id(e));
    client.xfer(&Signature::Invoker, &BigInt::zero(e), &to, &amount)
}

fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.data().has(key)
}

fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.data().get_unchecked(key).unwrap()
}

fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.data().set(key, id);
}

fn read_nonce(e: &Env, id: &Identifier) -> BigInt {
    let key = DataKey::Nonce(id.clone());
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

fn compute_price(e: &Env) -> BigInt {
    let starting_price = get_starting_price(e);
    let minimum_price = get_minimum_price(e);
    let starting_time = get_starting_time(e);
    let current_time = e.ledger().timestamp();
    let elapsed_time = current_time - starting_time;
    let rev_slope = get_slope(e);

    let computed = starting_price - BigInt::from_u64(e, elapsed_time) / rev_slope;

    if computed < minimum_price {
        minimum_price
    } else {
        computed
    }
}

pub trait AuctionContractTrait {
    // Sets the admin, the auction's token id, the prize item id, the starting auction price, the minimum auction price, and an "inverse slope" \( \delta_time / slope \)
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        item_id: BytesN<32>,
        starting_price: BigInt,
        minimum_price: BigInt,
        slope: BigInt,
    );

    // Returns the nonce for the admin
    fn nonce(e: Env) -> BigInt;

    // user "from" enters the auction at its current price
    fn buy(e: Env, from: Identifier);

    // fetch the current price of the auction
    fn get_price(e: Env) -> BigInt;
}

pub struct AuctionContract;

#[contractimpl]
impl AuctionContractTrait for AuctionContract {
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        item_id: BytesN<32>,
        starting_price: BigInt,
        minimum_price: BigInt,
        slope: BigInt,
    ) {
        if has_administrator(&e) {
            panic!("admin is already set");
        }

        let time = e.ledger().timestamp();

        write_administrator(&e, admin);
        put_token_id(&e, token_id);
        put_item_id(&e, item_id);
        put_starting_price(&e, starting_price);
        put_starting_time(&e, time);
        put_minimum_price(&e, minimum_price);
        put_slope(&e, slope);
    }

    fn nonce(e: Env) -> BigInt {
        read_nonce(&e, &read_administrator(&e))
    }

    fn buy(e: Env, from: Identifier) {
        let price = compute_price(&e);
        transfer_to_admin(&e, &from, price);
        empty_contract(&e, from);
    }

    fn get_price(e: Env) -> BigInt {
        compute_price(&e)
    }
}
