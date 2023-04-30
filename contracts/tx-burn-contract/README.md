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
    pub max_extra_balance_to_burn_per_tx: Uint128,
    pub multiplier: u8,
}

pub enum ExecuteMsg {
    BurnTokens {
        amount: Uint128,
    },
    UpdatePreferences {
        max_extra_burn_amount_per_tx: Option<Uint128>,
        multiplier: Option<u8>,
    },
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },
}
```

&nbsp;

## Queries

```rust
pub enum QueryMsg {
     Info {},
}

pub struct Config {
    pub admin: Addr,
    pub max_extra_balance_to_burn_per_tx: Uint128,
    pub multiplier: u8,
}
```

&nbsp;

### To run unit tests located in the .cargo/config file

`$ RUST_BACKTRACE=1 cargo unit-test`

 &nbsp;

## How to test

### Build contract from source

`$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6`

&nbsp;

### Store the contract on the blockchain and get the <CODE_ID>

`$ export RES=$(chihuahuad tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)`

`$ echo $RES`

`$ export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')`

`echo $CODE_ID`

&nbsp;

### Instantiate the contract

#### Prepare the json message payload

```zsh
$ export INIT='{"max_extra_balance_to_burn_per_tx":"2000000","multiplier":2}';

$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "TX BURN CONTRACT" $TXFLAG -y --no-admin
```

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

`$ export CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')`

`$ echo $CONTRACT`

&nbsp;

### Send some funds to $CONTRACT

`chihuahuad tx bank send <from_account> $CONTRACT 2000000uhuahua $TXFLAG`

&nbsp;

### Check $CONTRACT balance

`chihuahuad query bank balances $CONTRACT $NODE`

`$ chihuahuad query bank balances $(chihuahuad keys show -a cryptoql) $NODE`

&nbsp;

### Test BurnTokens

```zsh
export E_PAYLOAD='{"burn_tokens":{"amount": "1000000"}}'

chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --from <account_name> --amount=1000000uhuahua  $NODE $TXFLAG -y
```

&nbsp;

### Test UpdatePreferences

```zsh
// Note: max_extra_burn_amount_per_tx and multiplier are optional
export E_PAYLOAD='{"update_preferences":{"max_extra_burn_amount_per_tx": "5000000", "multiplier":"3"}}'

chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --from <account_name> $NODE $TXFLAG -y
```

&nbsp;

### Test WithdrawBalance

```zsh
// Note: to_address is optional
export E_PAYLOAD='{"withdraw_balance": {"funds": {"amount": "1000000", "denom":"uhuahua"}}}'

chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --from <account_name> $NODE $TXFLAG -y
```

&nbsp;