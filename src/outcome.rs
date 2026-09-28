//! HIP-4 outcome identity and metadata, separate from spot pair and token indices.

use crate::{types::Coin, Error};
use serde::Deserialize;

/// A checked binary outcome side. Only sides zero and one are valid.
/// Its encoding is `10 * outcome + side`; coin, token, and asset IDs are distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutcomeId {
    outcome: u32,
    side: u8,
}

impl OutcomeId {
    /// Validate the binary side and all derived IDs without overflowing.
    pub fn new(outcome: u32, side: u8) -> Result<Self, Error> {
        if side > 1 {
            return Err(Error::Invalid("outcome side must be 0 or 1".into()));
        }
        outcome
            .checked_mul(10)
            .and_then(|v| v.checked_add(u32::from(side)))
            .and_then(|v| v.checked_add(100_000_000))
            .ok_or(Error::Overflow)?;
        Ok(Self { outcome, side })
    }

    /// Outcome identifier from `outcomeMeta`.
    pub fn outcome(self) -> u32 {
        self.outcome
    }
    /// Binary side index into `sideSpecs`; labels are not assumed to be Yes/No.
    pub fn side(self) -> u8 {
        self.side
    }
    /// Shared numeric encoding; construction guarantees it fits in the asset ID.
    pub fn encoding(self) -> u32 {
        self.outcome * 10 + u32::from(self.side)
    }
    /// Subscription coin, prefixed with `#`.
    pub fn coin(self) -> Result<Coin, Error> {
        Coin::new(format!("#{}", self.encoding()))
    }
    /// Corresponding outcome token name, prefixed with `+`, not a subscription coin.
    pub fn token_name(self) -> String {
        format!("+{}", self.encoding())
    }
    /// Encoded Hyperliquid asset ID. This library does not submit exchange actions.
    pub fn asset_id(self) -> u32 {
        100_000_000 + self.encoding()
    }
}

impl TryFrom<&Coin> for OutcomeId {
    type Error = Error;
    fn try_from(coin: &Coin) -> Result<Self, Error> {
        let digits = coin
            .as_str()
            .strip_prefix('#')
            .ok_or_else(|| Error::Invalid("outcome subscription coins start with #".into()))?;
        if digits.is_empty()
            || !digits.bytes().all(|b| b.is_ascii_digit())
            || (digits.len() > 1 && digits.starts_with('0'))
        {
            return Err(Error::Invalid(
                "outcome coin must use a canonical decimal encoding".into(),
            ));
        }
        let encoding: u32 = digits.parse().map_err(|_| Error::Overflow)?;
        Self::new(encoding / 10, (encoding % 10) as u8)
    }
}

/// Current outcome specifications returned by the `outcomeMeta` info request.
#[derive(Debug, Clone, Deserialize)]
pub struct OutcomeMeta {
    /// Binary outcome definitions. Extra question/deployer metadata is tolerated.
    pub outcomes: Vec<OutcomeSpec>,
}

/// One outcome, with two independently named token sides.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeSpec {
    /// Outcome identifier used to derive both subscription coins.
    pub outcome: u32,
    /// Protocol or deployer label; may be shared by many outcome instances.
    pub name: String,
    /// Resolution specification, preserved verbatim rather than interpreted as a trading rule.
    pub description: String,
    /// Exactly two side labels, in encoding order.
    pub side_specs: [OutcomeSide; 2],
    /// Quote token name when supplied by the API.
    #[serde(default)]
    pub quote_token: Option<String>,
}

/// Display information for a single binary side.
#[derive(Debug, Clone, Deserialize)]
pub struct OutcomeSide {
    /// Metadata-supplied label, such as Yes/No or Change/No Change.
    pub name: String,
}
