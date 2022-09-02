use cosmwasm_std::{Addr, Coin};
use cw20::Logo;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub token_creation_fee: Coin,
}

pub struct TempEntry {
    pub token_name: String,
    pub symbol: String,
    pub logo: Logo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Entry {
    pub id: u64,
    pub token_name: String,
    pub symbol: String,
    pub logo: Logo,
    pub contract_addr: Addr,
}

// Here we create a unique sub-index that maps the id to entry
pub struct EntryIndexes<'a> {
    pub key: UniqueIndex<'a, u64, Entry>,
}

impl IndexList<Entry> for EntryIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Entry>> + '_> {
        let v: Vec<&dyn Index<Entry>> = vec![&self.key];
        Box::new(v.into_iter())
    }
}

// Here we create a unique IndexedMap whose default maps from (token_name, symbol) to Entry
pub fn entries<'a>() -> IndexedMap<'a, (&'a str, &'a str), Entry, EntryIndexes<'a>> {
    let indexes = EntryIndexes {
        key: UniqueIndex::new(|e| e.id, "TOKEN_NAME_SYMBOL"),
    };

    IndexedMap::new("TOKENS_LIST", indexes)
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");

// Limits for the custom range query
pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;
