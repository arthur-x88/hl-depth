//! Hyperliquid metadata resolution and validation regressions.
use hl_depth::{
    market::MarketKind,
    metadata::{MarketCatalog, PerpMeta, SpotMeta},
    types::Coin,
};
use serde_json::json;

fn inputs() -> (PerpMeta, SpotMeta) {
    let perps = serde_json::from_value(json!({"universe":[
        {"name":"BTC","szDecimals":5,"maxLeverage":40},
        {"name":"xyz:XYZ100","szDecimals":4},
        {"name":"OLD","szDecimals":1,"isDelisted":true}
    ]}))
    .unwrap();
    let spot = serde_json::from_value(json!({"universe":[
        {"name":"PURR/USDC","index":0,"tokens":[1,0]},
        {"name":"@107","index":107,"tokens":[150,0]}
    ],"tokens":[
        {"name":"HYPE","index":150,"szDecimals":2},
        {"name":"USDC","index":0,"szDecimals":8},
        {"name":"PURR","index":1,"szDecimals":0}
    ]}))
    .unwrap();
    (perps, spot)
}

#[test]
fn spot_precision_uses_base_token_index_not_array_position_or_pair_index() {
    let (perps, spot) = inputs();
    let catalog = MarketCatalog::new(&perps, &spot).unwrap();
    let hype = catalog.resolve(&Coin::new("@107").unwrap()).unwrap();
    assert_eq!(hype.label(), "HYPE/USDC");
    assert_eq!(hype.kind(), MarketKind::Spot);
    assert_eq!(hype.rules().unwrap().size_decimals(), 2);
    assert_eq!(hype.rules().unwrap().price_decimals(), 6);
    assert_eq!(catalog.instruments().len(), 4);
    assert!(catalog.resolve(&Coin::new("HYPE/USDC").unwrap()).is_none());
}

#[test]
fn perp_and_hip3_names_are_preserved_and_delisted_markets_excluded() {
    let (perps, spot) = inputs();
    let catalog = MarketCatalog::new(&perps, &spot).unwrap();
    let btc = catalog.resolve(&Coin::new("BTC").unwrap()).unwrap();
    assert_eq!(btc.rules().unwrap().size_decimals(), 5);
    assert_eq!(btc.rules().unwrap().price_decimals(), 1);
    assert_eq!(btc.kind(), MarketKind::Perpetual);
    assert!(catalog.resolve(&Coin::new("xyz:XYZ100").unwrap()).is_some());
    assert!(catalog.resolve(&Coin::new("XYZ100").unwrap()).is_none());
    assert!(catalog.resolve(&Coin::new("OLD").unwrap()).is_none());
    assert_eq!(
        catalog
            .resolve(&Coin::new("PURR/USDC").unwrap())
            .unwrap()
            .rules()
            .unwrap()
            .size_decimals(),
        0
    );
}

#[test]
fn missing_tokens_duplicate_indices_and_wrong_wire_coins_are_rejected() {
    let (perps, original) = inputs();
    let mut spot = original.clone();
    spot.tokens.remove(0);
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let mut spot = original.clone();
    spot.tokens.push(spot.tokens[0].clone());
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let mut spot = original.clone();
    spot.universe.push(spot.universe[0].clone());
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let mut spot = original.clone();
    spot.universe[1].name = Coin::new("@150").unwrap();
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let mut spot = original.clone();
    spot.universe[0].tokens = [0, 0];
    assert!(MarketCatalog::new(&perps, &spot).is_err());
}

#[test]
fn duplicate_perps_and_unusable_size_precision_are_rejected() {
    let (original, spot) = inputs();
    let mut perps = original.clone();
    perps.universe.push(perps.universe[0].clone());
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let mut perps = original;
    perps.universe[0].sz_decimals = 7;
    assert!(MarketCatalog::new(&perps, &spot).is_err());
    let (perps, mut spot) = inputs();
    spot.tokens[0].sz_decimals = 9;
    assert!(MarketCatalog::new(&perps, &spot).is_err());
}
