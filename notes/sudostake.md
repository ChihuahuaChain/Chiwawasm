
# Draft: SudoStake: Staking Asset Request for Liquidity

&nbsp;

## Motivation

Blockstream has a mining derivatives market where users can buy the rights to hashing power to mine Bitcoins over a certain period defined in the contract.

When a user buys a [BMN(Blockstream mining note)](https://blockstream.com/finance/bmn/), Bitcoin hashing power (measured in TH/s) gets allocated to the user, and mining rewards streams to an escrow account that releases the funds to the BMN holder after the expiration date defined in the contract.

Meanwhile, the BMN is still tradable in a secondary market for users who need liquidity before the expiration of the mining contract.

&nbsp;

### Benefits to miners

* BMN allows miners to generate liquidity in exchange for hashing power they control, thereby bringing liquidity to the otherwise illiquid but cashflow-rich mining business.

* BMN lowers the barrier to entry into the mining business by allowing investors to buy BMN representing mining hash rates.

&nbsp;

## Renting staking/voting power on the cosmos

We can describe a protocol that allows stakeholders to create vaults holding staked assets. This time, instead of a marketplace for trading hash rates, there is a marketplace for trading staking deals over a specified duration in exchange for liquidity, denominated in any IBC-enabled assets in the allow-list.

&nbsp;

### How it works

* Anyone can mint one of 10,000 unique vaults.

* Vault owners can manage their staking assets using the vault, including staking, un-staking, claiming rewards, voting, and withdrawing funds.

* Vault owners can rent out staking and voting rights to staking assets held in the vault for a specified duration.

* Vault owners can list vaults for sale or transfer vaults to another owner.

&nbsp;

### Benefits to Vault owners

Vault owners can rent out rights to their staking assets at a discounted rate in exchange for upfront liquidity for a specified duration in a peer-to-peer marketplace, thereby unlocking liquidity for their staking assets without giving up control to a third party.

&nbsp;

## Contracts specification

The protocol defines two primary smart-contracts for managing these interactions.

* `VAULTS_CONTRACT`
* `VAULT_CONTRACT`

&nbsp;

### VAULTS_CONTRACT

#### `state: vaults_list`

A list of 10,000 vaults created by the VAULTS_CONTRACT.

#### `fn: Mint`

Creates a new vault by calling the instantiate method of the VAULT_CONTRACT, which returns a contract address, that is then associated with the msg.sender.

#### `fn: Transfer`

Allows a vault owner to transfer ownership to another user.

#### `fn: ListForSale`

A vault owner can list their vault for sale for a fixed price.

&nbsp;

### VAULT_CONTRACT

#### `state: vault_preferences`

A struct that holds the config parameters of the vault.

#### `fn: Delegate`

Allows the vault owner to stake the assets to a validator.

#### `fn: Undelegate`

Allows the vault owner to unstake the assets.

#### `fn: Redelegate`

Allows the vault owner to redelegate their stake to another validator.

#### `fn: WithdrawFunds`

Allows the vault owner to withdraw staking assets held in the vault.

#### `fn: ClaimRewards`

Allows the vault owners (renters included), to call the claim rewards method.

#### `fn: UpdateVaultPreferences`

Allows the vault owner to update vault preferences.

&nbsp;

## Governance

Completely decentralized, only burn 100k HUAHUA to mint a vault.

&nbsp;

## Market Places

Anyone can create frontends/marketplaces that allow users' interaction with vaults.
