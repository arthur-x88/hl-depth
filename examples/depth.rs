//! Executable examples and regression checks for the public market-data API.
use hl_depth::{
    book::OrderBook,
    market::{MarketKind, MarketRules},
    types::{Coin, Price, Quantity, Side},
    wire::BookSnapshot,
};
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot: BookSnapshot = serde_json::from_str(
        r#"{
        "coin":"BTC","time":1700000000000,"levels":[
            [{"px":"65000","sz":"0.5","n":3},{"px":"64999","sz":"1.2","n":4}],
            [{"px":"65001","sz":"0.4","n":2},{"px":"65002","sz":"1.1","n":5}]
        ]}"#,
    )?;
    let mut book = OrderBook::new(Coin::new("BTC")?);
    book.replace(snapshot)?;
    let rules = MarketRules::new(MarketKind::Perpetual, 5)?;
    rules.validate_price(Price::new(dec!(65001))?)?;
    rules.validate_size(Quantity::new(dec!(0.75))?)?;
    println!("Synthetic BTC snapshot (not live data)");
    println!(
        "Spread: {} | Mid: {}",
        book.spread().unwrap(),
        book.mid_price().unwrap()
    );
    println!(
        "Buy 0.75 BTC visible-depth VWAP: {}",
        book.vwap(Side::Buy, Quantity::new(dec!(0.75))?)?
    );
    println!(
        "Fractional price 65001.1 accepted: {}",
        rules.validate_price(Price::new(dec!(65001.1))?).is_ok()
    );
    Ok(())
}
