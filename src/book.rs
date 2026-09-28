//! Atomic L2 snapshot replacement and visible-depth execution estimates.

use crate::{
    types::{Coin, Quantity, Side},
    wire::BookSnapshot,
    Decimal, Error,
};

/// A single-coin book containing only the most recently accepted snapshot.
#[derive(Debug, Clone)]
pub struct OrderBook {
    coin: Coin,
    snapshot: Option<BookSnapshot>,
}

impl OrderBook {
    /// Start without a snapshot. Callers must treat it as unavailable until replaced.
    pub fn new(coin: Coin) -> Self {
        Self {
            coin,
            snapshot: None,
        }
    }

    /// Validate first, then replace all levels. Invalid or stale data leaves state intact.
    pub fn replace(&mut self, snapshot: BookSnapshot) -> Result<(), Error> {
        if snapshot.coin != self.coin {
            return Err(Error::Invalid("snapshot coin does not match book".into()));
        }
        snapshot.validate()?;
        if let Some(current) = &self.snapshot {
            if snapshot.time < current.time {
                return Err(Error::Stale {
                    received: snapshot.time,
                    current: current.time,
                });
            }
        }
        self.snapshot = Some(snapshot);
        Ok(())
    }

    /// Forget the current snapshot, for example on a transport disconnection.
    pub fn clear(&mut self) {
        self.snapshot = None;
    }

    /// Last accepted depth. This alone does not establish transport freshness.
    pub fn snapshot(&self) -> Option<&BookSnapshot> {
        self.snapshot.as_ref()
    }

    /// Best ask minus best bid, or unavailable when either side is empty.
    pub fn spread(&self) -> Option<Decimal> {
        let s = self.snapshot.as_ref()?;
        s.levels[1]
            .first()?
            .px
            .value()
            .checked_sub(s.levels[0].first()?.px.value())
    }

    /// Midpoint of the best quotes, with checked decimal arithmetic.
    pub fn mid_price(&self) -> Option<Decimal> {
        let s = self.snapshot.as_ref()?;
        let bid = s.levels[0].first()?.px.value();
        bid.checked_add(self.spread()?.checked_div(Decimal::TWO)?)
    }

    /// Estimate average fill price from visible depth. Buy walks asks; sell walks bids.
    /// Excludes fees, slippage beyond this snapshot, and hidden liquidity.
    pub fn vwap(&self, side: Side, quantity: Quantity) -> Result<Decimal, Error> {
        if quantity.value().is_zero() {
            return Err(Error::Invalid("fill quantity must be positive".into()));
        }
        let snapshot = self.snapshot.as_ref().ok_or(Error::InsufficientLiquidity)?;
        let levels = &snapshot.levels[if side == Side::Buy { 1 } else { 0 }];
        let mut remaining = quantity.value();
        let mut cost = Decimal::ZERO;
        for level in levels {
            let fill = remaining.min(level.sz.value());
            cost = cost
                .checked_add(fill.checked_mul(level.px.value()).ok_or(Error::Overflow)?)
                .ok_or(Error::Overflow)?;
            remaining = remaining.checked_sub(fill).ok_or(Error::Overflow)?;
            if remaining.is_zero() {
                return cost.checked_div(quantity.value()).ok_or(Error::Overflow);
            }
        }
        Err(Error::InsufficientLiquidity)
    }
}
