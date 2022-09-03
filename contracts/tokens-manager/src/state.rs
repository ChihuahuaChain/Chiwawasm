use cosmwasm_std::{Addr, Coin};
use cw20::Logo;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token_creation_fee: Coin,
    pub token_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TempEntry {
    pub name: String,
    pub symbol: String,
    pub logo: Logo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Entry {
    pub id: u64,
    pub name: String,
    pub symbol: String,
    pub logo: Logo,
    pub contract_addr: Addr,
}

// Here we create a unique sub-index that maps the id to entry.
// This is important because we want to be able to paginate the data
// Learn more about kvstore https://github.com/cosmos/iavl
pub struct EntryIndexes<'a> {
    pub id: UniqueIndex<'a, u64, Entry>,
    // pub name: UniqueIndex<'a, String, Entry>,
}

// This implements the get_indexes trait that returns the list od indexes
impl IndexList<Entry> for EntryIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Entry>> + '_> {
        let v: Vec<&dyn Index<Entry>> = vec![&self.id];
        Box::new(v.into_iter())
    }
}

// Here we create a unique IndexedMap whose default maps from symbol to Entry
pub fn entries<'a>() -> IndexedMap<'a, &'a str, Entry, EntryIndexes<'a>> {
    let indexes = EntryIndexes {
        id: UniqueIndex::new(|e| e.id, "ENTRY_ID"),
        // name: UniqueIndex::new(|e| e.name.clone(), "ENTRY_NAME"),
    };

    IndexedMap::new("ENTRY_LIST", indexes)
}

// This stores the config variables during initialization of the contract
pub const INIT_CONFIG: Item<Config> = Item::new("INIT_CONFIG");

// This stores tbe TempEntry when creating a new token before the contract
// address is gotten
pub const TEMP_ENTRY_STATE: Item<TempEntry> = Item::new("TempEntry");

// This keeps track of the number of items in the ENTRY_LIST
pub const ENTRY_SEQ: Item<u64> = Item::new("ENTRY_SEQ");

// Limits for the custom range query
pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;
