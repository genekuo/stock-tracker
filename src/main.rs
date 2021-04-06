use yahoo_finance_api as yahoo;
use clap::Clap;
use chrono::prelude::*;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Gene Kuo",
    about = "Milestone 1: a simple tracker"
)]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();

    let from = opts.from.parse().expect("Can't parse from date");

    //let from = NaiveDate::parse_from_str(opts.from.as_str(), "%Y-%m-%d").unwrap();
    //let from = DateTime::<Utc>::from_utc(from.and_hms(0, 0, 0), Utc);
    //let end = Utc::now();

    println!("period start,symbol,price,change %,min,max,30d avg");
    for symbol in opts.symbols.split(",") {
        let provider = yahoo::YahooConnector::new();
        if let Ok(response) = provider.get_quote_history(symbol, from, Utc::now()) {
            match response.quotes() {
                Ok(mut quotes) => {
                    if !quotes.is_empty() {
                        quotes.sort_by_cached_key(|k| k.timestamp);
                        let closes: Vec<f64> = quotes.iter().map(|q| q.adjclose as f64).collect();
                        if !closes.is_empty() {
                            let period_max: f64 = max(&closes).unwrap();
                            let period_min: f64 = min(&closes).unwrap();
                            let last_price = *closes.last().unwrap_or(&0.0);
                            let (_, pct_change) = price_diff(&closes).unwrap_or((0.0, 0.0));
                            let sma = n_window_sma(30, &closes).unwrap_or_default();
                            println!(
                                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                                from.to_rfc3339(),
                                symbol,
                                last_price,
                                pct_change * 100.0,
                                period_min,
                                period_max,
                                sma.last().unwrap_or(&0.0)
                            );
                        }
                    }
                }
                _ => {
                    eprint!("No quotes found for symbol '{}'", symbol);
                }
            }
        } else {
            eprint!("No quotes found for symbol '{}'", symbol);
        }
    }
    Ok(())
}

///
/// Calculates the absolute and relative (price) change between the beginning and ending of an f64 series. 
/// The relative (price) change is relative to the beginning.
///
/// # Returns
///
/// A tuple `(absolute, relative)` difference.
///
fn price_diff(a: &[f64]) -> Option<(f64, f64)> {
    if !a.is_empty() {
        let (first, last) = (a.first().unwrap(), a.last().unwrap());
        let abs_diff = last - first;
        let first = if *first == 0.0 { 1.0 } else { *first };
        let rel_diff = abs_diff / first;
        Some((abs_diff, rel_diff))
    } else {
        None
    }
}

///
/// Find the maximum in a series of f64
///
fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
    }
}

///
/// Find the minimum in a series of f64
///
fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
    }
}

///
/// Window function to create a simple moving average
///
fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && n > 1 {
        Some(
            series
                .windows(n)
                .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                .collect(),
        )
    } else {
        None
    }
}
