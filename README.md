# Chiwawasm Repo

[![Rust](https://github.com/ChihuahuaChain/CosmWasm/actions/workflows/rust.yml/badge.svg)](https://github.com/ChihuahuaChain/CosmWasm/actions/workflows/rust.yml)

&nbsp;

This repo is a collection of smart contracts built with the
[cosmwasm](https://github.com/CosmWasm/cosmwasm) framework.

This repo's organization is relatively simple. The top-level directory is just a placeholder
and has no real code. And we use workspaces to add multiple contracts below.
This allows us to compile all contracts with one command.

## Usage

The following contracts are available for use. For each of the contracts in `contracts`, you can view the source code under `src`

* [Burn-Contract](https://github.com/ChihuahuaChain/CosmWasm/tree/main/contracts/burn-contract)

A basic contract to burn token balances

&nbsp;

* [Tokens-Manger](https://github.com/ChihuahuaChain/Chiwawasm/tree/main/contracts/tokens-manager)

The cw20 tokens manager allow users to pay a `token_creation_fee` to mint cw20 tokens managed by this contract

&nbsp;

* [Token-Swap](https://github.com/ChihuahuaChain/Chiwawasm/tree/main/contracts/token-swap)

A constant product (AMM) implementation that allows the trading of any `CW20` or `IBC` tokens quoted against the base token `HUAHUA`

&nbsp;

## Preparing for merge

Before you merge the code, make sure it builds and passes all tests using the command below.

`./devtools/build_test_all.sh`

&nbsp;

## Release builds

You can build release artifacts manually like this, which creates a reproducible
optimized build for each contract and saves them to the `./artifacts` directory:

`docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6 ./contracts/*/`

&nbsp;

## Chihuahuad: Working with smart contracts

### Add test accounts

`$ chihuahuad keys add <wallet_name> --recover`

&nbsp;

### List test accounts

`$ chihuahuad keys list`

&nbsp;

### View chihuahuad config variables

`$ open ~/.chihuahua/config/config.toml`

&nbsp;

### Block explorer

<https://testnet.explorer.erialos.me/chihuahua>

&nbsp;

### Source config env for use in the shell (zsh)

`$ source ~/.profile`

`$ export CHAIN_ID="chitestnet-1"`

`$ export RPC="<rpc_endpoint>"`

`$ export NODE=(--node $RPC)`

`$ export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.25stake --gas auto --gas-adjustment 1.3)`

`$ export TXFLAG_LOCAL=(--gas-prices 0.25stake --gas auto --gas-adjustment 1.3 --keyring-backend test)`

&nbsp;

### Query balances

`$ chihuahuad query bank total $NODE`

`$ chihuahuad query bank balances $(chihuahuad keys show -a palingram) $NODE`

&nbsp;

### Send funds to other account

`$ chihuahuad tx bank send <sender_account_name> <receiver_address> <amount><denom> $TXFLAG`

&nbsp;

### See the list of code uploaded to the testnet

`$ chihuahuad query wasm list-code $NODE`

&nbsp;

### Generate a new cosm-wasm project from template

`$ cargo install cargo-generate --features vendored-openssl`

`$ cargo generate --git <https://github.com/CosmWasm/cosmwasm-template.git> --name my-first-contract`

&nbsp;

### Store the contract on the blockchain and get the <CODE_ID>

`$ export RES=$(chihuahuad tx wasm store artifacts/<contract_name.wasm> --from <account_name> $TXFLAG -y --output json -b block)`

`$ echo $RES`

`$ export CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')`

`echo $CODE_ID`

&nbsp;

### Get a list of contracts instantiated for <CODE_ID>

`$ chihuahuad query wasm list-contract-by-code $CODE_ID $NODE --output json`

&nbsp;

### Prepare the json message payload

`$ export INIT='{}'`

&nbsp;

### Instantiate the contract

`$ chihuahuad tx wasm instantiate $CODE_ID "$INIT" --from <account_name> --label "BURN TEST CONTRACT" $TXFLAG -y --no-admin`

&nbsp;

### Get the latest contract instantiated for contract with $CODE_ID

`$ export CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')`

`$ echo $CONTRACT`

&nbsp;

### Check the contract details

`$ chihuahuad query wasm contract $CONTRACT $NODE`

&nbsp;

### Check the contract balance

`$ chihuahuad query bank balances $CONTRACT $NODE`

&nbsp;

### query the entire contract state

`$ chihuahuad query wasm contract-state all $CONTRACT $NODE`

&nbsp;

### query the data for a storage key in the contract-state directly

`$ chihuahuad query wasm contract-state raw $CONTRACT 636F6E666967 $NODE  --output "json" | jq -r '.data' | base64 -d`

&nbsp;

### Calling execute methods

`$ export E_PAYLOAD='{"burn_contract_balance":{}}'`

`$ chihuahuad tx wasm execute $CONTRACT "$E_PAYLOAD" --amount 1000000000stake --from palingram $NODE $TXFLAG -y`

&nbsp;

### calling query methods

`$ export Q_PAYLOAD='{"query_list":{}}'`

`$ chihuahuad query wasm contract-state smart $CONTRACT "$Q_PAYLOAD" $NODE --output json`

&nbsp;

### If you prefer to work in a NodeJs environment, run the following command to start the node REPL this is complete with cosmos sdk interactions

`npx @cosmjs/cli@^0.28.1 --init <https://raw.githubusercontent.com/InterWasm/cw-plus-helpers/main/base.ts> --init <https://raw.githubusercontent.com/InterWasm/cw-plus-helpers/main/cw20-base.ts>
<https://docs.cosmwasm.com/docs/1.0/getting-started/interact-with-contract>`

&nbsp;

### [Check here for more info on @cosmjs/cli](https://www.npmjs.com/package/@cosmjs/cli)
