# hl-depth

[![CI](https://github.com/arthur-x88/hl-depth/actions/workflows/ci.yml/badge.svg)](https://github.com/arthur-x88/hl-depth/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/arthur-x88/hl-depth/blob/main/LICENSE)

Rust types, market metadata, and level-2 (L2) order books for Hyperliquid. Validate decimal
prices and sizes, resolve market identifiers, and calculate fills against visible
depth. Supports spot, perpetuals, HIP-3 assets, and HIP-4 outcome identifiers.

The library runs without network access. Its companion,
[hl-flow](https://github.com/arthur-x88/hl-flow), handles metadata requests, live
feeds, and replay.

## Quick start

Requires Rust 1.90 or newer.

```bash
git clone https://github.com/arthur-x88/hl-depth.git
cd hl-depth
cargo run --locked --example depth
```

The example builds a synthetic BTC book and checks an order against its visible
liquidity and price precision:

```text
Synthetic BTC snapshot (not live data)
Spread: 1 | Mid: 65000.50
Buy 0.75 BTC visible-depth VWAP: 65001.466666666666666666666667
Fractional price 65001.1 accepted: false
```

To use the library, add the Git dependency to your `Cargo.toml`. The crate is
currently distributed through Git.

```toml
[dependencies]
hl-depth = { git = "https://github.com/arthur-x88/hl-depth" }
```

## API

| Module | What it handles |
| --- | --- |
| `types` | Decimal `Price` and `Quantity`, `Coin` identifier syntax, and Hyperliquid's `B`/`A` trade sides. JSON decoding applies the same value checks as construction. |
| `market` | Price and size precision: significant figures, the integer-price exception, and spot/perp decimal budgets. |
| `metadata` | `meta` and `spotMeta` responses, token-index joins, market lookup, and exclusion of delisted perps. |
| `outcome` | HIP-4 binary sides and the distinct coin, token, and asset encodings. |
| `wire` | Trade prints and L2 snapshots with decimal strings and millisecond timestamps. |
| `book` | Snapshot replacement, spread, midpoint, and visible-depth VWAP. |

For example, validate an order using a perpetual market with `szDecimals = 5`:

```rust
use hl_depth::{
    market::{MarketKind, MarketRules},
    types::{Price, Quantity},
    Decimal,
};

fn main() -> Result<(), hl_depth::Error> {
    let rules = MarketRules::new(MarketKind::Perpetual, 5)?;
    let price = Price::new(Decimal::from(65_001))?;
    let size = Quantity::new(Decimal::new(75, 2))?;
    rules.validate_price(price)?;
    rules.validate_size(size)?;
    Ok(())
}
```

For live applications, resolve the instrument through `MarketCatalog` and use its
metadata-derived rules. Spot size precision comes from the base token. Token
indices, pair indices, and positions in a JSON array are separate values.

## Book behavior

Each `l2Book` message replaces the published depth. Levels absent from the new
snapshot disappear. Stale, crossed, duplicated, misordered, or wrong-coin
snapshots return an error and leave the previous book intact.

Empty sides are valid; spread and midpoint are then unavailable. VWAP walks the
visible levels and returns an error if they cannot fill the requested quantity
or the notional calculation overflows. It excludes fees and later price changes.
Decimal division uses the precision available in `rust_decimal`.

An exchange timestamp cannot establish connection health. Clear cached books
when the feed disconnects, and refresh market metadata when listings change.

## HIP-4 outcomes

`MarketCatalog::from_outcomes` builds an outcome catalog; `with_outcomes` adds
outcomes to an existing catalog. Labels and resolution text come from
`outcomeMeta`.

For outcome `1`, side `0`, `OutcomeId` produces three distinct identifiers:

| Representation | Value |
| --- | --- |
| Market subscription coin | `#10` |
| Token name | `+10` |
| Asset ID | `100000010` |

Only sides `0` and `1` are accepted. Coin validation rejects malformed outcome
encodings and `+` token names used as market coins.

`Instrument::rules()` returns `None` for HIP-4 because `outcomeMeta` does not
provide size precision. Outcome market data is supported; order-size and tick
validation remain unavailable. A metadata listing does not establish liquidity.

## Checks

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
```

CI runs on Linux and Windows. Tests cover value validation, snapshot rollback,
VWAP, metadata joins, and outcome identities. The Rust example above is a
doctest. See [validation results](https://github.com/arthur-x88/hl-depth/blob/main/VALIDATION.md)
for recorded checks and live integration results.

## Hyperliquid references

- [Price and size precision](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size)
- [WebSocket payloads](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [Perpetual metadata](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Spot and outcome metadata](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [Asset and outcome identifiers](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/asset-ids)

## License

Released under the [MIT License](https://github.com/arthur-x88/hl-depth/blob/main/LICENSE).
The license file contains the terms and retained copyright notices, including
those for adapted code. This project is not affiliated with Hyperliquid.
