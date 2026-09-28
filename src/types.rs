//! Validated decimal wrappers, adapted from the original generic value types.
//!
//! Deserialization uses the same validation as direct construction.

use crate::{Decimal, Error};
use serde::{Deserialize, Serialize};

/// A Hyperliquid market coin, preserving `@`, DEX prefixes, and checked HIP-4 `#` encodings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Coin(String);

impl Coin {
    /// Reject empty/whitespace/control characters, malformed HIP-4 encodings,
    /// and `+` outcome token names (which are not subscription coins).
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(Error::Invalid(
                "coin must be nonempty without whitespace or control characters".into(),
            ));
        }
        if value.starts_with('+') {
            return Err(Error::Invalid(
                "HIP-4 token names use +; market subscription coins use #".into(),
            ));
        }
        let coin = Self(value);
        if coin.as_str().starts_with('#') {
            crate::outcome::OutcomeId::try_from(&coin)?;
        }
        Ok(coin)
    }

    /// The unmodified exchange identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Coin {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Error> {
        Self::new(value)
    }
}

impl From<Coin> for String {
    fn from(value: Coin) -> Self {
        value.0
    }
}

impl std::fmt::Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A strictly positive exact decimal price, encoded as a JSON string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Price(Decimal);

impl Price {
    /// Reject zero and negative prices.
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value <= Decimal::ZERO {
            return Err(Error::Invalid("price must be positive".into()));
        }
        Ok(Self(value))
    }

    /// The exact underlying decimal.
    pub fn value(self) -> Decimal {
        self.0
    }
}

impl TryFrom<String> for Price {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Error> {
        Self::new(Decimal::from_str_exact(&value).map_err(|e| Error::Invalid(e.to_string()))?)
    }
}

impl From<Price> for String {
    fn from(value: Price) -> Self {
        value.0.normalize().to_string()
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A non-negative exact decimal quantity, encoded as a JSON string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Quantity(Decimal);

impl Quantity {
    /// Reject negative quantities. Orders and trade prints additionally require nonzero size.
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value < Decimal::ZERO {
            return Err(Error::Invalid("quantity cannot be negative".into()));
        }
        Ok(Self(value))
    }

    /// The exact underlying decimal.
    pub fn value(self) -> Decimal {
        self.0
    }
}

impl TryFrom<String> for Quantity {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Error> {
        Self::new(Decimal::from_str_exact(&value).map_err(|e| Error::Invalid(e.to_string()))?)
    }
}

impl From<Quantity> for String {
    fn from(value: Quantity) -> Self {
        value.0.normalize().to_string()
    }
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The side codes used in Hyperliquid trade messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// Buy / bid (`B`).
    #[serde(rename = "B")]
    Buy,
    /// Sell / ask (`A`).
    #[serde(rename = "A")]
    Sell,
}
