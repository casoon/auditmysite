//! Vocabulary, entity-identity, geo-plausibility and `sameAs` checks for
//! structured data (#1271).
//!
//! Pure offline functions operating on parsed [`JsonLdSchema`] nodes:
//! - Vocabulary validation via [`web_checks::structured_data::validate_vocabulary`]
//!   (unknown `@type`, unknown properties, domain mismatches, range violations).
//! - Single-page `@id` hint on organization/local-business entities.
//! - Offline DACH (DE/AT/CH) geo-coordinate plausibility against `addressCountry`
//!   and 2-digit postal code regions (`postalCode`).
//! - `sameAs` authority-profile hints (self-domain-only, missing business
//!   profile for `LocalBusiness`, missing knowledge-graph/registry profile for
//!   `Organization`).

use serde_json::Value;
use web_checks::structured_data::{is_subtype_of, validate_vocabulary, VocabIssue};

use crate::seo::schema::{JsonLdSchema, SchemaIssue, SchemaIssueSeverity, SchemaType};

/// Collect vocabulary, identity, geo-plausibility and `sameAs` issues for all
/// valid JSON-LD nodes on a page.
pub(crate) fn collect_plausibility_issues(schemas: &[JsonLdSchema]) -> Vec<SchemaIssue> {
    let mut issues = Vec::new();
    for schema in schemas.iter().filter(|s| s.is_valid) {
        let primary_type = schema
            .schema_types
            .first()
            .cloned()
            .unwrap_or_else(|| "JSON-LD".to_string());

        // 1. schema.org vocabulary validation (web-checks 0.10)
        for schema_type in &schema.schema_types {
            for vocab_issue in validate_vocabulary(schema_type, &schema.content) {
                issues.push(vocab_issue_to_schema_issue(schema_type, vocab_issue));
            }
        }

        let is_local_business = schema.schema_types.iter().any(|t| {
            is_subtype_of(t, "LocalBusiness")
                || t.parse::<SchemaType>()
                    .is_ok_and(|st| st.is_local_business_like())
        });
        let is_org = is_local_business
            || schema.schema_types.iter().any(|t| {
                is_subtype_of(t, "Organization")
                    || t.parse::<SchemaType>()
                        .is_ok_and(|st| st.is_organization_like())
            });

        // 2. Single-page organization without @id
        if is_org {
            let has_id = schema
                .content
                .get("@id")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty());
            if !has_id {
                issues.push(SchemaIssue {
                    schema_type: primary_type.clone(),
                    severity: SchemaIssueSeverity::Recommended,
                    issue_type: "schema_org_missing_id".to_string(),
                    message: format!(
                        "{primary_type} has no @id — without a stable @id, search engines treat repeated declarations across pages as separate entities."
                    ),
                });
            }
        }

        // 3. Geo ↔ address plausibility (offline DACH PLZ & country bounds)
        if let Some(geo_issue) = check_geo_plausibility(&primary_type, &schema.content) {
            issues.push(geo_issue);
        }

        // 4. sameAs authority profile hints
        if is_org {
            if let Some(same_as_issue) =
                check_same_as(&primary_type, &schema.content, is_local_business)
            {
                issues.push(same_as_issue);
            }
        }
    }
    issues
}

fn vocab_issue_to_schema_issue(primary_type: &str, issue: VocabIssue) -> SchemaIssue {
    match issue {
        VocabIssue::UnknownType {
            schema_type,
            suggestion,
        } => {
            let hint = suggestion
                .map(|s| format!(" (did you mean \"{s}\"?)"))
                .unwrap_or_default();
            SchemaIssue {
                schema_type: schema_type.clone(),
                severity: SchemaIssueSeverity::Error,
                issue_type: "schema_vocab_unknown_type".to_string(),
                message: format!(
                    "Schema type \"{schema_type}\" does not exist in schema.org{hint} — search engines ignore or misclassify the node."
                ),
            }
        }
        VocabIssue::UnknownProperty {
            property,
            suggestion,
        } => {
            let hint = suggestion
                .map(|s| format!(" (did you mean \"{s}\"?)"))
                .unwrap_or_default();
            SchemaIssue {
                schema_type: primary_type.to_string(),
                severity: SchemaIssueSeverity::Error,
                issue_type: "schema_vocab_unknown_property".to_string(),
                message: format!(
                    "{primary_type}: property \"{property}\" is not part of schema.org{hint}."
                ),
            }
        }
        VocabIssue::InvalidPropertyDomain {
            property,
            schema_type,
            expected_domain,
        } => SchemaIssue {
            schema_type: schema_type.clone(),
            severity: SchemaIssueSeverity::Error,
            issue_type: "schema_vocab_invalid_domain".to_string(),
            message: format!(
                "{schema_type}: property \"{property}\" belongs to {expected_domain}, not {schema_type}."
            ),
        },
        VocabIssue::InvalidRange { property, expected } => SchemaIssue {
            schema_type: primary_type.to_string(),
            severity: SchemaIssueSeverity::Required,
            issue_type: "schema_vocab_invalid_range".to_string(),
            message: format!(
                "{primary_type}: property \"{property}\" has an invalid value type (expected {expected})."
            ),
        },
    }
}

// ─── Geo ↔ Address Plausibility ──────────────────────────────────────────────

fn parse_coord(val: &Value) -> Option<f64> {
    if let Some(n) = val.as_f64() {
        return n.is_finite().then_some(n);
    }
    if let Some(s) = val.as_str() {
        let normalized = s.trim().replace(',', ".");
        return normalized.parse::<f64>().ok().filter(|n| n.is_finite());
    }
    None
}

fn normalize_country(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DE" | "DEU" | "GERMANY" | "DEUTSCHLAND" => Some("DE"),
        "AT" | "AUT" | "AUSTRIA" | "ÖSTERREICH" | "OESTERREICH" => Some("AT"),
        "CH" | "CHE" | "SWITZERLAND" | "SCHWEIZ" | "SUISSE" | "SVIZZERA" => Some("CH"),
        _ => None,
    }
}

fn country_bbox(country: &str) -> Option<(f64, f64, f64, f64)> {
    // (min_lat, max_lat, min_lon, max_lon)
    match country {
        "DE" => Some((47.2, 55.1, 5.8, 15.1)),
        "AT" => Some((46.3, 49.1, 9.5, 17.2)),
        "CH" => Some((45.8, 47.9, 5.9, 10.6)),
        _ => None,
    }
}

fn check_geo_plausibility(primary_type: &str, content: &Value) -> Option<SchemaIssue> {
    let geo = content.get("geo")?;
    let geo_obj = geo.as_object()?;

    let lat = geo_obj.get("latitude").and_then(parse_coord);
    let lon = geo_obj.get("longitude").and_then(parse_coord);
    let (lat, lon) = match (lat, lon) {
        (Some(la), Some(lo)) if (-90.0..=90.0).contains(&la) && (-180.0..=180.0).contains(&lo) => {
            (la, lo)
        }
        _ => {
            return Some(SchemaIssue {
                schema_type: primary_type.to_string(),
                severity: SchemaIssueSeverity::Required,
                issue_type: "schema_geo_range_invalid".to_string(),
                message: format!(
                    "{primary_type}: geo coordinates are missing or outside valid latitude/longitude bounds."
                ),
            });
        }
    };

    let address = content.get("address").and_then(Value::as_object)?;
    let country = address
        .get("addressCountry")
        .and_then(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
        .and_then(normalize_country);

    if let Some(code) = country {
        if let Some((min_lat, max_lat, min_lon, max_lon)) = country_bbox(code) {
            if !(min_lat..=max_lat).contains(&lat) || !(min_lon..=max_lon).contains(&lon) {
                return Some(SchemaIssue {
                    schema_type: primary_type.to_string(),
                    severity: SchemaIssueSeverity::Error,
                    issue_type: "schema_geo_country_mismatch".to_string(),
                    message: format!(
                        "{primary_type}: geo coordinates ({lat:.4}, {lon:.4}) lie outside the declared addressCountry ({code})."
                    ),
                });
            }
        }
    }

    let postal_raw = address
        .get("postalCode")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let digits: String = postal_raw.chars().filter(char::is_ascii_digit).collect();

    // Infer country from 5-digit DE PLZ if addressCountry was omitted
    let effective_country = country.or_else(|| (digits.len() == 5).then_some("DE"));
    if let (Some(code), Some((ref_lat, ref_lon, max_km))) = (
        effective_country,
        effective_country.and_then(|c| postal_region_centroid(c, &digits)),
    ) {
        let dist_km = haversine_km(lat, lon, ref_lat, ref_lon);
        if dist_km > max_km {
            return Some(SchemaIssue {
                schema_type: primary_type.to_string(),
                severity: SchemaIssueSeverity::Required,
                issue_type: "schema_geo_address_mismatch".to_string(),
                message: format!(
                    "{primary_type}: geo coordinates ({lat:.4}, {lon:.4}) are ~{:.0} km away from postal code region {code}-{postal_raw}.",
                    dist_km
                ),
            });
        }
    }

    None
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

/// Offline centroid `(lat, lon, max_plausible_km)` for DACH 2-digit postal
/// code regions. Conservative radii avoid false positives for large rural
/// regions while reliably catching misplaced city/state coordinates.
fn postal_region_centroid(country: &str, digits: &str) -> Option<(f64, f64, f64)> {
    if digits.len() < 2 {
        return None;
    }
    let prefix = &digits[..2];
    match country {
        "DE" if digits.len() == 5 => de_plz2_centroid(prefix).map(|(la, lo)| (la, lo, 75.0)),
        "AT" if digits.len() == 4 => at_plz1_centroid(&digits[..1]).map(|(la, lo)| (la, lo, 95.0)),
        "CH" if digits.len() == 4 => ch_plz1_centroid(&digits[..1]).map(|(la, lo)| (la, lo, 85.0)),
        _ => None,
    }
}

fn at_plz1_centroid(prefix: &str) -> Option<(f64, f64)> {
    match prefix {
        "1" => Some((48.21, 16.37)), // Wien
        "2" => Some((48.10, 16.40)), // Niederösterreich Ost/Süd
        "3" => Some((48.20, 15.63)), // Niederösterreich West
        "4" => Some((48.30, 14.28)), // Oberösterreich
        "5" => Some((47.80, 13.04)), // Salzburg
        "6" => Some((47.27, 11.39)), // Tirol / Vorarlberg
        "7" => Some((47.50, 16.40)), // Burgenland
        "8" => Some((47.07, 15.44)), // Steiermark
        "9" => Some((46.62, 14.31)), // Kärnten
        _ => None,
    }
}

fn ch_plz1_centroid(prefix: &str) -> Option<(f64, f64)> {
    match prefix {
        "1" => Some((46.52, 6.63)), // Westschweiz (Genf / Lausanne)
        "2" => Some((47.05, 7.05)), // Neuenburg / Jura / Biel
        "3" => Some((46.95, 7.45)), // Bern / Oberland / Wallis
        "4" => Some((47.55, 7.59)), // Basel / Solothurn / Aargau West
        "5" => Some((47.39, 8.05)), // Aargau
        "6" => Some((47.05, 8.31)), // Zentralschweiz / Tessin
        "7" => Some((46.85, 9.53)), // Graubünden
        "8" => Some((47.37, 8.54)), // Zürich / Thurgau / Schaffhausen
        "9" => Some((47.42, 9.37)), // Ostschweiz (St. Gallen)
        _ => None,
    }
}

/// Approximate geographic centroids for German 2-digit PLZ Leitregionen.
fn de_plz2_centroid(prefix: &str) -> Option<(f64, f64)> {
    let coords = match prefix {
        "01" => (51.05, 13.74),               // Dresden
        "02" => (51.18, 14.43),               // Bautzen / Görlitz
        "03" => (51.76, 14.33),               // Cottbus
        "04" => (51.34, 12.37),               // Leipzig
        "06" => (51.48, 11.97),               // Halle / Dessau
        "07" => (50.88, 11.59),               // Gera / Jena
        "08" => (50.72, 12.49),               // Zwickau / Plauen
        "09" => (50.83, 12.92),               // Chemnitz
        "10" | "12" | "13" => (52.52, 13.40), // Berlin
        "14" => (52.40, 13.06),               // Potsdam / Brandenburg
        "15" => (52.35, 14.05),               // Frankfurt (Oder)
        "16" => (52.85, 13.45),               // Oranienburg / Eberswalde
        "17" => (53.85, 13.38),               // Neubrandenburg / Greifswald
        "18" => (54.09, 12.14),               // Rostock / Stralsund
        "19" => (53.63, 11.41),               // Schwerin / Wismar
        "20" | "22" => (53.55, 9.99),         // Hamburg
        "21" => (53.45, 9.98),                // Harburg / Lüneburg / Stade
        "23" => (53.87, 10.69),               // Lübeck
        "24" => (54.32, 10.13),               // Kiel / Flensburg
        "25" => (53.95, 9.45),                // Itzehoe / Husum / Heide
        "26" => (53.14, 8.21),                // Oldenburg / Wilhelmshaven
        "27" => (53.30, 8.85),                // Bremerhaven / Cuxhaven
        "28" => (53.08, 8.80),                // Bremen
        "29" => (52.62, 10.08),               // Celle / Uelzen
        "30" => (52.37, 9.73),                // Hannover
        "31" => (52.15, 9.75),                // Hildesheim / Hameln
        "32" => (52.12, 8.67),                // Herford / Minden / Detmold
        "33" => (52.02, 8.53),                // Bielefeld / Paderborn
        "34" => (51.31, 9.49),                // Kassel
        "35" => (50.59, 8.68),                // Gießen / Marburg / Wetzlar
        "36" => (50.55, 9.68),                // Fulda
        "37" => (51.53, 9.93),                // Göttingen
        "38" => (52.26, 10.52),               // Braunschweig / Wolfsburg
        "39" => (52.13, 11.63),               // Magdeburg
        "40" => (51.23, 6.78),                // Düsseldorf
        "41" => (51.19, 6.44),                // Mönchengladbach / Neuss
        "42" => (51.26, 7.15),                // Wuppertal / Solingen
        "44" => (51.51, 7.46),                // Dortmund / Bochum
        "45" => (51.45, 7.01),                // Essen / Gelsenkirchen
        "46" => (51.55, 6.85),                // Oberhausen / Bottrop
        "47" => (51.43, 6.76),                // Duisburg / Krefeld
        "48" => (51.96, 7.63),                // Münster / Rheine
        "49" => (52.28, 8.05),                // Osnabrück
        "50" => (50.94, 6.96),                // Köln
        "51" => (50.99, 7.13),                // Leverkusen / Bergisch Gladbach
        "52" => (50.78, 6.08),                // Aachen / Düren
        "53" => (50.73, 7.10),                // Bonn
        "54" => (49.75, 6.64),                // Trier
        "55" => (49.99, 8.25),                // Mainz / Bad Kreuznach
        "56" => (50.36, 7.59),                // Koblenz
        "57" => (50.87, 8.02),                // Siegen
        "58" => (51.36, 7.47),                // Hagen / Iserlohn
        "59" => (51.68, 7.82),                // Hamm / Soest / Arnsberg
        "60" => (50.11, 8.68),                // Frankfurt am Main
        "61" => (50.23, 8.62),                // Bad Homburg / Friedberg
        "63" => (50.10, 9.00),                // Offenbach / Hanau / Aschaffenburg
        "64" => (49.87, 8.65),                // Darmstadt
        "65" => (50.08, 8.24),                // Wiesbaden
        "66" => (49.24, 7.00),                // Saarbrücken
        "67" => (49.48, 8.15),                // Ludwigshafen / Kaiserslautern
        "68" => (49.49, 8.47),                // Mannheim
        "69" => (49.41, 8.69),                // Heidelberg
        "70" => (48.78, 9.18),                // Stuttgart
        "71" => (48.88, 9.18),                // Ludwigsburg / Böblingen
        "72" => (48.52, 9.05),                // Tübingen / Reutlingen
        "73" => (48.70, 9.65),                // Göppingen / Esslingen / Aalen
        "74" => (49.14, 9.22),                // Heilbronn
        "75" => (48.89, 8.70),                // Pforzheim
        "76" => (49.01, 8.40),                // Karlsruhe / Baden-Baden
        "77" => (48.47, 7.94),                // Offenburg
        "78" => (47.90, 8.80),                // Konstanz / Villingen-Schwenningen
        "79" => (47.99, 7.85),                // Freiburg
        "80" | "81" => (48.14, 11.58),        // München
        "82" => (48.05, 11.35),               // Starnberg / Garmisch / Fürstenfeldbruck
        "83" => (47.86, 12.13),               // Rosenheim / Traunstein
        "84" => (48.54, 12.15),               // Landshut
        "85" => (48.40, 11.60),               // Ingolstadt / Freising
        "86" => (48.37, 10.90),               // Augsburg
        "87" => (47.73, 10.31),               // Kempten / Memmingen
        "88" => (47.78, 9.61),                // Ravensburg / Friedrichshafen
        "89" => (48.40, 9.99),                // Ulm
        "90" => (49.45, 11.08),               // Nürnberg / Fürth
        "91" => (49.59, 11.00),               // Erlangen / Ansbach
        "92" => (49.44, 11.86),               // Amberg / Weiden
        "93" => (49.01, 12.10),               // Regensburg
        "94" => (48.57, 13.45),               // Passau / Straubing
        "95" => (49.95, 11.58),               // Bayreuth / Hof
        "96" => (49.89, 10.89),               // Bamberg / Coburg
        "97" => (49.79, 9.93),                // Würzburg / Schweinfurt
        "98" => (50.61, 10.69),               // Suhl
        "99" => (50.98, 11.03),               // Erfurt / Weimar
        _ => return None,
    };
    Some(coords)
}

// ─── sameAs Authority Profile Hints ──────────────────────────────────────────

fn extract_same_as(content: &Value) -> Vec<&str> {
    match content.get("sameAs") {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![trimmed]
            }
        }
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn normalized_host(raw_url: &str) -> Option<String> {
    url::Url::parse(raw_url).ok().and_then(|u| {
        u.host_str()
            .map(|h| h.trim_start_matches("www.").to_ascii_lowercase())
    })
}

fn is_business_profile_url(url_lower: &str) -> bool {
    [
        "google.com/maps",
        "maps.google.",
        "g.page/",
        "maps.app.goo.gl/",
        "business.google.com",
        "maps.apple.com",
        "bing.com/maps",
        "yelp.",
        "gelbeseiten.",
        "dasoertliche.",
        "11880.",
        "provenexpert.",
        "trustpilot.",
        "kununu.",
    ]
    .iter()
    .any(|needle| url_lower.contains(needle))
}

fn is_authority_profile_url(url_lower: &str) -> bool {
    is_business_profile_url(url_lower)
        || [
            "wikidata.org/",
            "wikipedia.org/",
            "handelsregister.de",
            "unternehmensregister.de",
            "northdata.",
            "crunchbase.com/",
            "opencorporates.com/",
            "linkedin.com/company/",
        ]
        .iter()
        .any(|needle| url_lower.contains(needle))
}

fn check_same_as(
    primary_type: &str,
    content: &Value,
    is_local_business: bool,
) -> Option<SchemaIssue> {
    let urls = extract_same_as(content);
    if urls.is_empty() {
        return None;
    }

    let own_host = content
        .get("url")
        .and_then(Value::as_str)
        .and_then(normalized_host);

    if let Some(ref own) = own_host {
        let all_self = urls
            .iter()
            .all(|u| normalized_host(u).as_deref() == Some(own.as_str()));
        if all_self {
            return Some(SchemaIssue {
                schema_type: primary_type.to_string(),
                severity: SchemaIssueSeverity::Recommended,
                issue_type: "schema_same_as_self_only".to_string(),
                message: format!(
                    "{primary_type}: sameAs only links to the site's own domain ({own}) — add external authority or business profiles to connect the entity."
                ),
            });
        }
    }

    let lowered: Vec<String> = urls.iter().map(|u| u.to_ascii_lowercase()).collect();
    if is_local_business && !lowered.iter().any(|u| is_business_profile_url(u)) {
        return Some(SchemaIssue {
            schema_type: primary_type.to_string(),
            severity: SchemaIssueSeverity::Recommended,
            issue_type: "schema_same_as_missing_business_profile".to_string(),
            message: format!(
                "{primary_type}: sameAs does not reference a search-engine business profile (e.g. Google Business Profile / Maps) to connect the local entity to the Knowledge Graph."
            ),
        });
    }

    if !is_local_business && !lowered.iter().any(|u| is_authority_profile_url(u)) {
        return Some(SchemaIssue {
            schema_type: primary_type.to_string(),
            severity: SchemaIssueSeverity::Recommended,
            issue_type: "schema_same_as_missing_authority".to_string(),
            message: format!(
                "{primary_type}: sameAs does not reference an authority profile (e.g. Wikidata, commercial register, or official company profile)."
            ),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(schema_type: &str, content: Value) -> JsonLdSchema {
        JsonLdSchema {
            schema_type: schema_type.to_string(),
            schema_types: vec![schema_type.to_string()],
            content,
            is_valid: true,
        }
    }

    #[test]
    fn flags_unknown_schema_type_web_design_company() {
        let issues = collect_plausibility_issues(&[node(
            "WebDesignCompany",
            json!({
                "@context": "https://schema.org",
                "@type": "WebDesignCompany",
                "name": "CASOON"
            }),
        )]);
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "schema_vocab_unknown_type"
                && i.severity == SchemaIssueSeverity::Error));
    }

    #[test]
    fn flags_geo_country_and_postal_code_mismatches() {
        // Coordinates in Paris for a German address -> High country mismatch
        let paris_de = collect_plausibility_issues(&[node(
            "LocalBusiness",
            json!({
                "@type": "LocalBusiness",
                "@id": "https://example.de/#org",
                "name": "Test GmbH",
                "address": {
                    "@type": "PostalAddress",
                    "postalCode": "18055",
                    "addressLocality": "Rostock",
                    "addressCountry": "DE"
                },
                "geo": { "@type": "GeoCoordinates", "latitude": 48.8566, "longitude": 2.3522 }
            }),
        )]);
        assert!(paris_de
            .iter()
            .any(|i| i.issue_type == "schema_geo_country_mismatch"
                && i.severity == SchemaIssueSeverity::Error));

        // Coordinates in Munich for a Rostock (18055) address -> Medium PLZ mismatch
        let munich_rostock = collect_plausibility_issues(&[node(
            "LocalBusiness",
            json!({
                "@type": "LocalBusiness",
                "@id": "https://example.de/#org",
                "name": "Test GmbH",
                "address": {
                    "@type": "PostalAddress",
                    "postalCode": "18055",
                    "addressLocality": "Rostock",
                    "addressCountry": "DE"
                },
                "geo": { "@type": "GeoCoordinates", "latitude": 48.1351, "longitude": 11.5820 }
            }),
        )]);
        assert!(munich_rostock
            .iter()
            .any(|i| i.issue_type == "schema_geo_address_mismatch"
                && i.severity == SchemaIssueSeverity::Required));

        // Valid Rostock coordinates for 18055 -> no geo issue
        let valid_rostock = collect_plausibility_issues(&[node(
            "LocalBusiness",
            json!({
                "@type": "LocalBusiness",
                "@id": "https://example.de/#org",
                "name": "Test GmbH",
                "address": {
                    "@type": "PostalAddress",
                    "postalCode": "18055",
                    "addressLocality": "Rostock",
                    "addressCountry": "DE"
                },
                "geo": { "@type": "GeoCoordinates", "latitude": 54.0924, "longitude": 12.0991 }
            }),
        )]);
        assert!(!valid_rostock
            .iter()
            .any(|i| i.issue_type.starts_with("schema_geo_")));
    }

    #[test]
    fn flags_same_as_without_business_profile_and_missing_id() {
        let issues = collect_plausibility_issues(&[node(
            "ProfessionalService",
            json!({
                "@type": "ProfessionalService",
                "name": "CASOON",
                "url": "https://casoon.de/",
                "sameAs": ["https://github.com/casoon", "https://www.linkedin.com/in/jseidel"]
            }),
        )]);
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "schema_org_missing_id"));
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "schema_same_as_missing_business_profile"));
    }

    #[test]
    fn accepts_local_business_with_google_maps_same_as() {
        let issues = collect_plausibility_issues(&[node(
            "LocalBusiness",
            json!({
                "@type": "LocalBusiness",
                "@id": "https://casoon.de/#org",
                "name": "CASOON",
                "url": "https://casoon.de/",
                "sameAs": ["https://maps.app.goo.gl/abcdef", "https://github.com/casoon"]
            }),
        )]);
        assert!(!issues
            .iter()
            .any(|i| i.issue_type.starts_with("schema_same_as_")));
        assert!(!issues
            .iter()
            .any(|i| i.issue_type == "schema_org_missing_id"));
    }
}
