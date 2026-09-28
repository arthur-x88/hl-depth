//! Executable examples and regression checks for the public market-data API.
use hl_depth::{
    book::OrderBook,
    market::{MarketKind, MarketRules},
    types::{Coin, Price, Quantity, Side},
    wire::BookSnapshot,
    Decimal, Error,
};
use rust_decimal_macros::dec;
use serde_json::json;

fn snapshot(time: u64) -> BookSnapshot {
    serde_json::from_value(json!({"coin":"BTC","time":time,"levels":[
        [{"px":"99","sz":"2","n":1},{"px":"98","sz":"3","n":2}],
        [{"px":"101","sz":"1","n":1},{"px":"102","sz":"2","n":2}]
    ]}))
    .unwrap()
}

#[test]
fn deserialization_cannot_bypass_value_invariants() {
    for value in ["0", "-1", "NaN", "1.00000000000000000000000000001"] {
        assert!(
            serde_json::from_value::<Price>(json!(value)).is_err(),
            "{value}"
        );
    }
    assert!(serde_json::from_value::<Price>(json!(1.25)).is_err());
    assert!(serde_json::from_value::<Quantity>(json!("-0.1")).is_err());
    for value in ["", "BT C", "BTC\n", "BTC\u{0000}"] {
        assert!(serde_json::from_value::<Coin>(json!(value)).is_err());
    }
}

#[test]
fn identifiers_and_decimal_strings_round_trip() {
    for value in ["BTC", "@107", "xyz:XYZ100"] {
        let coin = Coin::new(value).unwrap();
        assert_eq!(serde_json::to_value(coin).unwrap(), json!(value));
    }
    let price = Price::new(dec!(0.00012340)).unwrap();
    assert_eq!(serde_json::to_value(price).unwrap(), json!("0.0001234"));
    assert_eq!(
        serde_json::from_value::<Price>(json!("0.0001234")).unwrap(),
        price
    );
    assert!(serde_json::from_value::<Side>(json!("buy")).is_err());
}

#[test]
fn published_precision_examples_and_integer_exception() {
    let perps = MarketRules::new(MarketKind::Perpetual, 0).unwrap();
    for value in [dec!(1234.5), dec!(0.001234), dec!(123456), dec!(123456.000)] {
        perps.validate_price(Price::new(value).unwrap()).unwrap();
    }
    for value in [dec!(1234.56), dec!(0.0012345)] {
        assert!(perps.validate_price(Price::new(value).unwrap()).is_err());
    }
    let perps = MarketRules::new(MarketKind::Perpetual, 1).unwrap();
    assert!(perps
        .validate_price(Price::new(dec!(0.01234)).unwrap())
        .is_ok());
    assert!(perps
        .validate_price(Price::new(dec!(0.012345)).unwrap())
        .is_err());
    for decimals in [0, 1] {
        MarketRules::new(MarketKind::Spot, decimals)
            .unwrap()
            .validate_price(Price::new(dec!(0.0001234)).unwrap())
            .unwrap();
    }
    assert!(MarketRules::new(MarketKind::Spot, 2)
        .unwrap()
        .validate_price(Price::new(dec!(0.0001234)).unwrap())
        .is_err());
}

#[test]
fn size_precision_and_metadata_are_validated() {
    let rules = MarketRules::new(MarketKind::Perpetual, 3).unwrap();
    assert!(rules
        .validate_size(Quantity::new(dec!(1.0010)).unwrap())
        .is_ok());
    assert!(rules
        .validate_size(Quantity::new(dec!(1.0001)).unwrap())
        .is_err());
    assert!(rules
        .validate_size(Quantity::new(dec!(0)).unwrap())
        .is_err());
    assert!(MarketRules::new(MarketKind::Perpetual, 7).is_err());
    assert!(MarketRules::new(MarketKind::Spot, 9).is_err());
}

#[test]
fn snapshots_replace_depth_instead_of_merging() {
    let mut book = OrderBook::new(Coin::new("BTC").unwrap());
    book.replace(snapshot(10)).unwrap();
    assert_eq!(book.spread(), Some(dec!(2)));
    assert_eq!(book.mid_price(), Some(dec!(100)));
    let mut next = snapshot(11);
    next.levels[0].truncate(1);
    next.levels[1].clear();
    book.replace(next).unwrap();
    assert_eq!(book.snapshot().unwrap().levels[0].len(), 1);
    assert_eq!(book.spread(), None);
    assert_eq!(book.mid_price(), None);
    book.clear();
    assert!(book.snapshot().is_none());
}

#[test]
fn invalid_and_stale_snapshots_leave_book_unchanged() {
    let mut book = OrderBook::new(Coin::new("BTC").unwrap());
    let original = snapshot(10);
    book.replace(original.clone()).unwrap();
    let mut invalid = vec![snapshot(9)];
    let mut other = snapshot(11);
    other.coin = Coin::new("ETH").unwrap();
    invalid.push(other);
    let mut crossed = snapshot(11);
    crossed.levels[0][0].px = Price::new(dec!(101)).unwrap();
    invalid.push(crossed);
    let mut reversed = snapshot(11);
    reversed.levels[0].reverse();
    invalid.push(reversed);
    let mut duplicate = snapshot(11);
    duplicate.levels[1][1].px = duplicate.levels[1][0].px;
    invalid.push(duplicate);
    let mut zero = snapshot(11);
    zero.levels[0][0].sz = Quantity::new(dec!(0)).unwrap();
    invalid.push(zero);
    let mut no_orders = snapshot(11);
    no_orders.levels[0][0].n = 0;
    invalid.push(no_orders);
    for incoming in invalid {
        assert!(book.replace(incoming).is_err());
        assert_eq!(book.snapshot(), Some(&original));
    }
}

#[test]
fn vwap_walks_correct_side_and_requires_enough_visible_depth() {
    let mut book = OrderBook::new(Coin::new("BTC").unwrap());
    book.replace(snapshot(10)).unwrap();
    assert_eq!(
        book.vwap(Side::Buy, Quantity::new(dec!(2)).unwrap())
            .unwrap(),
        dec!(101.5)
    );
    assert_eq!(
        book.vwap(Side::Sell, Quantity::new(dec!(4)).unwrap())
            .unwrap(),
        dec!(98.5)
    );
    assert_eq!(
        book.vwap(Side::Buy, Quantity::new(dec!(4)).unwrap()),
        Err(Error::InsufficientLiquidity)
    );
    assert!(book
        .vwap(Side::Buy, Quantity::new(dec!(0)).unwrap())
        .is_err());
}

#[test]
fn oversized_notional_is_a_typed_error() {
    let mut book = OrderBook::new(Coin::new("BTC").unwrap());
    let mut huge = snapshot(10);
    huge.levels[1] = vec![hl_depth::wire::Level {
        px: Price::new(Decimal::MAX).unwrap(),
        sz: Quantity::new(dec!(2)).unwrap(),
        n: 1,
    }];
    book.replace(huge).unwrap();
    assert_eq!(
        book.vwap(Side::Buy, Quantity::new(dec!(2)).unwrap()),
        Err(Error::Overflow)
    );
}
