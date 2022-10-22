# Soroban Dutch Auction Smart Contract

Dutch auctions are one of the most popular auction types since it works well and scales easily given its simplicity. In a Dutch auction, the auctioneer initializes the auction with a starting price, which is lowered as time passes until a bid is received. The $price = f(\Delta time)$ function does not have to conform to any specific function group, but in this auction contract we resembled a linear function since Soroban doesn't currently have support for fixed point math.

This branch is for a contract which is more composable and can be easily used by another contract to run a auction with more customizations.
