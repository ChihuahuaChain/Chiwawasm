# Details

This contract allows for the creation of a liquidity pool, with the following properties.

1. A user can create a new instance of the contract and set himself as admin.

2. Anyone can call the contract with an amount to burn, the contract takes that value and calculates 
   the total amount to be burned.

   ```
   total = value_received + add_from_contract_balance(value_received * multipler)
   ```

3. Admin can update the `multiplier` factor and the `max_balance_to_burn`.

4. Admin can withdraw contract balance at any time.

&nbsp;

## Messages

```rust
pub struct InstantiateMsg {
    pub max_balance_to_burn: Uint128,
    pub multiplier: u8,
}

pub enum ExecuteMsg {
    BurnTokens {
        amount: Uint128,
    },
    UpdateMaxBurnAmount {
        amount: Uint128,
    },
    UpdateMultiplier {
        val: u8,
    },
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },
}
```

&nbsp;

### To run unit tests located in the .cargo/config file

`$ RUST_BACKTRACE=1 cargo unit-test`

 &nbsp;
