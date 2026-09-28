# Changelog

## 0.2.0 — 2026-09-28

- Add typed Hyperliquid perpetual and spot metadata and an immutable market catalog.
- Resolve spot precision through base-token indices and preserve exact HIP-3 wire names.
- Exclude delisted perps and reject duplicate markets, missing token references, and invalid precision.
- Expose size and price decimal budgets for callers and diagnostics.
- Add HIP-4 binary outcome metadata, checked coin/token/asset encoding, and catalog integration.
- Reject malformed `#` identifiers and `+` token names at the market-data boundary.
- Keep outcome order precision unavailable when metadata does not specify it.

## 0.1.0 — 2026-09-28

- Start a fresh history for the focused Hyperliquid adaptation.
- Reduce the generic finance surface to validated decimal values and market data.
- Add perp/spot precision checks, wire payloads, atomic L2 snapshots, and VWAP.
- Add synthetic examples, regression tests, compiled README examples, and CI.
