//! Public market-data payloads. Unknown additional exchange fields are tolerated.

use crate::{
    types::{Coin, Price, Quantity, Side},
    Error,
};
use serde::{Deserialize, Serialize};

/// A trade print. Validate before feeding an aggregation pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    /// Exchange coin identifier.
    pub coin: Coin,
    /// Exchange side code.
    pub side: Side,
    /// Exact execution price.
    pub px: Price,
    /// Exact base asset quantity.
    pub sz: Quantity,
    /// Exchange timestamp in milliseconds.
    pub time: u64,
    /// Trade identifier; unique only together with time and coin.
    pub tid: u64,
}

impl Trade {
    /// Reject a zero-size trade print.
    pub fn validate(&self) -> Result<(), Error> {
        if self.sz.value().is_zero() {
            return Err(Error::Invalid("trade size must be positive".into()));
        }
        Ok(())
    }

    /// The documented compound identifier `(time, coin, tid)`.
    pub fn key(&self) -> (u64, &str, u64) {
        (self.time, self.coin.as_str(), self.tid)
    }
}

/// One aggregated price level from the L2 feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    /// Level price.
    pub px: Price,
    /// Total resting size.
    pub sz: Quantity,
    /// Number of resting orders at this price.
    pub n: u64,
}

/// A complete snapshot of the depth published by the exchange, not a delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookSnapshot {
    /// Exchange coin identifier.
    pub coin: Coin,
    /// Exchange timestamp in milliseconds.
    pub time: u64,
    /// Bids in descending price order, then asks in ascending price order.
    pub levels: [Vec<Level>; 2],
}

impl BookSnapshot {
    /// Check positive levels, strict price ordering, and an uncrossed spread.
    pub fn validate(&self) -> Result<(), Error> {
        for (side, levels) in self.levels.iter().enumerate() {
            if levels.iter().any(|l| l.sz.value().is_zero() || l.n == 0) {
                return Err(Error::Invalid(
                    "book levels require positive size and order count".into(),
                ));
            }
            if levels.windows(2).any(|p| {
                if side == 0 {
                    p[0].px <= p[1].px
                } else {
                    p[0].px >= p[1].px
                }
            }) {
                return Err(Error::Invalid(
                    "book levels are duplicated or incorrectly ordered".into(),
                ));
            }
        }
        if let (Some(bid), Some(ask)) = (self.levels[0].first(), self.levels[1].first()) {
            if bid.px >= ask.px {
                return Err(Error::Invalid("book is crossed or locked".into()));
            }
        }
        Ok(())
    }
}
