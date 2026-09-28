//! Exact decimal values and snapshot semantics for Hyperliquid market data.
//!
//! The network-free companion to `hyperliquid-stream`.

#![doc = include_str!("../README.md")]

pub mod book;
pub mod market;
pub mod types;
pub mod wire;

pub use rust_decimal::Decimal;

/// Errors raised while validating market values or replacing a book snapshot.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A value or snapshot violates an invariant.
    #[error("invalid market data: {0}")]
    Invalid(String),
    /// A snapshot would move the book backwards in exchange time.
    #[error("stale snapshot: received {received}, current {current}")]
    Stale {
        /// Incoming exchange timestamp in milliseconds.
        received: u64,
        /// Last accepted timestamp in milliseconds.
        current: u64,
    },
    /// Visible depth cannot fill the requested quantity.
    #[error("insufficient visible liquidity")]
    InsufficientLiquidity,
    /// Exact arithmetic exceeded the representable decimal range.
    #[error("decimal arithmetic overflow")]
    Overflow,
}
