# Validation — 0.2.0 — 2026-09-28

Local environment: Rust 1.90.0, Linux under WSL2.

| Check | Result |
| --- | --- |
| `cargo test --locked` | 16 regression tests and 1 README doctest passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| `cargo fmt --check` | Passed |
| `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | Passed |
| `cargo run --locked --example depth` | Completed; output reproduced in README |

The companion stream was also exercised against the public mainnet feed: its
decoder accepted live BTC prints and L2 snapshots using these types. This is a
bounded integration check, not an endurance or execution-quality benchmark.

Metadata tests cover base-token index joins, spot pair identifiers, HIP-3 prefixes,
delisted perpetuals, duplicate metadata, and invalid precision. HIP-4 tests cover
binary side encoding, distinct coin/token/asset IDs, overflow, metadata labels,
and unavailable order precision. The companion decoded mainnet HIP-4 books and
recent trade messages for `#12090` using these types.

Linux and Windows automation results are available in the repository's Actions
tab. No performance or profitability claim is made by these checks.
