use std::collections::BTreeMap;

use crate::error::Error;
use crate::price::PriceBook;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeMap {
    codes: BTreeMap<String, String>,
}

impl EdgeMap {
    pub fn region_id(&self, location: &str) -> Option<&str> {
        let code = iata_code(location)?;
        self.codes.get(&code).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.codes
            .iter()
            .map(|(code, region)| (code.as_str(), region.as_str()))
    }

    pub fn len(&self) -> usize {
        self.codes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.codes.is_empty()
    }

    pub fn check_regions(&self, prices: &PriceBook) -> Result<(), Error> {
        let known = prices.index_map();
        for (code, region) in &self.codes {
            if !known.contains_key(region.as_str()) {
                return Err(Error::new(format!(
                    "edge {code} hoort bij onbekende prijsregio {region}"
                )));
            }
        }
        Ok(())
    }
}

/// Eerste drie letters van een CloudFront-edge, bijvoorbeeld `AMS50-C1` → `AMS`.
pub fn iata_code(location: &str) -> Option<String> {
    let bytes = location.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    if !bytes[0].is_ascii_alphabetic()
        || !bytes[1].is_ascii_alphabetic()
        || !bytes[2].is_ascii_alphabetic()
    {
        return None;
    }
    let mut index = 3;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index < bytes.len() {
        if bytes[index] != b'-' {
            return None;
        }
        index += 1;
        if index == bytes.len() {
            return None;
        }
        while index < bytes.len() {
            if !bytes[index].is_ascii_alphanumeric() {
                return None;
            }
            index += 1;
        }
    }
    Some(location[..3].to_ascii_uppercase())
}

fn is_sep(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r')
}

pub fn load_edges(text: &str) -> Result<EdgeMap, Error> {
    let mut codes = BTreeMap::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line_no = lineno + 1;
        let line = raw.trim_matches(is_sep);
        if line.starts_with('#') {
            continue;
        }
        if let Some(bad) = line.chars().find(|c| c.is_whitespace() && !is_sep(*c)) {
            return Err(Error::new(format!(
                "edge-kaart:{line_no}: ongeldig witruimteteken U+{:04X}; gebruik alleen spaties of tabs",
                bad as u32
            )));
        }
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(is_sep).filter(|part| !part.is_empty());
        let code = parts
            .next()
            .ok_or_else(|| Error::new(format!("edge-kaart:{line_no}: lege regel")))?;
        let region = parts.next().ok_or_else(|| {
            Error::new(format!("edge-kaart:{line_no}: code {code} mist een regio"))
        })?;
        if parts.next().is_some() {
            return Err(Error::new(format!(
                "edge-kaart:{line_no}: te veel kolommen"
            )));
        }
        if code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Err(Error::new(format!(
                "edge-kaart:{line_no}: {code} is geen IATA-code van drie letters"
            )));
        }
        let code = code.to_ascii_uppercase();
        if !region
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase())
            || !region
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        {
            return Err(Error::new(format!(
                "edge-kaart:{line_no}: ongeldige regio {region}"
            )));
        }
        if codes.insert(code.clone(), region.to_string()).is_some() {
            return Err(Error::new(format!(
                "edge-kaart:{line_no}: dubbele code {code}"
            )));
        }
    }
    if codes.is_empty() {
        return Err(Error::new("edge-kaart: geen codes"));
    }
    Ok(EdgeMap { codes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn official() -> EdgeMap {
        load_edges(include_str!("../data/edges.txt")).unwrap()
    }

    #[test]
    fn ams_pop_is_europe() {
        assert_eq!(
            official().region_id("AMS50-C1"),
            Some("europe_israel_turkiye")
        );
    }

    #[test]
    fn fra_pop_is_europe() {
        assert_eq!(
            official().region_id("FRA6-C1"),
            Some("europe_israel_turkiye")
        );
    }

    #[test]
    fn iad_pop_is_us() {
        assert_eq!(official().region_id("IAD89-P1"), Some("us_mexico_canada"));
    }

    #[test]
    fn lax_without_suffix_is_us() {
        assert_eq!(official().region_id("LAX1"), Some("us_mexico_canada"));
        assert_eq!(official().region_id("LAX"), Some("us_mexico_canada"));
    }

    #[test]
    fn nrt_is_japan() {
        assert_eq!(official().region_id("NRT12"), Some("japan"));
        assert_eq!(official().region_id("KIX1-C1"), Some("japan"));
    }

    #[test]
    fn lowercase_edge_is_accepted() {
        assert_eq!(
            official().region_id("ams50-c1"),
            Some("europe_israel_turkiye")
        );
    }

    #[test]
    fn unknown_iata_is_onbekend() {
        assert_eq!(official().region_id("ZZZ1-C1"), None);
        assert_eq!(official().region_id("QQQ"), None);
    }

    #[test]
    fn malformed_location_is_onbekend() {
        assert_eq!(iata_code(""), None);
        assert_eq!(iata_code("AB"), None);
        assert_eq!(iata_code("AMS-"), None);
        assert_eq!(iata_code("AMS50-"), None);
        assert_eq!(iata_code("12ABC"), None);
        assert_eq!(official().region_id("???"), None);
    }

    #[test]
    fn china_airports_are_onbekend() {
        let edges = official();
        for code in ["PEK50-C1", "PVG1", "SHA", "BJS", "CAN", "SZX", "CTU", "HGH"] {
            assert_eq!(edges.region_id(code), None, "{code}");
        }
    }

    #[test]
    fn tlv_and_ist_are_europe_not_the_middle_east() {
        let edges = official();
        assert_eq!(edges.region_id("TLV50-C1"), Some("europe_israel_turkiye"));
        assert_eq!(edges.region_id("IST1"), Some("europe_israel_turkiye"));
        assert_ne!(edges.region_id("TLV"), Some("africa_middle_east"));
    }

    #[test]
    fn dxb_and_jnb_are_africa_middle_east() {
        let edges = official();
        assert_eq!(edges.region_id("DXB1-C1"), Some("africa_middle_east"));
        assert_eq!(edges.region_id("JNB50"), Some("africa_middle_east"));
        assert_eq!(edges.region_id("CPT1"), Some("africa_middle_east"));
    }

    #[test]
    fn gru_bom_sin_syd_map_to_their_columns() {
        let edges = official();
        assert_eq!(edges.region_id("GRU1"), Some("south_america"));
        assert_eq!(edges.region_id("BOM2"), Some("india"));
        assert_eq!(edges.region_id("SIN3-P1"), Some("asia_pacific"));
        assert_eq!(edges.region_id("SYD1-C1"), Some("australia_new_zealand"));
    }

    #[test]
    fn embedded_edges_are_unique_and_priced() {
        let edges = official();
        let groups: &[(&str, &[&str])] = &[
            (
                "us_mexico_canada",
                &[
                    "ATL", "BOS", "ORD", "DFW", "DEN", "HIO", "IAH", "JAX", "LAX", "MIA", "MSP",
                    "JFK", "EWR", "PHL", "PHX", "SLC", "SFO", "SEA", "IAD", "DTW", "BNA", "IND",
                    "MCI", "PIT", "CMH", "CLT", "YTO", "YUL", "YVR", "QRO",
                ],
            ),
            (
                "europe_israel_turkiye",
                &[
                    "AMS", "ARN", "ATH", "BER", "BCN", "BRU", "BUD", "CDG", "CPH", "DUB", "DUS",
                    "FCO", "FRA", "HEL", "IST", "LIS", "LHR", "MAD", "MAN", "MRS", "MUC", "MXP",
                    "OSL", "OTP", "PRG", "SOF", "TLV", "TXL", "VIE", "WAW", "ZRH",
                ],
            ),
            (
                "africa_middle_east",
                &["CPT", "JNB", "NBO", "LOS", "DXB", "FJR", "BAH", "MCT"],
            ),
            (
                "south_america",
                &["GRU", "GIG", "FOR", "BOG", "EZE", "SCL", "LIM"],
            ),
            ("japan", &["NRT", "KIX"]),
            (
                "australia_new_zealand",
                &["SYD", "MEL", "PER", "BNE", "AKL"],
            ),
            (
                "asia_pacific",
                &[
                    "HKG", "CGK", "MNL", "SIN", "ICN", "TPE", "BKK", "KUL", "SGN", "HAN",
                ],
            ),
            ("india", &["BOM", "DEL", "MAA", "BLR", "HYD", "CCU"]),
        ];
        let mut seen = 0usize;
        for &(region, codes) in groups {
            for code in codes {
                assert_eq!(edges.region_id(code), Some(region), "{code}");
                seen += 1;
            }
        }
        assert_eq!(edges.len(), seen);
        let prices = crate::default_prices().unwrap();
        edges.check_regions(&prices).unwrap();
        crate::default_edges().unwrap();
    }

    #[test]
    fn duplicate_edge_code_is_rejected() {
        let text = "AMS europe_israel_turkiye\nams us_mexico_canada\n";
        assert!(load_edges(text).unwrap_err().message.contains("dubbele"));
    }

    #[test]
    fn edge_code_in_the_map_must_be_three_letters() {
        let text = "AMS50 europe_israel_turkiye\n";
        assert!(load_edges(text).is_err());
    }

    fn edge_message(text: &str) -> String {
        load_edges(text).unwrap_err().message
    }

    #[test]
    fn edge_map_error_texts_are_fixed() {
        assert_eq!(
            edge_message("AMS\n"),
            "edge-kaart:1: code AMS mist een regio"
        );
        assert_eq!(
            edge_message("# kop\n\nAMS europe extra\n"),
            "edge-kaart:3: te veel kolommen"
        );
        assert_eq!(
            edge_message("AMS50 europe\n"),
            "edge-kaart:1: AMS50 is geen IATA-code van drie letters"
        );
        assert_eq!(
            edge_message("A1S europe\n"),
            "edge-kaart:1: A1S is geen IATA-code van drie letters"
        );
        assert_eq!(
            edge_message("AMS Europe\n"),
            "edge-kaart:1: ongeldige regio Europe"
        );
        assert_eq!(
            edge_message("AMS 1europe\n"),
            "edge-kaart:1: ongeldige regio 1europe"
        );
        assert_eq!(
            edge_message("AMS europe\nams europe\n"),
            "edge-kaart:2: dubbele code AMS"
        );
        assert_eq!(edge_message(""), "edge-kaart: geen codes");
        assert_eq!(
            edge_message("# alleen commentaar\n\n"),
            "edge-kaart: geen codes"
        );
    }

    #[test]
    fn edge_with_unknown_price_region_error_text_is_fixed() {
        let edges = load_edges("AMS no_such_region\n").unwrap();
        let err = edges.check_regions(&crate::default_prices().unwrap());
        assert_eq!(
            err.unwrap_err().message,
            "edge AMS hoort bij onbekende prijsregio no_such_region"
        );
    }

    #[test]
    fn edge_file_says_unknown_is_not_a_guess() {
        let text = include_str!("../data/edges.txt");
        assert!(text.contains("niet het land van de kijker"));
        assert!(text.contains("niet geraden"));
        assert!(text.contains("China"));
    }

    #[test]
    fn rejects_unicode_whitespace_in_edge_lines() {
        for bad in ['\u{a0}', '\u{3000}', '\u{2028}'] {
            for text in [
                format!("{bad}\n"),
                format!("AMS{bad}eu\n"),
                format!("AMS eu{bad}\n"),
                format!("{bad}AMS eu\n"),
            ] {
                let msg = load_edges(&text).unwrap_err().to_string();
                assert!(msg.contains("witruimteteken"), "{bad:?}: {msg}");
            }
        }
    }

    #[test]
    fn ascii_tab_and_cr_stay_valid_in_edge_lines() {
        let map = load_edges("\tAMS\teu \r\nFRA eu\r\n").unwrap();
        assert_eq!(map.codes.len(), 2);
    }
}
