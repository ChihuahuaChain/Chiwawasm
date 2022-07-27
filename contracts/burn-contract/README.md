# Details

This contract allows for the creation of a liquidity pool, with the following properties.

1. Any client can attempt to burn `daily_burn_amount` of the contract's balance by calling the `BurnDailyQuota` execute method of the contract once every `burn_delay_seconds`.

2. `sudo` method `SetMaxDailyBurn`  which can only be called by raising an on-chain proposal targeting this contract.

3. `sudo` method `WithdrawFundsToCommunityPool`  which can only be called by raising an on-chain proposal targeting this contract.

&nbsp;

## Messages

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

## Queries

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

## How to test

---

### Build contract from source

`$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6`

&nbsp;

### Store the contract on the blockchain and get the <CODE_ID>

`$ export RES=$(chihuahuad tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)`

`$ echo $RES`

`$ export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')`

`echo $CODE_ID`

&nbsp;

### Instantiate the contract

#### Prepare the json message payload

```javascript
// First lets get the variables ready
let init_msg = JSON.stringify({
    native_denom: "stake",
    daily_burn_amount: "100",
    burn_delay_seconds: 60,
});

export INIT='{"native_denom":"stake","daily_burn_amount":"100","burn_delay_seconds":60}';


$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "BURN TEST CONTRACT" $TXFLAG -y --no-admin
```

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

`$ export CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')`

`$ echo $CONTRACT`

&nbsp;

### Send some funds to $CONTRACT

`chihuahuad tx bank send cryptoql $CONTRACT 300stake $TXFLAG`

&nbsp;

### Check $CONTRACT balance

`chihuahuad query bank balances $CONTRACT $NODE`

`$ chihuahuad query bank balances $(chihuahuad keys show -a cryptoql) $NODE`

&nbsp;

### Test the burndailyquota

```zsh
export E_PAYLOAD='{"burn_daily_quota":{}}'

chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --from cryptoql $NODE $TXFLAG -y
```

&nbsp;

### Delegate some tokens to a testnet validator so we have some voting rights

```zsh
chihuahuad q staking validators $NODE --output json
chihuahuavaloper1892e7rwlzk0q6rmpy0ptk9dx8hnn08gx9f5ht8
chihuahuavaloper13gfuwmfufm2tx270j68wl7549gk5q3fyygyd3k

// Bond some tokens any of the above validators
export VALIDATOR="chihuahuavaloper13gfuwmfufm2tx270j68wl7549gk5q3fyygyd3k"
chihuahuad tx staking delegate $VALIDATOR 809824402463865stake --from cryptoql $TXFLAG  -y --output json
```

&nbsp;

### Raise a governance proposal SetMaxDailyBurn

```zsh
export PROPOSAL='{"set_max_daily_burn": {"amount": "150"}}'

chihuahuad tx gov submit-proposal sudo-contract $CONTRACT $PROPOSAL \
    --from cryptoql \
    --title "Change the daily burn amount" \
    --description "LFG" \
    --type sudo-contract \
    --deposit 10000000stake $NODE $TXFLAG -y --output json
```

&nbsp;

### Get the latest proposal_id

`chihuahuad q gov proposals $NODE`

`chihuahuad q gov proposal <proposal_id> $NODE`

&nbsp;

### Vote to make the proposal pass

`chihuahuad tx gov vote  <proposal_id> yes --from <account_name> $NODE --chain-id chitestnet-1 --gas-prices 0.25stake --output json`

&nbsp;

### Query contract config data to see if daily burn has been updated

```zsh
export Q_PAYLOAD='{"config":{}}'

chihuahuad query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json
```

&nbsp;

### Raise a governance proposal to WithdrawFundsToCommunityPool

```zsh
export PROPOSAL='{"withdraw_funds_to_community_pool": {"address": "chihuahua1u0nmsyuttatkmrnmnxm3243c2hkk5w7jd0u442"}}'

chihuahuad tx gov submit-proposal sudo-contract $CONTRACT $PROPOSAL \
    --from cryptoql \
    --title "Withdraw funds to community pool" \
    --description "LFG" \
    --type sudo-contract \
    --deposit 10000000stake $NODE $TXFLAG -y --output json
```
