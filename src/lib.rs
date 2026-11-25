#![deny(unsafe_code)]

//! Offline-schatting van CloudFront-egressie uit standaardlogs.
//! Geen AWS SDK en geen netwerk.

mod bill;
mod edge;
mod error;
#[allow(unsafe_code)]
mod interrupt;
mod money;
mod parse;
mod price;
mod report;
#[allow(unsafe_code)]
mod stdout_probe;

pub use bill::{build_report, MonthBill, PathRow, RegionRow, Report};
pub use edge::{load_edges, EdgeMap};
pub use error::{Error, OVERFLOW};
pub use interrupt::{
    hold as hold_sigint, install as install_sigint_handler, release as release_sigint,
    EXIT_INTERRUPTED, INTERRUPTED_LINE,
};
pub use money::{format_usd, is_date, price_age_warning, today_utc, BYTES_PER_GB};
pub use parse::{check_file_size, load_logs, parse_log, path_text, read_log_file, Record};
pub use price::{load_prices, PriceBook, RegionPrice};
pub use report::{render_json, render_markdown, render_text, EXCLUDES, NOTE};
pub use stdout_probe::stdout_was_closed_at_start;

pub fn embedded_price_text() -> &'static str {
    include_str!("../data/prices.txt")
}

pub fn embedded_edge_text() -> &'static str {
    include_str!("../data/edges.txt")
}

pub fn default_prices() -> Result<PriceBook, Error> {
    load_prices(embedded_price_text())
}

pub fn default_edges() -> Result<EdgeMap, Error> {
    let edges = load_edges(embedded_edge_text())?;
    let prices = default_prices()?;
    edges.check_regions(&prices)?;
    Ok(edges)
}
