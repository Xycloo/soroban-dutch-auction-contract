# Soroban Dutch Auction Smart Contract

Dutch auctions are one of the most popular auction types since it works well and scales easily given its simplicity. In a Dutch auction, the auctioneer initializes the auction with a starting price, which is lowered as time passes until a bid is received. The $price = f(\Delta time)$ function does not have to conform to any specific function group, but in this auction contract we resembled a linear function since Soroban doesn't currently have support for fixed point math.

## Contract Description
Such auction contract should provide three invocation functions:
#### initialize
This is the invocation that the auction's admin will use to set up the auction. The admin should specify
  - the token to be used in the auction (for example USDC)
  - the prize token id, so the prize of the auction
  - the already-mentioned starting price for the auction
  - the minimum price (or reserve price), so that the auction's prize doens't reach a price of 0 by default
  - a slope, so how fast the price decreases over time. Keep in mind that in this contract implementation, it is more an "inverse slope", meaning that the price equation won't look like this $y = -mx + k$, but like this $y = \frac{x}{-m} + k$ where $y$ is the price, $x$ is the time that has passed since the initialization of the auction, $k$ is the starting price, and $m$ is our "inverse slope". This means that our slope is inversely proportional to the speed to which the price decreases.
  
```rust
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

        let time = e.ledger().timestamp(); // fetch the current time

        write_administrator(&e, admin);
        put_token_id(&e, token_id);
        put_item_id(&e, item_id);
        put_starting_price(&e, starting_price);
        put_starting_time(&e, time);
        put_minimum_price(&e, minimum_price);
        put_slope(&e, slope);
    }
```

These auction settings are saved in the contract data with the `env.data().put(KEY, VALUE)`. For example, the `put_starting_time()` function does the following:
  
```rust
  fn put_starting_time(e: &Env, time: u64) {
    let key = DataKey::Timestamp;
    e.data().set(key, time);
}
```
This data entry can then be read with the `env.data().get(KEY)`, as we do [here](https://github.com/Xycloo/soroban-dutch-auction-contract/blob/main/src/lib.rs#L77).

  
#### buy (or placing a bid) 
Here a bidder buys the prize item at the current computed price. The buyer is transferring money (i.e the auction's token) to the auction's admin, then the contract empties itself by sending all of its prize tokens to the buyer

```rust
fn buy(e: Env, from: Identifier) {
    let price = compute_price(&e);
    transfer_to_admin(&e, &from, price); // bidder pays the auction's admin
    empty_contract(&e, from); // transfer the prize from the contract to the bidder
}
```

where `compute_price` simply solves the above mentioned price equation $f(\Delta price) = \frac{x}{-m} + k$:

```rust
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
```

#### get_price 
A simple getter that returns the computed price for the current timestamp. This one just calls the `compute_price` fn when invoked.



## Testing the Contract
The testing workflow is the follwing:
1. create admin accounts for the two standard tokens we are going to use (USDC and TEST token).
2. create accounts for two users, user1 (the admin) and user2 (the bidder).
3. set the ledger's time to a recent timestamp.
4. register and initialize the two token contracts
5. register and initialize the auction contract with:
- USDC as auction token.
- TEST token as auction prize.
- starting price of 5 usdc.
- minimum price of 1 usdc.
- slope of 900, so that the auction reaches the minimum price in an hour (3600, since $5 - (3600/900) = 1 = minimum price$).
- mint TEST tokens to the admin, and then transfer them into the contract (this is the prize).
- mint usdc to the bidder so that they can bid.
- update ledger time to simulate $\Delta time = 1800$, which is the same as saying that half an hour has passed since the beginning of the auction.
- allow the auction contract to transfer $n$ usdc out of the bidder's account, where $n$ is the computed price. 
- bidder makes the bid.
- finally, make all required assertion to verify that the auction worked as excpected.

```bash
❯ git clone https://github.com/Xycloo/soroban-dutch-auction-contract/
❯ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.03s
     Running unittests src/lib.rs (target/debug/deps/soroban_dutch_auction_contract-b58f3f171dd11289)

running 1 test
test test::test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests soroban-dutch-auction-contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

