# Details

This contract allows for the creation of a liquidity pool, with the following properties.

1. Any client can attempt to burn `daily_burn_amount` of the contract's balance by calling the `BurnDailyQuota` execute method of the contract once every `burn_delay_seconds`.

2. `sudo` method `SetMaxDailyBurn`  which can only be called by raising an onchain proposal targeting this contract.

2. `sudo` method `WithdrawFundsToCommunityPool`  which can only be called by raising an onchain proposal targeting this contract.

&nbsp;

### Messages

```rust
pub struct InstantiateMsg {
    pub native_denom: String,
    pub daily_burn_amount: u128,
    pub burn_delay_seconds: u64,
}

pub enum ExecuteMsg {
    BurnDailyQuota {},
}

pub enum SudoMsg {
    SetMaxDailyBurn { amount: Uint128 },
    WithdrawFundsToCommunityPool { 
        address: String 
    },
}
```
&nbsp;

### Queries

```rust
pub enum QueryMsg {
    Config {},
    Balance {},
}

pub struct BalanceResponse {
    pub amount: Coin,
}
```

&nbsp;

### To run unit tests located in the .cargo/config file

`$ RUST_BACKTRACE=1 cargo unit-test`

 &nbsp;

### To compile an optimized build with a docker image

`$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6`

  &nbsp;
