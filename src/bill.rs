use std::collections::{BTreeMap, BTreeSet};

use crate::edge::{iata_code, EdgeMap};
use crate::error::Error;
use crate::money::{is_month, mul_div_round};
use crate::parse::Record;
use crate::price::PriceBook;

/// Maandrekening. De gratis 1 TB en de gratis verzoeken gelden per maand,
/// over de prijsregio's samen, en gaan van de duurste eenheden af.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub source_url: String,
    pub checked_on: String,
    pub bytes_per_gb: u128,
    pub free_data_bytes: u128,
    pub free_requests: u128,
    pub months: Vec<MonthBill>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthBill {
    pub month: String,
    pub regions: Vec<RegionRow>,
    pub paths: Vec<PathRow>,
    pub unknown_codes: Vec<String>,
    pub total_usd_micros: u128,
    pub free_data_bytes_applied: u128,
    pub free_requests_applied: u128,
    pub unpriced_bytes: u128,
    pub unpriced_requests: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionRow {
    pub id: String,
    pub label: String,
    pub edges: Vec<String>,
    pub priced: bool,
    pub bytes: u128,
    pub http_requests: u128,
    pub https_requests: u128,
    pub other_requests: u128,
    pub usd_micros: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRow {
    pub path: String,
    pub bytes: u128,
    pub requests: u128,
    pub usd_micros: u128,
    pub unpriced_bytes: u128,
    pub unpriced_requests: u128,
}

struct Slice {
    region: usize,
    bytes: u128,
    rate: u64,
    tier: usize,
}

struct ReqSlice {
    region: usize,
    https: bool,
    count: u128,
    rate: u64,
}

struct DataPriced {
    usd_micros: Vec<u128>,
    free_bytes_applied: u128,
}

struct RequestsPriced {
    http_usd_micros: Vec<u128>,
    https_usd_micros: Vec<u128>,
    free_requests_applied: u128,
}

#[derive(Default)]
struct Acc {
    bytes: u128,
    http: u128,
    https: u128,
    other: u128,
    codes: BTreeSet<String>,
}

struct Slot {
    bytes: u128,
    http: u128,
    https: u128,
    other: u128,
}

pub fn build_report(
    prices: &PriceBook,
    edges: &EdgeMap,
    records: &[Record],
    month: Option<&str>,
) -> Result<Report, Error> {
    edges.check_regions(prices)?;
    if let Some(only) = month {
        if !is_month(only) {
            return Err(Error::new(format!("maand {only} is ongeldig")));
        }
    }
    let mut by_month: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
    for record in records {
        if !is_month(&record.month) {
            return Err(Error::new(format!("maand {} is ongeldig", record.month)));
        }
        if month.is_some_and(|only| only != record.month) {
            continue;
        }
        by_month
            .entry(record.month.clone())
            .or_default()
            .push(record);
    }
    let mut months = Vec::new();
    for (name, rows) in by_month {
        months.push(price_month(prices, edges, &name, &rows)?);
    }
    Ok(Report {
        source_url: prices.source_url.clone(),
        checked_on: prices.checked_on.clone(),
        bytes_per_gb: prices.bytes_per_gb,
        free_data_bytes: prices.free_data_bytes,
        free_requests: prices.free_requests,
        months,
    })
}

fn price_month(
    prices: &PriceBook,
    edges: &EdgeMap,
    month: &str,
    records: &[&Record],
) -> Result<MonthBill, Error> {
    let width = prices.regions.len();
    let mut regions = Vec::new();
    regions.resize_with(width, Acc::default);
    let mut unknown = Acc::default();
    let known = prices.index_map();
    let mut path_reg: BTreeMap<(String, usize), Slot> = BTreeMap::new();
    let mut path_unknown: BTreeMap<String, Slot> = BTreeMap::new();

    for record in records {
        let bytes = u128::from(record.bytes);
        let protocol = record.protocol.to_ascii_lowercase();
        match edges.region_id(&record.location) {
            Some(region_id) => {
                let Some(&index) = known.get(region_id) else {
                    return Err(Error::new(format!("prijsregio {region_id} ontbreekt")));
                };
                let Some(code) = iata_code(&record.location) else {
                    return Err(Error::new("edge zonder IATA-code"));
                };
                let acc = &mut regions[index];
                acc.codes.insert(code);
                add_u128(&mut acc.bytes, bytes)?;
                let slot = path_reg
                    .entry((record.path.clone(), index))
                    .or_insert(Slot {
                        bytes: 0,
                        http: 0,
                        https: 0,
                        other: 0,
                    });
                add_u128(&mut slot.bytes, bytes)?;
                match protocol.as_str() {
                    "http" => {
                        add_u128(&mut acc.http, 1)?;
                        add_u128(&mut slot.http, 1)?;
                    }
                    "https" => {
                        add_u128(&mut acc.https, 1)?;
                        add_u128(&mut slot.https, 1)?;
                    }
                    _ => {
                        add_u128(&mut acc.other, 1)?;
                        add_u128(&mut slot.other, 1)?;
                    }
                }
            }
            None => {
                let code = iata_code(&record.location).unwrap_or_else(|| record.location.clone());
                unknown.codes.insert(code);
                add_u128(&mut unknown.bytes, bytes)?;
                let slot = path_unknown.entry(record.path.clone()).or_insert(Slot {
                    bytes: 0,
                    http: 0,
                    https: 0,
                    other: 0,
                });
                add_u128(&mut slot.bytes, bytes)?;
                match protocol.as_str() {
                    "http" => {
                        add_u128(&mut unknown.http, 1)?;
                        add_u128(&mut slot.http, 1)?;
                    }
                    "https" => {
                        add_u128(&mut unknown.https, 1)?;
                        add_u128(&mut slot.https, 1)?;
                    }
                    _ => {
                        add_u128(&mut unknown.other, 1)?;
                        add_u128(&mut slot.other, 1)?;
                    }
                }
            }
        }
    }

    let byte_rows: Vec<u128> = regions.iter().map(|acc| acc.bytes).collect();
    let data = price_data(&byte_rows, prices)?;
    let mut request_rows = Vec::new();
    for (index, acc) in regions.iter().enumerate() {
        if acc.http > 0 {
            request_rows.push((index, false, acc.http));
        }
        if acc.https > 0 {
            request_rows.push((index, true, acc.https));
        }
    }
    let requests = price_requests(&request_rows, prices)?;

    let mut path_rows: BTreeMap<String, PathRow> = BTreeMap::new();
    for ((path, _), slot) in &path_reg {
        let row = path_rows.entry(path.clone()).or_insert_with(|| PathRow {
            path: path.clone(),
            bytes: 0,
            requests: 0,
            usd_micros: 0,
            unpriced_bytes: 0,
            unpriced_requests: 0,
        });
        add_u128(&mut row.bytes, slot.bytes)?;
        add_u128(&mut row.requests, slot.http)?;
        add_u128(&mut row.requests, slot.https)?;
        add_u128(&mut row.requests, slot.other)?;
        add_u128(&mut row.unpriced_requests, slot.other)?;
    }
    for (path, slot) in &path_unknown {
        let row = path_rows.entry(path.clone()).or_insert_with(|| PathRow {
            path: path.clone(),
            bytes: 0,
            requests: 0,
            usd_micros: 0,
            unpriced_bytes: 0,
            unpriced_requests: 0,
        });
        let calls = slot.http + slot.https + slot.other;
        add_u128(&mut row.bytes, slot.bytes)?;
        add_u128(&mut row.requests, calls)?;
        add_u128(&mut row.unpriced_bytes, slot.bytes)?;
        add_u128(&mut row.unpriced_requests, calls)?;
    }

    // One pass over the (path, region) pairs; the pairs of a region stay in path order.
    let mut by_region: Vec<RegionPaths<'_>> = Vec::new();
    by_region.resize_with(width, Vec::new);
    for ((path, region), slot) in &path_reg {
        by_region[*region].push((path, slot));
    }
    for (index, rows) in by_region.iter().enumerate() {
        grant(&mut path_rows, data.usd_micros[index], &weights_bytes(rows))?;
        grant(
            &mut path_rows,
            requests.http_usd_micros[index],
            &weights_count(rows, false),
        )?;
        grant(
            &mut path_rows,
            requests.https_usd_micros[index],
            &weights_count(rows, true),
        )?;
    }

    let mut region_rows = Vec::new();
    let mut total = 0u128;
    for (index, acc) in regions.iter().enumerate() {
        if acc.bytes == 0 && acc.http == 0 && acc.https == 0 && acc.other == 0 {
            continue;
        }
        let usd = data.usd_micros[index]
            + requests.http_usd_micros[index]
            + requests.https_usd_micros[index];
        add_u128(&mut total, usd)?;
        region_rows.push(RegionRow {
            id: prices.regions[index].id.clone(),
            label: prices.regions[index].label.clone(),
            edges: acc.codes.iter().cloned().collect(),
            priced: true,
            bytes: acc.bytes,
            http_requests: acc.http,
            https_requests: acc.https,
            other_requests: acc.other,
            usd_micros: usd,
        });
    }
    let unknown_calls = unknown.http + unknown.https + unknown.other;
    if unknown.bytes > 0 || unknown_calls > 0 {
        region_rows.push(RegionRow {
            id: "onbekend".to_string(),
            label: "onbekend".to_string(),
            edges: unknown.codes.iter().cloned().collect(),
            priced: false,
            bytes: unknown.bytes,
            http_requests: unknown.http,
            https_requests: unknown.https,
            other_requests: unknown.other,
            usd_micros: 0,
        });
    }
    let mut unpriced_requests = unknown_calls;
    for acc in &regions {
        add_u128(&mut unpriced_requests, acc.other)?;
    }
    Ok(MonthBill {
        month: month.to_string(),
        regions: region_rows,
        paths: path_rows.into_values().collect(),
        unknown_codes: unknown.codes.into_iter().collect(),
        total_usd_micros: total,
        free_data_bytes_applied: data.free_bytes_applied,
        free_requests_applied: requests.free_requests_applied,
        unpriced_bytes: unknown.bytes,
        unpriced_requests,
    })
}

/// The paths of one region, in path order, with their slots.
type RegionPaths<'a> = Vec<(&'a String, &'a Slot)>;

fn weights_bytes(rows: &RegionPaths<'_>) -> Vec<(String, u128)> {
    rows.iter()
        .filter(|(_, slot)| slot.bytes > 0)
        .map(|(path, slot)| ((*path).clone(), slot.bytes))
        .collect()
}

fn weights_count(rows: &RegionPaths<'_>, https: bool) -> Vec<(String, u128)> {
    rows.iter()
        .filter_map(|(path, slot)| {
            let count = if https { slot.https } else { slot.http };
            (count > 0).then(|| ((*path).clone(), count))
        })
        .collect()
}

fn grant(
    paths: &mut BTreeMap<String, PathRow>,
    total: u128,
    weights: &[(String, u128)],
) -> Result<(), Error> {
    if total == 0 {
        return Ok(());
    }
    for (path, micros) in allocate(total, weights)? {
        let Some(row) = paths.get_mut(&path) else {
            return Err(Error::new("pad ontbreekt bij de verdeling"));
        };
        add_u128(&mut row.usd_micros, micros)?;
    }
    Ok(())
}

fn price_data(bytes_per_region: &[u128], book: &PriceBook) -> Result<DataPriced, Error> {
    if bytes_per_region.len() != book.regions.len() {
        return Err(Error::new("regio-aantallen wijken af"));
    }
    let ends = tier_ends(book.bytes_per_gb)?;
    let mut slices = Vec::new();
    for (region, bytes) in bytes_per_region.iter().copied().enumerate() {
        if bytes == 0 {
            continue;
        }
        for (len, rate, tier) in segments(bytes, book.regions[region].data_micros_per_gb, ends) {
            slices.push(Slice {
                region,
                bytes: len,
                rate,
                tier,
            });
        }
    }
    slices.sort_by(|left, right| {
        right
            .rate
            .cmp(&left.rate)
            .then(left.region.cmp(&right.region))
            .then(left.tier.cmp(&right.tier))
    });
    let mut usd_micros = vec![0u128; book.regions.len()];
    let mut free_left = book.free_data_bytes;
    let mut free_bytes_applied = 0u128;
    for slice in slices {
        let waive = slice.bytes.min(free_left);
        free_left -= waive;
        add_u128(&mut free_bytes_applied, waive)?;
        let billable = slice.bytes - waive;
        if billable == 0 {
            continue;
        }
        let cost = mul_div_round(billable, u128::from(slice.rate), book.bytes_per_gb)?;
        add_u128(&mut usd_micros[slice.region], cost)?;
    }
    Ok(DataPriced {
        usd_micros,
        free_bytes_applied,
    })
}

fn price_requests(rows: &[(usize, bool, u128)], book: &PriceBook) -> Result<RequestsPriced, Error> {
    let mut slices = Vec::new();
    for &(region, https, count) in rows {
        if count == 0 {
            continue;
        }
        let Some(price) = book.regions.get(region) else {
            return Err(Error::new("verzoeken voor een ontbrekende regio"));
        };
        let rate = if https {
            price.https_micros_per_10k
        } else {
            price.http_micros_per_10k
        };
        slices.push(ReqSlice {
            region,
            https,
            count,
            rate,
        });
    }
    slices.sort_by(|left, right| {
        right
            .rate
            .cmp(&left.rate)
            .then(left.region.cmp(&right.region))
            .then(right.https.cmp(&left.https))
    });
    let mut http_usd_micros = vec![0u128; book.regions.len()];
    let mut https_usd_micros = vec![0u128; book.regions.len()];
    let mut free_left = book.free_requests;
    let mut free_requests_applied = 0u128;
    for slice in slices {
        let waive = slice.count.min(free_left);
        free_left -= waive;
        add_u128(&mut free_requests_applied, waive)?;
        let billable = slice.count - waive;
        if billable == 0 {
            continue;
        }
        let cost = mul_div_round(billable, u128::from(slice.rate), 10_000)?;
        let slot = if slice.https {
            &mut https_usd_micros[slice.region]
        } else {
            &mut http_usd_micros[slice.region]
        };
        add_u128(slot, cost)?;
    }
    Ok(RequestsPriced {
        http_usd_micros,
        https_usd_micros,
        free_requests_applied,
    })
}

fn tier_ends(bytes_per_gb: u128) -> Result<[u128; 6], Error> {
    let tb = bytes_per_gb
        .checked_mul(1024)
        .ok_or_else(|| Error::new("overflow in 1 TB"))?;
    let pb = tb
        .checked_mul(1024)
        .ok_or_else(|| Error::new("overflow in 1 PB"))?;
    Ok([
        mul(tb, 10)?,
        mul(tb, 50)?,
        mul(tb, 150)?,
        mul(tb, 500)?,
        pb,
        mul(pb, 5)?,
    ])
}

fn segments(bytes: u128, rates: [u64; 7], ends: [u128; 6]) -> Vec<(u128, u64, usize)> {
    let mut out = Vec::new();
    let mut start = 0u128;
    for (tier, end) in ends.iter().enumerate() {
        if bytes <= start {
            break;
        }
        let cut = (*end).min(bytes);
        if cut > start {
            out.push((cut - start, rates[tier], tier));
        }
        if bytes <= *end {
            return out;
        }
        start = *end;
    }
    if bytes > start {
        out.push((bytes - start, rates[6], 6));
    }
    out
}

fn allocate(total: u128, weights: &[(String, u128)]) -> Result<Vec<(String, u128)>, Error> {
    if weights.is_empty() {
        return Err(Error::new("geen verdeling voor een bedrag"));
    }
    let sum = weights.iter().try_fold(0u128, |acc, (_, weight)| {
        acc.checked_add(*weight)
            .ok_or_else(|| Error::new(crate::error::OVERFLOW))
    })?;
    if sum == 0 {
        return Err(Error::new("geen verdeling voor een bedrag"));
    }
    let mut parts = Vec::with_capacity(weights.len());
    let mut assigned = 0u128;
    for (key, weight) in weights {
        let product = total
            .checked_mul(*weight)
            .ok_or_else(|| Error::new(crate::error::OVERFLOW))?;
        let base = product / sum;
        add_u128(&mut assigned, base)?;
        parts.push((key.clone(), base, product % sum, *weight));
    }
    let mut left = total - assigned;
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by(|&left_i, &right_i| {
        parts[right_i]
            .2
            .cmp(&parts[left_i].2)
            .then(parts[right_i].3.cmp(&parts[left_i].3))
            .then_with(|| parts[left_i].0.cmp(&parts[right_i].0))
    });
    let mut cursor = 0;
    while left > 0 {
        let index = order[cursor % order.len()];
        add_u128(&mut parts[index].1, 1)?;
        left -= 1;
        cursor += 1;
    }
    let distributed = parts.iter().try_fold(0u128, |acc, part| {
        acc.checked_add(part.1)
            .ok_or_else(|| Error::new(crate::error::OVERFLOW))
    })?;
    if distributed != total {
        return Err(Error::new("verdeling sluit niet"));
    }
    Ok(parts
        .into_iter()
        .map(|(key, base, _, _)| (key, base))
        .collect())
}

fn add_u128(slot: &mut u128, value: u128) -> Result<(), Error> {
    *slot = slot
        .checked_add(value)
        .ok_or_else(|| Error::new(crate::error::OVERFLOW))?;
    Ok(())
}

fn mul(value: u128, factor: u128) -> Result<u128, Error> {
    value
        .checked_mul(factor)
        .ok_or_else(|| Error::new(crate::error::OVERFLOW))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::load_logs;
    use std::path::PathBuf;

    const GB: u128 = 1_073_741_824;

    fn prices() -> PriceBook {
        crate::default_prices().unwrap()
    }

    fn edges() -> EdgeMap {
        crate::default_edges().unwrap()
    }

    fn gb(count: u128) -> u64 {
        u64::try_from(count * GB).unwrap()
    }

    fn rec(month: &str, location: &str, bytes: u64, path: &str, protocol: &str) -> Record {
        Record {
            month: month.to_string(),
            location: location.to_string(),
            bytes,
            path: path.to_string(),
            protocol: protocol.to_string(),
        }
    }

    fn bill(records: &[Record]) -> MonthBill {
        build_report(&prices(), &edges(), records, None)
            .unwrap()
            .months
            .pop()
            .unwrap()
    }

    fn us_only(bytes: u64) -> MonthBill {
        bill(&[rec("2026-01", "IAD89-P1", bytes, "/a", "https")])
    }

    #[test]
    fn record_with_invalid_month_is_rejected() {
        let records = [rec("2026-13", "IAD89-P1", 1, "/a", "https")];
        let err = build_report(&prices(), &edges(), &records, None).unwrap_err();
        assert_eq!(err.message, "maand 2026-13 is ongeldig");
    }

    #[test]
    fn requested_month_that_is_invalid_is_rejected() {
        let err = build_report(&prices(), &edges(), &[], Some("2026-13")).unwrap_err();
        assert_eq!(err.message, "maand 2026-13 is ongeldig");
    }

    #[test]
    fn missing_price_region_error_text_is_fixed() {
        let map = crate::edge::load_edges("AMS no_such_region\n").unwrap();
        let record = rec("2026-01", "AMS50-C1", 1, "/a", "https");
        let err = price_month(&prices(), &map, "2026-01", &[&record]).unwrap_err();
        assert_eq!(err.message, "prijsregio no_such_region ontbreekt");
    }

    #[test]
    fn internal_error_texts_are_fixed() {
        let mut paths = BTreeMap::new();
        let err = grant(&mut paths, 10, &[("a".to_string(), 1)]).unwrap_err();
        assert_eq!(err.message, "pad ontbreekt bij de verdeling");
        let err = price_data(&[1], &prices()).err().unwrap();
        assert_eq!(err.message, "regio-aantallen wijken af");
        let err = price_requests(&[(99, true, 5)], &prices()).err().unwrap();
        assert_eq!(err.message, "verzoeken voor een ontbrekende regio");
        assert_eq!(
            tier_ends(u128::MAX).unwrap_err().message,
            "overflow in 1 TB"
        );
        assert_eq!(
            tier_ends(u128::MAX / 1024).unwrap_err().message,
            "overflow in 1 PB"
        );
        assert_eq!(
            allocate(5, &[]).unwrap_err().message,
            "geen verdeling voor een bedrag"
        );
        assert_eq!(
            allocate(5, &[("a".to_string(), 0)]).unwrap_err().message,
            "geen verdeling voor een bedrag"
        );
        assert_eq!(
            allocate(1, &[("a".to_string(), u128::MAX), ("b".to_string(), 1)])
                .unwrap_err()
                .message,
            crate::error::OVERFLOW
        );
    }

    #[test]
    fn one_tebibyte_in_one_region_is_free() {
        let month = us_only(gb(1024));
        assert_eq!(month.total_usd_micros, 0);
        assert_eq!(month.free_data_bytes_applied, 1024 * GB);
    }

    #[test]
    fn one_gb_above_the_free_terabyte_uses_the_us_rate() {
        let month = us_only(gb(1025));
        assert_eq!(month.total_usd_micros, 85_000);
        assert_eq!(month.free_data_bytes_applied, 1024 * GB);
    }

    #[test]
    fn ten_tb_bills_nine_tb_at_the_first_rate() {
        let month = us_only(gb(10 * 1024));
        assert_eq!(month.total_usd_micros, 9 * 1024 * 85_000);
    }

    #[test]
    fn one_gb_past_ten_tb_uses_the_second_rate() {
        let month = us_only(gb(10 * 1024 + 1));
        assert_eq!(month.total_usd_micros, 9 * 1024 * 85_000 + 80_000);
    }

    #[test]
    fn free_terabyte_comes_off_the_dearer_region() {
        let month = bill(&[
            rec("2026-01", "IAD1", gb(2048), "/us", "http"),
            rec("2026-01", "BOM1", gb(2048), "/in", "https"),
        ]);
        let us = region(&month, "us_mexico_canada");
        let india = region(&month, "india");
        assert_eq!(us.usd_micros, 2048 * 85_000);
        assert_eq!(india.usd_micros, 1024 * 109_000);
    }

    #[test]
    fn equal_rates_waive_the_earlier_region() {
        let month = bill(&[
            rec("2026-01", "IAD1", gb(2048), "/us", "http"),
            rec("2026-01", "AMS1", gb(2048), "/eu", "https"),
        ]);
        assert_eq!(region(&month, "us_mexico_canada").usd_micros, 1024 * 85_000);
        assert_eq!(
            region(&month, "europe_israel_turkiye").usd_micros,
            2048 * 85_000
        );
    }

    #[test]
    fn both_regions_under_one_tb_stay_free() {
        let month = bill(&[
            rec("2026-01", "IAD1", gb(100), "/us", "http"),
            rec("2026-01", "AMS1", gb(100), "/eu", "https"),
        ]);
        assert_eq!(month.total_usd_micros, 0);
        assert_eq!(month.free_data_bytes_applied, 200 * GB);
    }

    #[test]
    fn fifty_tb_plus_one_gb_uses_the_third_rate() {
        let month = us_only(gb(50 * 1024 + 1));
        let expected = 9 * 1024 * 85_000 + 40 * 1024 * 80_000 + 60_000;
        assert_eq!(month.total_usd_micros, expected);
    }

    #[test]
    fn one_pb_plus_one_gb_uses_the_four_pb_rate() {
        let month = us_only(gb(1024 * 1024 + 1));
        let expected = 9 * 1024 * 85_000
            + 40 * 1024 * 80_000
            + 100 * 1024 * 60_000
            + 350 * 1024 * 40_000
            + 524 * 1024 * 30_000
            + 25_000;
        assert_eq!(month.total_usd_micros, expected);
    }

    #[test]
    fn bytes_past_five_pb_use_the_over_rate() {
        let month = us_only(gb(5 * 1024 * 1024 + 1));
        let expected = 9 * 1024 * 85_000
            + 40 * 1024 * 80_000
            + 100 * 1024 * 60_000
            + 350 * 1024 * 40_000
            + 524 * 1024 * 30_000
            + 4096 * 1024 * 25_000
            + 20_000;
        assert_eq!(month.total_usd_micros, expected);
    }

    #[test]
    fn requests_inside_the_free_allowance_cost_nothing() {
        let book = prices();
        let sa = book.index("south_america").unwrap();
        let small = price_requests(&[(sa, true, 100)], &book).unwrap();
        assert_eq!(small.https_usd_micros[sa], 0);
        assert_eq!(small.free_requests_applied, 100);
        let full = price_requests(&[(sa, true, 10_000_000)], &book).unwrap();
        assert_eq!(full.https_usd_micros[sa], 0);
        assert_eq!(full.free_requests_applied, 10_000_000);
    }

    #[test]
    fn https_above_the_free_requests_uses_the_table() {
        let book = prices();
        let sa = book.index("south_america").unwrap();
        let priced = price_requests(&[(sa, true, 10_010_000)], &book).unwrap();
        assert_eq!(priced.free_requests_applied, 10_000_000);
        assert_eq!(priced.https_usd_micros[sa], 22_000);
    }

    #[test]
    fn free_requests_come_off_the_dearer_request() {
        let book = prices();
        let us = book.index("us_mexico_canada").unwrap();
        let sa = book.index("south_america").unwrap();
        let priced = price_requests(&[(us, false, 10_000_000), (sa, true, 10_000)], &book).unwrap();
        assert_eq!(priced.https_usd_micros[sa], 0);
        assert_eq!(priced.http_usd_micros[us], 7_500);
    }

    #[test]
    fn us_http_and_https_rates_differ() {
        let book = prices();
        let us = book.index("us_mexico_canada").unwrap();
        let http = price_requests(&[(us, false, 10_010_000)], &book).unwrap();
        let https = price_requests(&[(us, true, 10_010_000)], &book).unwrap();
        assert_eq!(http.http_usd_micros[us], 7_500);
        assert_eq!(https.https_usd_micros[us], 10_000);
    }

    #[test]
    fn equal_request_rates_waive_https_before_http() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 1000
free_data_bytes 1000000000
free_requests 1
label us US
region us 0 0 0 0 0 0 0 0.0100 0.0100
",
            &[
                rec("2026-01", "IAD1", 0, "/http", "http"),
                rec("2026-01", "IAD1", 0, "/https", "https"),
            ],
        );
        assert_eq!(month.free_requests_applied, 1);
        assert_eq!(month.total_usd_micros, 1);
        assert_eq!(path(&month, "/https").usd_micros, 0);
        assert_eq!(path(&month, "/http").usd_micros, 1);
    }

    #[test]
    fn unknown_bytes_do_not_consume_the_free_terabyte() {
        let month = bill(&[
            rec("2026-01", "ZZZ1", gb(1024), "/x", "https"),
            rec("2026-01", "IAD1", gb(2048), "/a", "http"),
        ]);
        assert_eq!(region(&month, "us_mexico_canada").usd_micros, 1024 * 85_000);
        assert_eq!(month.unpriced_bytes, 1024 * GB);
        assert_eq!(month.free_data_bytes_applied, 1024 * GB);
    }

    #[test]
    fn unknown_requests_do_not_consume_the_free_requests() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 1000
free_data_bytes 1000000000
free_requests 1
label us US
region us 0 0 0 0 0 0 0 0.0100 0.0100
",
            &[
                rec("2026-01", "IAD1", 0, "/a", "http"),
                rec("2026-01", "IAD1", 0, "/a", "http"),
                rec("2026-01", "ZZZ1", 0, "/x", "https"),
            ],
        );
        assert_eq!(month.total_usd_micros, 1);
        assert_eq!(month.free_requests_applied, 1);
        assert_eq!(month.unpriced_requests, 1);
    }

    #[test]
    fn other_protocol_does_not_consume_free_requests() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 1000
free_data_bytes 1000000000
free_requests 1
label us US
region us 0 0 0 0 0 0 0 0.0100 0.0100
",
            &[
                rec("2026-01", "IAD1", 0, "/a", "ws"),
                rec("2026-01", "IAD1", 0, "/a", "http"),
            ],
        );
        assert_eq!(month.total_usd_micros, 0);
        assert_eq!(month.free_requests_applied, 1);
        assert_eq!(month.unpriced_requests, 1);
    }

    #[test]
    fn months_each_get_their_own_free_terabyte() {
        let report = build_report(
            &prices(),
            &edges(),
            &[
                rec("2026-01", "IAD1", gb(1025), "/a", "https"),
                rec("2026-02", "IAD1", gb(1025), "/a", "https"),
            ],
            None,
        )
        .unwrap();
        assert_eq!(report.months.len(), 2);
        assert_eq!(report.months[0].total_usd_micros, 85_000);
        assert_eq!(report.months[1].total_usd_micros, 85_000);
    }

    #[test]
    fn path_costs_sum_to_the_month_total() {
        let month = bill(&[
            rec("2026-01", "IAD1", gb(1025), "/a", "https"),
            rec("2026-01", "IAD1", gb(1), "/b", "https"),
        ]);
        assert_eq!(month.total_usd_micros, 170_000);
        let sum: u128 = month.paths.iter().map(|path| path.usd_micros).sum();
        assert_eq!(sum, month.total_usd_micros);
        let a = path(&month, "/a");
        let b = path(&month, "/b");
        assert!(a.usd_micros > b.usd_micros);
    }

    #[test]
    fn remainder_micros_go_to_the_larger_remainder() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 3
free_data_bytes 0
free_requests 0
label us US
region us 0.000010 0.000010 0.000010 0.000010 0.000010 0.000010 0.000010 0 0
",
            &[
                rec("2026-01", "IAD1", 1, "/a", "http"),
                rec("2026-01", "IAD1", 2, "/b", "http"),
            ],
        );
        assert_eq!(month.total_usd_micros, 10);
        assert_eq!(path(&month, "/a").usd_micros, 3);
        assert_eq!(path(&month, "/b").usd_micros, 7);
    }

    #[test]
    fn a_larger_remainder_beats_a_larger_weight() {
        // 3 bytes are one gigabyte at `bytes_per_gb 3`: 2 micro-USD, split over weights 1 and 2.
        // /a: 2 * 1 / 3 = 0 remainder 2; /b: 2 * 2 / 3 = 1 remainder 1. The spare micro goes to /a.
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 3
free_data_bytes 0
free_requests 0
label us US
region us 0.000002 0.000002 0.000002 0.000002 0.000002 0.000002 0.000002 0 0
",
            &[
                rec("2026-01", "IAD1", 1, "/a", "http"),
                rec("2026-01", "IAD1", 2, "/b", "http"),
            ],
        );
        assert_eq!(month.total_usd_micros, 2);
        assert_eq!(path(&month, "/a").usd_micros, 1);
        assert_eq!(path(&month, "/b").usd_micros, 1);
    }

    #[test]
    fn equal_remainder_goes_to_the_larger_weight() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 4
free_data_bytes 0
free_requests 0
label us US
region us 0.000002 0.000002 0.000002 0.000002 0.000002 0.000002 0.000002 0 0
",
            &[
                rec("2026-01", "IAD1", 1, "/a", "http"),
                rec("2026-01", "IAD1", 3, "/b", "http"),
            ],
        );
        assert_eq!(month.total_usd_micros, 2);
        assert_eq!(path(&month, "/a").usd_micros, 0);
        assert_eq!(path(&month, "/b").usd_micros, 2);
    }

    #[test]
    fn equal_weight_remainder_goes_to_the_earlier_path() {
        let month = custom_bill(
            "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 2
free_data_bytes 0
free_requests 0
label us US
region us 0.000001 0.000001 0.000001 0.000001 0.000001 0.000001 0.000001 0 0
",
            &[
                rec("2026-01", "IAD1", 1, "/b", "http"),
                rec("2026-01", "IAD1", 1, "/a", "http"),
            ],
        );
        assert_eq!(month.total_usd_micros, 1);
        assert_eq!(path(&month, "/a").usd_micros, 1);
        assert_eq!(path(&month, "/b").usd_micros, 0);
    }

    #[test]
    fn japan_one_gb_above_free_uses_the_japan_rate() {
        let month = bill(&[rec("2026-01", "NRT12", gb(1025), "/a", "https")]);
        assert_eq!(region(&month, "japan").usd_micros, 114_000);
    }

    #[test]
    fn fixture_has_two_regions_and_two_paths() {
        let month = fixture();
        assert_eq!(region(&month, "us_mexico_canada").bytes, 2000);
        assert_eq!(
            region(&month, "us_mexico_canada").edges,
            vec!["IAD".to_string()]
        );
        assert_eq!(region(&month, "europe_israel_turkiye").bytes, 1500);
        assert_eq!(
            region(&month, "europe_israel_turkiye").edges,
            vec!["AMS".to_string(), "FRA".to_string()]
        );
        assert_eq!(path(&month, "/video.mp4").bytes, 1500);
        assert_eq!(path(&month, "/img.png").bytes, 2000);
        assert_eq!(month.total_usd_micros, 0);
    }

    #[test]
    fn fixture_labels_unknown_edge_onbekend() {
        let month = fixture();
        let unknown = region(&month, "onbekend");
        assert!(!unknown.priced);
        assert_eq!(unknown.bytes, 50);
        assert_eq!(unknown.edges, vec!["ZZZ".to_string()]);
        assert_eq!(month.unpriced_bytes, 50);
        assert_eq!(month.unpriced_requests, 1);
        assert_eq!(path(&month, "/geheim").unpriced_bytes, 50);
    }

    #[test]
    fn zero_byte_request_still_counts() {
        let month = bill(&[rec("2026-01", "IAD1", 0, "/a", "https")]);
        let row = region(&month, "us_mexico_canada");
        assert_eq!(row.bytes, 0);
        assert_eq!(row.https_requests, 1);
        assert_eq!(month.free_requests_applied, 1);
        assert_eq!(month.total_usd_micros, 0);
    }

    fn region<'a>(month: &'a MonthBill, id: &str) -> &'a RegionRow {
        month.regions.iter().find(|row| row.id == id).unwrap()
    }

    fn path<'a>(month: &'a MonthBill, name: &str) -> &'a PathRow {
        month.paths.iter().find(|row| row.path == name).unwrap()
    }

    fn fixture() -> MonthBill {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/twee_regios.log");
        let records = load_logs(&[path]).unwrap();
        bill(&records)
    }

    fn custom_bill(text: &str, records: &[Record]) -> MonthBill {
        let book = crate::load_prices(text).unwrap();
        let map = crate::load_edges("IAD us\n").unwrap();
        build_report(&book, &map, records, None)
            .unwrap()
            .months
            .pop()
            .unwrap()
    }
}
