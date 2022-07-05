# Details

This contract allows the admin to burn contract balance as well as transfer ownership to another admin.

## Messages

```rust
pub enum ExecuteMsg {
    BurnContractBalance {},
    TransferContractOwnership { new_owner: String },
}
```

### Queries

```rust
enum QueryMsg {
    QueryBalance {},
}

struct BalanceResponse {
    pub balance: String,
}
```

## Running this contract

You will need Rust 1.44.1+ with `wasm32-unknown-unknown` target installed.

You can run unit tests on this via:

`$ cargo unit-test`

Once you are happy with the content, you can compile it to wasm via:

`$ cargo run-script optimize`