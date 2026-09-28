//! Hyperliquid tick and lot precision checks; metadata is supplied by the caller.

use crate::{
    types::{Price, Quantity},
    Error,
};

/// The market's price decimal budget.
#[derive(Debug, Clone, Copy)]
pub enum MarketKind {
    /// Perpetual markets have a six-decimal budget.
    Perpetual,
    /// Spot markets have an eight-decimal budget.
    Spot,
}

/// Immutable precision rules built from the asset's `szDecimals` metadata.
#[derive(Debug, Clone, Copy)]
pub struct MarketRules {
    size_decimals: u32,
    price_decimals: u32,
}

impl MarketRules {
    /// Validate the metadata and calculate the price decimal budget.
    pub fn new(kind: MarketKind, size_decimals: u32) -> Result<Self, Error> {
        let budget: u32 = match kind {
            MarketKind::Perpetual => 6,
            MarketKind::Spot => 8,
        };
        let price_decimals = budget
            .checked_sub(size_decimals)
            .ok_or_else(|| Error::Invalid("szDecimals exceeds market decimal budget".into()))?;
        Ok(Self {
            size_decimals,
            price_decimals,
        })
    }

    /// Check an order price without silently rounding it.
    /// Integer prices are exempt from the five significant figure limit.
    pub fn validate_price(self, price: Price) -> Result<(), Error> {
        let value = price.value().normalize();
        if value.scale() > self.price_decimals {
            return Err(Error::Invalid("price has too many decimal places".into()));
        }
        if value.scale() > 0 && value.mantissa().to_string().len() > 5 {
            return Err(Error::Invalid(
                "fractional price exceeds five significant figures".into(),
            ));
        }
        Ok(())
    }

    /// Check a positive order size against the asset's lot precision.
    pub fn validate_size(self, size: Quantity) -> Result<(), Error> {
        if size.value().is_zero() || size.value().normalize().scale() > self.size_decimals {
            return Err(Error::Invalid(
                "size must be positive and respect szDecimals".into(),
            ));
        }
        Ok(())
    }
}
