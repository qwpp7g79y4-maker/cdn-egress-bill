use std::collections::{HashMap, HashSet};

use crate::error::Error;
use crate::money::{is_date, parse_usd_micros};

/// Pay-as-you-go tarieven. Geen flat-rate, geen korting, geen Savings Bundle.
///
/// Per prijsregio loopt het maandvolume over de klassieke staffel
/// 10 TB, 40 TB, 100 TB, 350 TB, 524 TB, 4 PB en daarboven.
/// De zeven datatarieven zijn de kolommen Next 9 TB tot Over 5 PB.
/// De eerste `free_data_bytes` (1 TB op de officiële lijst) zijn gratis
/// over alle regio's samen en gaan van de duurste bytes af.
/// Bij een gelijk tarief gaat de gratis laag naar de regio die eerder
/// in dit bestand staat.
///
/// HTTP- en HTTPS-verzoeken hebben één tarief per 10_000. De eerste
/// `free_requests` (10_000_000 op de officiële lijst) zijn samen gratis
/// en gaan van de duurste verzoeken af.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceBook {
    pub source_url: String,
    pub checked_on: String,
    pub bytes_per_gb: u128,
    pub free_data_bytes: u128,
    pub free_requests: u128,
    pub regions: Vec<RegionPrice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionPrice {
    pub id: String,
    pub label: String,
    /// next 9 TB, next 40 TB, next 100 TB, next 350 TB, next 524 TB, next 4 PB, over 5 PB.
    pub data_micros_per_gb: [u64; 7],
    pub http_micros_per_10k: u64,
    pub https_micros_per_10k: u64,
}

impl PriceBook {
    pub fn index(&self, id: &str) -> Option<usize> {
        self.regions.iter().position(|region| region.id == id)
    }

    /// Region id to position, for callers that look up many ids.
    pub fn index_map(&self) -> HashMap<&str, usize> {
        let mut map = HashMap::with_capacity(self.regions.len());
        for (position, region) in self.regions.iter().enumerate() {
            map.entry(region.id.as_str()).or_insert(position);
        }
        map
    }

    pub fn bytes_per_tb(&self) -> Result<u128, Error> {
        self.bytes_per_gb
            .checked_mul(1024)
            .ok_or_else(|| Error::new("overflow in de definitie van 1 TB"))
    }
}

pub fn load_prices(text: &str) -> Result<PriceBook, Error> {
    let mut source_url = None;
    let mut checked_on = None;
    let mut bytes_per_gb = None;
    let mut free_data_bytes = None;
    let mut free_requests = None;
    let mut regions = Vec::new();
    let mut labels: Vec<(String, String)> = Vec::new();
    let mut label_ids: HashSet<String> = HashSet::new();
    let mut region_ids: HashSet<String> = HashSet::new();

    if text.starts_with('\u{feff}') {
        return Err(Error::new(
            "prijsbestand: begint met een UTF-8 BOM; sla het bestand op zonder BOM",
        ));
    }

    for (lineno, raw) in text.lines().enumerate() {
        let line_no = lineno + 1;
        let line = raw.trim_matches(is_sep);
        if line.starts_with('#') {
            continue;
        }
        let fail = |message: String| Error::new(format!("prijsbestand:{line_no}: {message}"));
        if let Some(bad) = line.chars().find(|c| c.is_whitespace() && !is_sep(*c)) {
            return Err(fail(format!(
                "ongeldig witruimteteken U+{:04X}; gebruik alleen spaties of tabs",
                bad as u32
            )));
        }
        if line.is_empty() {
            continue;
        }
        let mut parts = fields(line);
        let key = parts.next().ok_or_else(|| fail("lege regel".to_string()))?;
        match key {
            "source_url" => {
                let url = one_token(parts, "source_url").map_err(fail)?;
                if !url.starts_with("https://") {
                    return Err(fail(format!("bron-URL moet met https:// beginnen: {url}")));
                }
                set_once(&mut source_url, url, "source_url").map_err(fail)?;
            }
            "checked_on" => {
                let date = one_token(parts, "checked_on").map_err(fail)?;
                if !is_date(&date) {
                    return Err(fail(format!("checked_on is geen datum: {date}")));
                }
                set_once(&mut checked_on, date, "checked_on").map_err(fail)?;
            }
            "bytes_per_gb" => {
                let value = one_u128(parts, "bytes_per_gb").map_err(fail)?;
                if value == 0 {
                    return Err(fail("bytes_per_gb moet groter zijn dan 0".to_string()));
                }
                set_once(&mut bytes_per_gb, value, "bytes_per_gb").map_err(fail)?;
            }
            "free_data_bytes" => {
                let value = one_u128(parts, "free_data_bytes").map_err(fail)?;
                set_once(&mut free_data_bytes, value, "free_data_bytes").map_err(fail)?;
            }
            "free_requests" => {
                let value = one_u128(parts, "free_requests").map_err(fail)?;
                set_once(&mut free_requests, value, "free_requests").map_err(fail)?;
            }
            "label" => {
                let rest = line
                    .strip_prefix("label")
                    .unwrap_or(line)
                    .trim_start_matches(is_sep);
                let (id, label) = rest
                    .split_once(is_sep)
                    .ok_or_else(|| fail("label mist een tekst".to_string()))?;
                check_id(id).map_err(fail)?;
                let label = label.trim_matches(is_sep);
                if label.is_empty() {
                    return Err(fail("label mist een tekst".to_string()));
                }
                if !label_ids.insert(id.to_string()) {
                    return Err(fail(format!("label voor regio {id} staat twee keer")));
                }
                labels.push((id.to_string(), label.to_string()));
            }
            "region" => {
                let tokens: Vec<&str> = fields(line).collect();
                if tokens.len() != 11 {
                    return Err(fail(
                        "een regio heeft een id, 7 datatarieven, http en https".to_string(),
                    ));
                }
                let id = tokens[1];
                check_id(id).map_err(fail)?;
                if id == "onbekend" {
                    return Err(fail(
                        "de regio-id onbekend is gereserveerd voor codes zonder kaart".to_string(),
                    ));
                }
                if !region_ids.insert(id.to_string()) {
                    return Err(fail(format!("dubbele regio {id}")));
                }
                let mut rates = [0u64; 7];
                for (slot, token) in rates.iter_mut().enumerate() {
                    *token = parse_usd_micros(tokens[2 + slot]).map_err(|err| fail(err.message))?;
                }
                let http = parse_usd_micros(tokens[9]).map_err(|err| fail(err.message))?;
                let https = parse_usd_micros(tokens[10]).map_err(|err| fail(err.message))?;
                regions.push(RegionPrice {
                    id: id.to_string(),
                    label: id.to_string(),
                    data_micros_per_gb: rates,
                    http_micros_per_10k: http,
                    https_micros_per_10k: https,
                });
            }
            other => return Err(fail(format!("onbekende sleutel {other}"))),
        }
    }

    let source_url = source_url.ok_or_else(|| Error::new("prijsbestand: bron-URL ontbreekt"))?;
    let checked_on = checked_on.ok_or_else(|| Error::new("prijsbestand: checked_on ontbreekt"))?;
    let bytes_per_gb =
        bytes_per_gb.ok_or_else(|| Error::new("prijsbestand: bytes_per_gb ontbreekt"))?;
    let free_data_bytes =
        free_data_bytes.ok_or_else(|| Error::new("prijsbestand: free_data_bytes ontbreekt"))?;
    let free_requests =
        free_requests.ok_or_else(|| Error::new("prijsbestand: free_requests ontbreekt"))?;
    if regions.is_empty() {
        return Err(Error::new("prijsbestand: geen regio"));
    }
    let positions: HashMap<String, usize> = regions
        .iter()
        .enumerate()
        .map(|(position, region)| (region.id.clone(), position))
        .collect();
    for (id, label) in labels {
        let position = positions
            .get(&id)
            .ok_or_else(|| Error::new(format!("prijsbestand: label voor onbekende regio {id}")))?;
        regions[*position].label = label;
    }
    Ok(PriceBook {
        source_url,
        checked_on,
        bytes_per_gb,
        free_data_bytes,
        free_requests,
        regions,
    })
}

/// Geldig scheidingsteken in een prijsbestand: spatie, tab of CR (LF splitst regels).
fn is_sep(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r')
}

fn fields(line: &str) -> impl Iterator<Item = &str> {
    line.split(is_sep).filter(|part| !part.is_empty())
}

fn check_id(id: &str) -> Result<(), String> {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err("lege regio-id".to_string());
    };
    if !first.is_ascii_lowercase()
        || !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(format!("ongeldige regio-id {id}"));
    }
    Ok(())
}

fn one_token<'a>(mut parts: impl Iterator<Item = &'a str>, key: &str) -> Result<String, String> {
    let value = parts
        .next()
        .ok_or_else(|| format!("{key} mist een waarde"))?;
    if parts.next().is_some() {
        return Err(format!("{key} heeft te veel waarden"));
    }
    Ok(value.to_string())
}

fn one_u128<'a>(mut parts: impl Iterator<Item = &'a str>, key: &str) -> Result<u128, String> {
    let token = one_token(&mut parts, key)?;
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{key} is geen geheel getal"));
    }
    token
        .parse()
        .map_err(|_| format!("{key} is geen geheel getal"))
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("dubbele sleutel {key}"));
    }
    *slot = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn official() -> PriceBook {
        load_prices(include_str!("../data/prices.txt")).unwrap()
    }

    #[test]
    fn embedded_source_and_date() {
        let book = official();
        assert_eq!(
            book.source_url,
            "https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/"
        );
        assert_eq!(book.checked_on, "2026-11-09");
    }

    #[test]
    fn embedded_has_eight_regions() {
        let book = official();
        let labels = [
            ("us_mexico_canada", "US/Mexico/Canada"),
            ("europe_israel_turkiye", "Europe/Israel/Türkiye"),
            (
                "africa_middle_east",
                "South Africa/Kenya/Nigeria/Egypt/Middle East",
            ),
            ("south_america", "South America"),
            ("japan", "Japan"),
            ("australia_new_zealand", "Australia/New Zealand"),
            (
                "asia_pacific",
                "Hong Kong/Indonesia/Philippines/Singapore/South Korea/Taiwan/Thailand/Malaysia/Vietnam",
            ),
            ("india", "India"),
        ];
        assert_eq!(book.regions.len(), labels.len());
        for (region, expected) in book.regions.iter().zip(labels) {
            assert_eq!(region.id, expected.0);
            assert_eq!(region.label, expected.1);
        }
    }

    #[test]
    fn embedded_rates_match_published_table() {
        let book = official();
        let table = [
            (
                "us_mexico_canada",
                [85_000, 80_000, 60_000, 40_000, 30_000, 25_000, 20_000],
                7_500,
                10_000,
            ),
            (
                "europe_israel_turkiye",
                [85_000, 80_000, 60_000, 40_000, 30_000, 25_000, 20_000],
                9_000,
                12_000,
            ),
            (
                "africa_middle_east",
                [110_000, 105_000, 90_000, 80_000, 60_000, 50_000, 40_000],
                9_000,
                12_000,
            ),
            (
                "south_america",
                [110_000, 105_000, 90_000, 80_000, 60_000, 50_000, 40_000],
                16_000,
                22_000,
            ),
            (
                "japan",
                [114_000, 89_000, 86_000, 84_000, 80_000, 70_000, 60_000],
                9_000,
                12_000,
            ),
            (
                "australia_new_zealand",
                [114_000, 98_000, 94_000, 92_000, 90_000, 85_000, 80_000],
                9_000,
                12_500,
            ),
            (
                "asia_pacific",
                [120_000, 100_000, 95_000, 90_000, 80_000, 70_000, 60_000],
                9_000,
                12_000,
            ),
            (
                "india",
                [109_000, 85_000, 82_000, 80_000, 78_000, 75_000, 72_000],
                9_000,
                12_000,
            ),
        ];
        assert_eq!(book.regions.len(), table.len());
        for (region, expected) in book.regions.iter().zip(table) {
            assert_eq!(region.id, expected.0);
            assert_eq!(region.data_micros_per_gb, expected.1);
            assert_eq!(region.http_micros_per_10k, expected.2);
            assert_eq!(region.https_micros_per_10k, expected.3);
        }
    }

    #[test]
    fn embedded_free_allowance() {
        let book = official();
        assert_eq!(book.bytes_per_gb, crate::money::BYTES_PER_GB);
        assert_eq!(book.free_data_bytes, book.bytes_per_tb().unwrap());
        assert_eq!(book.free_requests, 10_000_000);
    }

    #[test]
    fn rejects_price_file_with_bom() {
        let text = format!("\u{feff}{}", include_str!("../data/prices.txt"));
        let err = load_prices(&text).unwrap_err();
        assert!(err.message.contains("BOM"));
    }

    #[test]
    fn rejects_missing_source_url() {
        let text = "\
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 0.085 0.080 0.060 0.040 0.030 0.025 0.020 0.0075 0.0100
";
        let err = load_prices(text).unwrap_err();
        assert!(err.message.contains("bron-URL"));
    }

    #[test]
    fn rejects_bad_checked_on() {
        let text = "\
source_url https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/
checked_on 2026-02-29
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
";
        assert!(load_prices(text).is_err());
    }

    #[test]
    fn rejects_duplicate_region() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
region us 1 1 1 1 1 1 1 1 1
";
        assert!(load_prices(text).unwrap_err().message.contains("dubbele"));
    }

    #[test]
    fn rejects_repeated_label_for_same_region() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
label us US
label us Amerika
";
        let err = load_prices(text).unwrap_err();
        assert!(err.message.contains("twee keer"), "{}", err.message);
    }

    #[test]
    fn labels_for_different_regions_are_fine() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
region eu 1 1 1 1 1 1 1 1 1
label us US
label eu Europa
";
        let book = load_prices(text).unwrap();
        assert_eq!(book.regions[0].label, "US");
        assert_eq!(book.regions[1].label, "Europa");
    }

    #[test]
    fn rejects_short_region_row() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1
";
        assert!(load_prices(text).is_err());
    }

    #[test]
    fn rejects_zero_bytes_per_gb() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 0
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
";
        assert!(load_prices(text).is_err());
    }

    #[test]
    fn rejects_region_id_onbekend() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region onbekend 1 1 1 1 1 1 1 1 1
";
        assert!(load_prices(text).is_err());
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let text = "\
# commentaar

source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0

region us 0.085 0.080 0.060 0.040 0.030 0.025 0.020 0.0075 0.0100
";
        let book = load_prices(text).unwrap();
        assert_eq!(book.regions.len(), 1);
        assert_eq!(book.regions[0].data_micros_per_gb[0], 85_000);
    }

    #[test]
    fn rejects_unicode_whitespace_in_price_lines() {
        for bad in ['\u{a0}', '\u{3000}', '\u{2028}'] {
            for rest in [
                format!("{bad}\n"),
                format!("label us{bad}Naam\n{ROW}"),
                format!("label us Naam{bad}\n{ROW}"),
                format!("region{bad}us 1 1 1 1 1 1 1 1 1\n"),
            ] {
                let msg = price_message(&with_head(&rest));
                assert!(msg.contains("witruimteteken"), "{bad:?}: {msg}");
            }
        }
    }

    #[test]
    fn ascii_tab_and_cr_stay_valid() {
        let text = "source_url\thttps://example.test/prices\r\nchecked_on 2026-11-09 \r\nbytes_per_gb\t10\nfree_data_bytes 0\nfree_requests 0\n\tlabel\tus\tUS Noord \r\nregion\tus 1 1 1 1 1 1 1 1 1\r\n";
        let book = load_prices(text).unwrap();
        assert_eq!(book.regions[0].label, "US Noord");
    }

    #[test]
    fn label_with_spaces_is_kept() {
        let text = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
label us US/Mexico/Canada
region us 1 1 1 1 1 1 1 1 1
";
        let book = load_prices(text).unwrap();
        assert_eq!(book.regions[0].label, "US/Mexico/Canada");
    }

    const HEAD: &str = "\
source_url https://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
";
    const ROW: &str = "region us 1 1 1 1 1 1 1 1 1\n";

    fn price_message(text: &str) -> String {
        load_prices(text).unwrap_err().message
    }

    fn with_head(rest: &str) -> String {
        format!("{HEAD}{rest}")
    }

    #[test]
    fn price_file_error_texts_are_fixed() {
        assert_eq!(
            price_message(&format!("\u{feff}{HEAD}{ROW}")),
            "prijsbestand: begint met een UTF-8 BOM; sla het bestand op zonder BOM"
        );
        assert_eq!(
            price_message("source_url http://example.test/prices\n"),
            "prijsbestand:1: bron-URL moet met https:// beginnen: http://example.test/prices"
        );
        assert_eq!(
            price_message("source_url https://a.test/p\nchecked_on 2026-02-29\n"),
            "prijsbestand:2: checked_on is geen datum: 2026-02-29"
        );
        assert_eq!(
            price_message("bytes_per_gb 0\n"),
            "prijsbestand:1: bytes_per_gb moet groter zijn dan 0"
        );
        assert_eq!(
            price_message(&with_head("label us\n")),
            "prijsbestand:6: label mist een tekst"
        );
        assert_eq!(
            price_message(&with_head("label us US\nlabel us VS\n")),
            "prijsbestand:7: label voor regio us staat twee keer"
        );
        assert_eq!(
            price_message(&with_head("region us 1 1 1\n")),
            "prijsbestand:6: een regio heeft een id, 7 datatarieven, http en https"
        );
        assert_eq!(
            price_message(&with_head("region onbekend 1 1 1 1 1 1 1 1 1\n")),
            "prijsbestand:6: de regio-id onbekend is gereserveerd voor codes zonder kaart"
        );
        assert_eq!(
            price_message(&with_head(&format!("{ROW}{ROW}"))),
            "prijsbestand:7: dubbele regio us"
        );
        assert_eq!(
            price_message(&with_head("region us x 1 1 1 1 1 1 1 1\n")),
            "prijsbestand:6: ongeldig bedrag x"
        );
        assert_eq!(
            price_message(&with_head("region us 1 1 1 1 1 1 1 1 0.0000001\n")),
            "prijsbestand:6: te veel decimalen in 0.0000001"
        );
        assert_eq!(
            price_message(&with_head("frobnicate 1\n")),
            "prijsbestand:6: onbekende sleutel frobnicate"
        );
        assert_eq!(
            price_message(&with_head("region Us 1 1 1 1 1 1 1 1 1\n")),
            "prijsbestand:6: ongeldige regio-id Us"
        );
        assert_eq!(
            price_message(&with_head("label Us Tekst\n")),
            "prijsbestand:6: ongeldige regio-id Us"
        );
    }

    #[test]
    fn price_file_value_error_texts_are_fixed() {
        assert_eq!(
            price_message("source_url\n"),
            "prijsbestand:1: source_url mist een waarde"
        );
        assert_eq!(
            price_message("source_url https://a.test/p https://b.test/p\n"),
            "prijsbestand:1: source_url heeft te veel waarden"
        );
        assert_eq!(
            price_message("free_requests 1 2\n"),
            "prijsbestand:1: free_requests heeft te veel waarden"
        );
        assert_eq!(
            price_message("bytes_per_gb tien\n"),
            "prijsbestand:1: bytes_per_gb is geen geheel getal"
        );
        assert_eq!(
            price_message("free_data_bytes -1\n"),
            "prijsbestand:1: free_data_bytes is geen geheel getal"
        );
        assert_eq!(
            price_message(&format!("free_requests {}0\n", u128::MAX)),
            "prijsbestand:1: free_requests is geen geheel getal"
        );
        assert_eq!(
            price_message("source_url https://a.test/p\nsource_url https://a.test/p\n"),
            "prijsbestand:2: dubbele sleutel source_url"
        );
        assert_eq!(
            price_message(&with_head("checked_on 2026-11-10\n")),
            "prijsbestand:6: dubbele sleutel checked_on"
        );
        assert_eq!(
            price_message(&with_head("bytes_per_gb 11\n")),
            "prijsbestand:6: dubbele sleutel bytes_per_gb"
        );
        assert_eq!(
            price_message(&with_head("free_data_bytes 1\n")),
            "prijsbestand:6: dubbele sleutel free_data_bytes"
        );
        assert_eq!(
            price_message(&with_head("free_requests 1\n")),
            "prijsbestand:6: dubbele sleutel free_requests"
        );
    }

    #[test]
    fn price_file_missing_part_texts_are_fixed() {
        let lines: Vec<&str> = HEAD.lines().collect();
        let expected = [
            "prijsbestand: bron-URL ontbreekt",
            "prijsbestand: checked_on ontbreekt",
            "prijsbestand: bytes_per_gb ontbreekt",
            "prijsbestand: free_data_bytes ontbreekt",
            "prijsbestand: free_requests ontbreekt",
        ];
        for (skip, message) in expected.iter().enumerate() {
            let mut text = String::new();
            for (index, line) in lines.iter().enumerate() {
                if index != skip {
                    text.push_str(line);
                    text.push('\n');
                }
            }
            text.push_str(ROW);
            assert_eq!(price_message(&text), *message);
        }
        assert_eq!(price_message(HEAD), "prijsbestand: geen regio");
        assert_eq!(
            price_message(&with_head(&format!("{ROW}label eu Europa\n"))),
            "prijsbestand: label voor onbekende regio eu"
        );
    }

    #[test]
    fn region_id_check_texts_are_fixed() {
        assert_eq!(check_id("").unwrap_err(), "lege regio-id");
        assert_eq!(check_id("9x").unwrap_err(), "ongeldige regio-id 9x");
        assert_eq!(check_id("a-b").unwrap_err(), "ongeldige regio-id a-b");
        assert!(check_id("a_b9").is_ok());
    }

    #[test]
    fn one_terabyte_overflow_text_is_fixed() {
        let text = format!(
            "source_url https://a.test/p\nchecked_on 2026-11-09\nbytes_per_gb {}\nfree_data_bytes 0\nfree_requests 0\n{ROW}",
            u128::MAX
        );
        let book = load_prices(&text).unwrap();
        assert_eq!(
            book.bytes_per_tb().unwrap_err().message,
            "overflow in de definitie van 1 TB"
        );
    }

    #[test]
    fn rejects_http_source_url() {
        let text = "\
source_url http://example.test/prices
checked_on 2026-11-09
bytes_per_gb 10
free_data_bytes 0
free_requests 0
region us 1 1 1 1 1 1 1 1 1
";
        assert!(load_prices(text).is_err());
    }
}
