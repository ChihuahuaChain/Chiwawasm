# Chiwawasm

[![Rust](https://github.com/ChihuahuaChain/CosmWasm/actions/workflows/rust.yml/badge.svg)](https://github.com/ChihuahuaChain/CosmWasm/actions/workflows/rust.yml)

&nbsp;

This repository is a collection of smart contracts built with the
[cosmwasm](https://github.com/CosmWasm/cosmwasm) framework.

The repo's organization is relatively simple. The top-level directory is just a placeholder
and has no real code. And we use workspaces to add multiple contracts below.
This allows us to compile all contracts with one command.

&nbsp;

## Usage

The following contracts are available for use. For each of the contracts in `./contracts`, you can view the source code under `src`

&nbsp;

| Contracts (tag: v0.1.0)                                                        | Description                                                        |
| :----------------------------------------------------------------------------- | :----------------------------------------------------------------- |
| [Burn-Contract](contracts/burn-contract)                                       | A basic contract to burn token balances.                           |
| [Tx-Burn-Contract](contracts/tx-burn-contract)                                 | Burns tokens sent by the caller along with extra from contract     |
| [Tokens-Manger](contracts/tokens-manager)                                      | Manages the creation of new cw20 tokens for a fee                  |
| [Token-Swap](contracts/token-swap)                                             | Allows swapping between tokens                                     |


&nbsp;

## Preparing for merge

Before you merge the code, make sure it builds and passes all tests using the command below.

`$ cargo test`

&nbsp;

## Release builds

You can build release artifacts manually like this, which creates a reproducible
optimized build for each contract and saves them to the `./artifacts` directory:

```zsh
$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```

&nbsp;


## Working with smart contracts (chihuahuad)

(See [instructions](https://github.com/ChihuahuaChain/chihuahua/blob/main/README.md) on how install `chihuahuad`)

&nbsp;

### View chihuahuad config variables

```zsh
$ open ~/.chihuahua/config/config.toml
```

&nbsp;

### Generate a new cosm-wasm project from template

```zsh
$ cargo install cargo-generate --features vendored-openssl

$ cargo generate --git <https://github.com/CosmWasm/cosmwasm-template.git> --name my-first-contract
```
&nbsp;

### Testnet block explorers

https://testnet.ping.pub/chihuahua

https://testnet.explorer.erialos.me/chihuahua

&nbsp;

### Add test accounts

```zsh
$ chihuahuad keys add <wallet_name> --recover
```

&nbsp;

### List test accounts

```zsh
$ chihuahuad keys list
```

&nbsp;

### Export variables for use in terminal (zsh)

```zsh
$ source ~/.profile

$ export CHAIN_ID="chitestnet-5"

$ export RPC="https://chihuahua-testnet-rpc.polkachu.com:443"

$ export NODE=(--node $RPC)

$ export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.25stake --gas auto --gas-adjustment 1.3)

$ export TXFLAG_LOCAL=(--gas-prices 0.25stake --gas auto --gas-adjustment 1.3 --keyring-backend test)
```

&nbsp;

### Query balances

```zsh
$ chihuahuad query bank total $NODE

$ chihuahuad query bank balances $(chihuahuad keys show -a palingram) $NODE
```

&nbsp;

### Send funds to other account

```zsh
$ chihuahuad tx bank send <sender_account_name> <receiver_address> <amount><denom> $TXFLAG
```

&nbsp;

### See the list of code uploaded to the testnet

```zsh
$ chihuahuad query wasm list-code $NODE
```

&nbsp;


### Store the contract on the blockchain and get the <CODE_ID>

```zsh
$ export RES=$(chihuahuad tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)

$ export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')

$ echo $CODE_ID
```

&nbsp;

### Get a list of contracts instantiated for <CODE_ID>

```zsh
$ chihuahuad query wasm list-contract-by-code $CODE_ID $NODE --output json
```

&nbsp;

### Prepare the json message payload

```zsh
$ export INIT='{}'
```

&nbsp;

### Instantiate the contract

```zsh
$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "BURN TEST CONTRACT" $TXFLAG -y --no-admin
```

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

```zsh
$ export CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

$ echo $CONTRACT
```

&nbsp;

### Check the contract details

```zsh
$ chihuahuad query wasm contract $CONTRACT $NODE
```

&nbsp;

### Check the contract balance

```zsh
$ chihuahuad query bank balances $CONTRACT $NODE
```

&nbsp;

### query the entire contract state

```zsh
$ chihuahuad query wasm contract-state all $CONTRACT $NODE
```

&nbsp;

### query the data for a storage key in the contract-state directly

```zsh
$ chihuahuad query wasm contract-state raw $CONTRACT 636F6E666967 $NODE  --output "json" | jq -r '.data' | base64 -d
```

&nbsp;

### Calling execute methods

```zsh
$ export E_PAYLOAD='{"burn_contract_balance":{}}'

$ chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --amount 1000000000stake --from palingram $NODE $TXFLAG -y
```

&nbsp;

### calling query methods

```zsh
$ export Q_PAYLOAD='{"query_list":{}}'

$ chihuahuad query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json
```
