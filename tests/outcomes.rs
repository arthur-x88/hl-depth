//! HIP-4 identity and catalog regressions based on the documented wire encoding.
use hl_depth::{
    market::{MarketKind, MarketRules},
    metadata::MarketCatalog,
    outcome::{OutcomeId, OutcomeMeta},
    types::Coin,
};

#[test]
fn outcome_coin_token_and_asset_encodings_are_not_interchangeable() {
    let first = OutcomeId::new(1, 0).unwrap();
    assert_eq!(first.coin().unwrap().as_str(), "#10");
    assert_eq!(first.token_name(), "+10");
    assert_eq!(first.asset_id(), 100_000_010);
    let second = OutcomeId::new(1, 1).unwrap();
    assert_eq!(second.coin().unwrap().as_str(), "#11");
    assert_eq!(second.asset_id(), 100_000_011);
    assert_eq!(
        OutcomeId::try_from(&second.coin().unwrap()).unwrap(),
        second
    );
    assert_eq!((second.outcome(), second.side()), (1, 1));
}

#[test]
fn invalid_sides_noncanonical_coins_and_id_overflow_are_rejected() {
    for coin in [
        "+10",
        "@10",
        "#12",
        "#19",
        "#010",
        "#-10",
        "#",
        "#4294967295",
        "#999999999999",
    ] {
        assert!(
            Coin::new(coin)
                .and_then(|coin| OutcomeId::try_from(&coin))
                .is_err(),
            "{coin}"
        );
    }
    assert!(OutcomeId::new(1, 2).is_err());
    assert!(OutcomeId::new(u32::MAX, 0).is_err());
    assert!(OutcomeId::new(429_496_729, 0).is_err()); // encoding fits; asset offset does not
}

#[test]
fn metadata_keeps_side_labels_and_resolution_text_without_inventing_precision() {
    let meta: OutcomeMeta = serde_json::from_str(r#"{"outcomes":[{"outcome":123,"name":"Rate decision","description":"Synthetic resolution text","sideSpecs":[{"name":"Change"},{"name":"No Change"}],"quoteToken":"USDC"}],"questions":[]}"#).unwrap();
    let catalog = MarketCatalog::from_outcomes(&meta).unwrap();
    assert_eq!(catalog.instruments().len(), 2);
    let market = catalog.resolve(&Coin::new("#1231").unwrap()).unwrap();
    assert_eq!(market.kind(), MarketKind::Outcome);
    assert_eq!(market.label(), "Rate decision / No Change");
    assert_eq!(market.description(), Some("Synthetic resolution text"));
    assert_eq!(
        market.outcome_id().unwrap(),
        OutcomeId::new(123, 1).unwrap()
    );
    assert!(market.rules().is_none());
    assert!(MarketRules::new(MarketKind::Outcome, 2).is_err());
    assert!(Coin::new("+1231").is_err());
    assert!(catalog.with_outcomes(&meta).is_err());
}

#[test]
fn nonbinary_metadata_is_rejected_instead_of_silently_truncated() {
    let input = r#"{"outcomes":[{"outcome":1,"name":"Bad","description":"","sideSpecs":[{"name":"Only one"}]}]}"#;
    assert!(serde_json::from_str::<OutcomeMeta>(input).is_err());
}
