#![cfg(test)]

use crate::testutils::{register_test_contract as register_auction, AuctionContract};
use crate::token::{self, TokenMetadata};
use rand::{thread_rng, RngCore};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::bigint;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{testutils::Accounts, AccountId, BigInt, BytesN, Env, IntoVal};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn create_test_token_contract(e: &Env, admin: &AccountId) -> ([u8; 32], token::Client) {
    let id = e.register_contract_token(None);
    let token = token::Client::new(e, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "TEST coin".into_val(e),
            symbol: "TEST".into_val(e),
            decimals: 7,
        },
    );
    (id.into(), token)
}

fn create_usdc_contract(e: &Env, admin: &AccountId) -> ([u8; 32], token::Client) {
    let id = e.register_contract_token(None);
    let token = token::Client::new(e, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "USD coin".into_val(e),
            symbol: "USDC".into_val(e),
            decimals: 7,
        },
    );
    (id.into(), token)
}

fn create_auction_contract(
    e: &Env,
    admin: &AccountId,
    token_id: &[u8; 32],
    item_id: &[u8; 32],
    starting_price: BigInt,
    minimum_price: BigInt,
    slope: BigInt,
) -> ([u8; 32], AuctionContract) {
    let id = generate_contract_id();
    register_auction(&e, &id);
    let auction = AuctionContract::new(e, &id);
    auction.initialize(
        &Identifier::Account(admin.clone()),
        token_id,
        item_id,
        starting_price,
        minimum_price,
        slope,
    );
    (id, auction)
}

#[test]
fn test() {
    let e: Env = Default::default();
    let admin1 = e.accounts().generate(); // generating the usdc admin
    let admin2 = e.accounts().generate(); // generating the TEST token admin

    let user1 = e.accounts().generate(); // auction admin
    let user1_id = Identifier::Account(user1.clone());

    let user2 = e.accounts().generate(); // buyer
    let user2_id = Identifier::Account(user2.clone());

    // setting ledger time to a recent timestamp
    e.ledger().set(LedgerInfo {
        timestamp: 1666359075,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    let (usdc_id, usdc_token) = create_usdc_contract(&e, &admin1); // registered and initialized the usdc token contract
    let (test_token_id, test_token) = create_test_token_contract(&e, &admin2); // registered and initialized the TEST token contract

    // register and initializ the auction token contract, with usdc as auction token, test token as prize, a starting price of 5 usdc, a minimum price of 1 usdc, and a "slope" of 900, so that the auction reaches the minimum price after an hour \( 5 - (3600/900) = 1 = minimum_price \)
    let (contract_auction, auction) = create_auction_contract(
        &e,
        &user1,
        &usdc_id,                // auction token
        &test_token_id,          // prize token
        BigInt::from_u32(&e, 5), // starting price
        BigInt::from_u32(&e, 1), // minimum price
        bigint!(&e, 900),        // slope
    );

    let auction_id = Identifier::Contract(BytesN::from_array(&e, &contract_auction)); // the id of the auction

    // minting 10 test token to user1
    test_token.with_source_account(&admin2).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &BigInt::from_u32(&e, 10),
    );

    // user 1 deposits 10 TEST token into the auction as prize
    test_token.with_source_account(&user1).xfer(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &auction_id,
        &BigInt::from_u32(&e, 10),
    );

    // minting 1000 usdc to user2
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user2_id,
        &BigInt::from_u32(&e, 1000),
    );

    // re-adjust ledger timestamp so that 1800 seconds have passed (30 min)
    e.ledger().set(LedgerInfo {
        timestamp: 1666360875,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });

    // user2 deposits \(starting_price - (\delta_time / slope) \) usdc into auction, so 3 usdc \((5 - (1800 / 900)) = 3\)
    usdc_token.with_source_account(&user2).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &auction_id,
        &auction.get_price(),
    );

    // user2 enters the auction
    auction.buy(user2_id.clone());

    // the buyer (user2) should have \(1000 - 3\) as usdc balance
    assert_eq!(usdc_token.balance(&user2_id), 997);

    // the auction admin (user1) should have \( 3\) as usdc token balance (since user one bought in the auction at a price of 3)
    assert_eq!(usdc_token.balance(&user1_id), 3);

    // the buyer (user2) should have \( 10 \) as TEST token balance (bought at the auction)
    assert_eq!(test_token.balance(&user2_id), 10);
}
