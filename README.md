# cw721-countable
CW721 Implementation with Token IDs that increase consecutively from 1

## 🚀 Development

```bash
rustup default stable
cargo version
# If this is lower than 1.55.0+, update
rustup update stable

rustup target list --installed
rustup target add wasm32-unknown-unknown

# Test
cargo test

# Generate Schemas
cargo schema

# Optimized Compilation (ARM64)
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.6
```

## Contract State
```rust
pub struct State {
  // Next `token_id` to be minted
  pub token_count: i32,
}
```

## Messages

### Instantiate contract

```json
{
  "instantiate": {
    "name": "Tickets",
    "symbol": "SURE",
    "minter": "minter address"
  }
}
```

### Mint single token to a specific address
```jsonc
{
  "mint": {
    // The desired owner
    "owner": "owner address",

    // URL to JSON Metadata (FIXME: Should be auto-generated with `token_id` by default)
    "token_uri": "https://arweave.net/...",

    // Onchain metadata (Optional; FIXME: Should be removed)
    "extension": {
      // ...
    },
  },
}
```

### Burn token held by `sender`
```jsonc
{
  "burn": {
    "token_id": "1"
  }
}
```
