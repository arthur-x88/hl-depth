//! Hyperliquid `meta` and `spotMeta` responses and exact subscription identifiers.
//!
//! Token indices are identifiers, not positions in a JSON array. UI ticker aliases
//! are deliberately not guessed: resolve by the coin returned by Hyperliquid.

use crate::{
    market::{MarketKind, MarketRules},
    outcome::{OutcomeId, OutcomeMeta},
    types::Coin,
    Error,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

/// The perpetual market universe returned by `{"type":"meta"}`.
#[derive(Debug, Clone, Deserialize)]
pub struct PerpMeta {
    /// Assets, including optional delisted entries and DEX-prefixed HIP-3 names.
    pub universe: Vec<PerpAsset>,
}

/// Precision and availability fields for a perpetual market.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpAsset {
    /// Exact subscription coin, e.g. `BTC` or `xyz:XYZ100`.
    pub name: Coin,
    /// Maximum size decimal places.
    pub sz_decimals: u32,
    /// Delisted entries remain in metadata but must not be selected for a new feed.
    #[serde(default)]
    pub is_delisted: bool,
}

/// The spot pair and token universes returned by `{"type":"spotMeta"}`.
#[derive(Debug, Clone, Deserialize)]
pub struct SpotMeta {
    /// Traded spot pairs.
    pub universe: Vec<SpotPair>,
    /// Tokens referenced by each pair's `tokens` indices.
    pub tokens: Vec<SpotToken>,
}

/// A base or quote token in Hyperliquid's spot metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotToken {
    /// HyperCore token name, which may differ from a frontend alias.
    pub name: String,
    /// Token identifier, independent of the order of entries in the response.
    pub index: u32,
    /// Size precision, used when this token is the pair's base asset.
    pub sz_decimals: u32,
}

/// A spot market from the `universe` array.
#[derive(Debug, Clone, Deserialize)]
pub struct SpotPair {
    /// Wire coin: `PURR/USDC` for pair zero, otherwise `@{index}`.
    pub name: Coin,
    /// Pair index, not a token index.
    pub index: u32,
    /// Base and quote token indices, in that order.
    pub tokens: [u32; 2],
}

/// A resolved market with immutable, validated precision metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    coin: Coin,
    label: String,
    kind: MarketKind,
    rules: Option<MarketRules>,
    outcome: Option<OutcomeId>,
    description: Option<String>,
}

impl Instrument {
    /// Exact identifier accepted by Hyperliquid subscriptions.
    pub fn coin(&self) -> &Coin {
        &self.coin
    }
    /// Human-readable token pair for spot, or the full name for perpetuals.
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Spot, perpetual, or HIP-4 outcome market.
    pub fn kind(&self) -> MarketKind {
        self.kind
    }
    /// Price and size rules derived from metadata. Outcome metadata does not supply
    /// precision, so HIP-4 returns `None` rather than inventing a lot size.
    pub fn rules(&self) -> Option<MarketRules> {
        self.rules
    }

    /// The checked HIP-4 outcome/side identity, absent for spot and perpetuals.
    pub fn outcome_id(&self) -> Option<OutcomeId> {
        self.outcome
    }

    /// Outcome resolution text when available; not interpreted by this library.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// A deterministic catalog of perpetuals, spot pairs, and optional HIP-4 outcomes.
/// Refresh it when metadata changes; no network or global cache is hidden here.
#[derive(Debug, Clone)]
pub struct MarketCatalog {
    instruments: BTreeMap<String, Instrument>,
}

impl MarketCatalog {
    /// Validate references, duplicate identifiers, and precision before constructing a catalog.
    /// HIP-3 metadata is accepted with its existing DEX prefix intact.
    pub fn new(perps: &PerpMeta, spot: &SpotMeta) -> Result<Self, Error> {
        let mut instruments = BTreeMap::new();
        let mut perp_names = std::collections::HashSet::new();
        for asset in &perps.universe {
            if !perp_names.insert(asset.name.as_str()) {
                return Err(Error::Invalid(format!(
                    "duplicate perpetual coin {}",
                    asset.name
                )));
            }
            if asset.is_delisted {
                continue;
            }
            let instrument = Instrument {
                coin: asset.name.clone(),
                label: asset.name.to_string(),
                kind: MarketKind::Perpetual,
                rules: Some(MarketRules::new(MarketKind::Perpetual, asset.sz_decimals)?),
                outcome: None,
                description: None,
            };
            instruments.insert(asset.name.to_string(), instrument);
        }
        let mut tokens = HashMap::new();
        for token in &spot.tokens {
            if tokens.insert(token.index, token).is_some() {
                return Err(Error::Invalid(format!(
                    "duplicate spot token index {}",
                    token.index
                )));
            }
        }
        for pair in &spot.universe {
            let expected = if pair.index == 0 {
                "PURR/USDC".into()
            } else {
                format!("@{}", pair.index)
            };
            if pair.name.as_str() != expected {
                return Err(Error::Invalid(format!(
                    "spot pair {} must use coin {expected}",
                    pair.index
                )));
            }
            if pair.tokens[0] == pair.tokens[1] {
                return Err(Error::Invalid("spot base and quote must differ".into()));
            }
            let base = tokens.get(&pair.tokens[0]).ok_or_else(|| {
                Error::Invalid(format!("missing spot base token {}", pair.tokens[0]))
            })?;
            let quote = tokens.get(&pair.tokens[1]).ok_or_else(|| {
                Error::Invalid(format!("missing spot quote token {}", pair.tokens[1]))
            })?;
            let instrument = Instrument {
                coin: pair.name.clone(),
                label: format!("{}/{}", base.name, quote.name),
                kind: MarketKind::Spot,
                rules: Some(MarketRules::new(MarketKind::Spot, base.sz_decimals)?),
                outcome: None,
                description: None,
            };
            if instruments
                .insert(pair.name.to_string(), instrument)
                .is_some()
            {
                return Err(Error::Invalid(format!(
                    "duplicate market coin {}",
                    pair.name
                )));
            }
        }
        Ok(Self { instruments })
    }

    /// Resolve an exact coin; no ambiguous frontend-symbol conversion is performed.
    pub fn resolve(&self, coin: &Coin) -> Option<&Instrument> {
        self.instruments.get(coin.as_str())
    }

    /// Build an outcome-only catalog without fetching unrelated spot or perp metadata.
    pub fn from_outcomes(outcomes: &OutcomeMeta) -> Result<Self, Error> {
        Self {
            instruments: BTreeMap::new(),
        }
        .with_outcomes(outcomes)
    }

    /// Extend a catalog with both sides of each HIP-4 outcome. Duplicate or
    /// overflowing identifiers are errors. Order precision remains unavailable.
    pub fn with_outcomes(mut self, outcomes: &OutcomeMeta) -> Result<Self, Error> {
        for spec in &outcomes.outcomes {
            for (side, label) in spec.side_specs.iter().enumerate() {
                let id = OutcomeId::new(spec.outcome, side as u8)?;
                let coin = id.coin()?;
                let instrument = Instrument {
                    coin: coin.clone(),
                    label: format!("{} / {}", spec.name, label.name),
                    kind: MarketKind::Outcome,
                    rules: None,
                    outcome: Some(id),
                    description: Some(spec.description.clone()),
                };
                if self
                    .instruments
                    .insert(coin.to_string(), instrument)
                    .is_some()
                {
                    return Err(Error::Invalid(format!("duplicate outcome coin {coin}")));
                }
            }
        }
        Ok(self)
    }

    /// Iterate in a stable order by wire coin.
    pub fn instruments(&self) -> impl ExactSizeIterator<Item = &Instrument> {
        self.instruments.values()
    }
}
