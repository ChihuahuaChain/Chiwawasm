# Details

The cw20 tokens manager allow users to pay a `token_creation_fee` to mint cw20 tokens managed by this contract.

&nbsp;

## Messages

```rust
pub struct InstantiateMsg {
    pub token_creation_fee: Coin,
    pub token_code_id: u64,
}

pub enum ExecuteMsg {
    CreateToken { token_info: TokenInfo },
}
```

&nbsp;

## Queries

```rust
pub enum QueryMsg {
    Config {},
    QueryTokenList {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

pub struct TokenListResponse {
    pub entries: Vec<Entry>,
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

### TODO Instantiate the contract

#### Prepare the json message payload

```javascript
// First lets get the variables ready
let init_msg = JSON.stringify({
    token_creation_fee: {
        denom: "stake",
        amount: "10000000000"
    },
    token_code_id: 2,
});

export INIT='<init_msg>'


$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "BURN TEST CONTRACT" $TXFLAG -y --no-admin
```

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

`$ export CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')`

`$ echo $CONTRACT`

&nbsp;

### Inspect contract config

```zsh
export Q_PAYLOAD='{"config":{}}'

chihuahuad query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json
```

&nbsp;

### Test CreateToken

#### Prepare execute json message payload

```javascript
// First lets get the variables ready
let init_msg = JSON.stringify(
    { create_token: {
        token_info: {
            name: 'My CW20 Contract',
            symbol: 'HCW',
            decimals: 6,
            initial_balances: [{
                address: 'chihuahua1ehkt6apyy5sxeccw3dfgq8epntn6trfgxgjxtw',
                amount: '9999'
            }],
            mint: {
                minter: 'chihuahua1ehkt6apyy5sxeccw3dfgq8epntn6trfgxgjxtw',
                cap: '9999',
            },
            marketing: {
                project: 'My CW20 Contract',
                description: 'This is my cw20 contract',
                marketing: 'chihuahua1ehkt6apyy5sxeccw3dfgq8epntn6trfgxgjxtw',
                logo: {
                    url: 'https://example.com/image.jpg',
                },
            },
        }
    }}
);

export E_PAYLOAD='<init_msg>'

$ chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --from cryptoql --amount=10000000000stake $NODE $TXFLAG -y
```

&nbsp;

### Query QueryTokenList

export Q_PAYLOAD='{"query_token_list":{}}'

chihuahuad query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json