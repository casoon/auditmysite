use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{MobilePresentation, SecurityPresentation};

use super::super::super::helpers::{security_header_tier_label, security_issue_kind_label, yes_no};
use super::super::super::modules::derive_security_recommendations;
use super::{module_interpretation, normalized_module_grade, normalized_module_score};
use crate::util::truncate_url;

pub(super) fn build_security_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<SecurityPresentation> {
    let locale = i18n.locale();
    normalized.raw_security.map(|sec| {
        let security_score =
            normalized_module_score(&normalized.normalized, "Security").unwrap_or(sec.score);
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            (
                "Content-Security-Policy",
                &sec.headers.content_security_policy,
            ),
            (
                "Strict-Transport-Security",
                &sec.headers.strict_transport_security,
            ),
            (
                "X-Content-Type-Options",
                &sec.headers.x_content_type_options,
            ),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            (
                "Cross-Origin-Opener-Policy",
                &sec.headers.cross_origin_opener_policy,
            ),
            (
                "Cross-Origin-Resource-Policy",
                &sec.headers.cross_origin_resource_policy,
            ),
            (
                "Access-Control-Allow-Origin",
                &sec.headers.access_control_allow_origin,
            ),
            (
                "Access-Control-Allow-Credentials",
                &sec.headers.access_control_allow_credentials,
            ),
        ];

        let mut ssl_info = vec![
            ("HTTPS".to_string(), yes_no(locale, sec.ssl.https)),
            (
                localized_label(locale, "Gültiges Zertifikat", "Valid certificate"),
                yes_no(locale, sec.ssl.valid_certificate),
            ),
            ("HSTS".to_string(), yes_no(locale, sec.ssl.has_hsts)),
            (
                "HSTS Max-Age".to_string(),
                sec.ssl
                    .hsts_max_age
                    .map(|v| format!("{}s", v))
                    .unwrap_or_else(|| "—".to_string()),
            ),
            (
                localized_label(locale, "Subdomains", "Subdomains"),
                yes_no(locale, sec.ssl.hsts_include_subdomains),
            ),
            ("Preload".to_string(), yes_no(locale, sec.ssl.hsts_preload)),
        ];
        push_optional_ssl_row(&mut ssl_info, "TLS", sec.ssl.protocol.as_deref());
        push_optional_ssl_row(&mut ssl_info, "Cipher", sec.ssl.cipher.as_deref());
        push_optional_ssl_row(
            &mut ssl_info,
            localized_label(locale, "Zertifikat Subject", "Certificate subject").as_str(),
            sec.ssl.certificate_subject.as_deref(),
        );
        push_optional_ssl_row(
            &mut ssl_info,
            localized_label(locale, "Zertifikat Issuer", "Certificate issuer").as_str(),
            sec.ssl.certificate_issuer.as_deref(),
        );
        if let Some(days) = sec.ssl.certificate_expires_in_days {
            ssl_info.push(if locale == "en" {
                ("Expires in".to_string(), format!("{days} days"))
            } else {
                ("Läuft ab in".to_string(), format!("{days} Tage"))
            });
        }
        if let Some(chain_length) = sec.ssl.certificate_chain_length {
            ssl_info.push((
                localized_label(locale, "Chain-Länge", "Chain length"),
                chain_length.to_string(),
            ));
        }
        push_optional_ssl_row(
            &mut ssl_info,
            localized_label(locale, "Zertifikatsfehler", "Certificate error").as_str(),
            sec.ssl.certificate_error.as_deref(),
        );

        let band_score =
            crate::audit::interpretation::security_text_band(security_score as f32, &sec.issues)
                .representative_score();
        SecurityPresentation {
            score: security_score,
            grade: normalized_module_grade(&normalized.normalized, "Security")
                .unwrap_or_else(|| sec.grade.clone()),
            interpretation: module_interpretation(&normalized.normalized, "security", locale),
            band_label: crate::registry::FIVE_BAND
                .label(band_score, locale == "en")
                .to_string(),
            band_score: band_score.round() as u32,
            headers: header_checks
                .iter()
                .map(|(name, value)| {
                    let (status, val) = match value {
                        Some(v) => (
                            localized_label(locale, "Vorhanden", "Present"),
                            truncate_url(v, 50),
                        ),
                        None => (localized_label(locale, "Fehlt", "Missing"), "—".to_string()),
                    };
                    let tier =
                        security_header_tier_label(locale, crate::security::header_tier(name));
                    (name.to_string(), status, val, tier)
                })
                .collect(),
            ssl_info,
            issues: sec
                .issues
                .iter()
                .map(|i| {
                    let title = format!(
                        "{} — {}",
                        i.header,
                        security_issue_kind_label(locale, &i.issue_type)
                    );
                    // The stored message is canonical English (#406); the
                    // wording for this report's language is re-derived from
                    // the finding's kind and raw values.
                    (title, i.severity, i.localized_message(locale == "en"))
                })
                .collect(),
            recommendations: derive_security_recommendations(i18n, sec),
            protection: sec
                .protection
                .services
                .iter()
                .map(|s| (s.name.clone(), s.kind.clone()))
                .collect(),
            has_waf: sec.protection.has_waf,
            has_cdn: sec.protection.has_cdn,
        }
    })
}

/// A table label in the run language; these tables printed German labels
/// into English reports.
fn localized_label(locale: &str, de: &str, en: &str) -> String {
    if locale == "en" { en } else { de }.to_string()
}

fn push_optional_ssl_row(rows: &mut Vec<(String, String)>, label: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        rows.push((label.to_string(), truncate_url(value, 80)));
    }
}

pub(super) fn build_mobile_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<MobilePresentation> {
    let locale = i18n.locale();
    normalized.raw_mobile.map(|m| {
        let mobile_score =
            normalized_module_score(&normalized.normalized, "Mobile").unwrap_or(m.score);
        let small_targets = m.touch_targets.small_targets;
        let en = locale == "en";
        let context_hint = if !m.touch_targets.small_by_context.is_empty() {
            let parts: Vec<String> = m
                .touch_targets
                .small_by_context
                .iter()
                .take(3)
                .map(|(ctx, count)| {
                    // The raw key printed "im Bereich other" in German.
                    let area = crate::mobile::mobile_context_label(ctx, en);
                    if en {
                        format!("{} in {}", count, area)
                    } else {
                        format!("{} im Bereich {}", count, area)
                    }
                })
                .collect();
            format!(" ({})", parts.join(", "))
        } else {
            String::new()
        };
        let base_mobile = module_interpretation(&normalized.normalized, "mobile", locale);
        let mobile_interpretation = if small_targets >= 10 {
            // When context is known (e.g. "footer", "navigation"), qualify as locally scoped
            // so a high overall score paired with a large touch-target count reads correctly.
            let scope_note = if !context_hint.is_empty() {
                if en {
                    " — locally scoped"
                } else {
                    " — lokal begrenzt"
                }
            } else {
                ""
            };
            if en {
                format!(
                    "{} {} touch targets smaller than recommended (44×44 px){}{}.",
                    base_mobile, small_targets, context_hint, scope_note,
                )
            } else {
                format!(
                    "{} {} Touch-Targets kleiner als empfohlen (44×44 px){}{}.",
                    base_mobile, small_targets, context_hint, scope_note,
                )
            }
        } else {
            base_mobile
        };
        MobilePresentation {
            score: mobile_score,
            interpretation: mobile_interpretation,
            viewport: vec![
                (
                    "Viewport-Tag".to_string(),
                    yes_no(locale, m.viewport.has_viewport),
                ),
                (
                    "device-width".to_string(),
                    yes_no(locale, m.viewport.uses_device_width),
                ),
                (
                    "Initial Scale".to_string(),
                    yes_no(locale, m.viewport.has_initial_scale),
                ),
                (
                    localized_label(locale, "Skalierbar", "Scalable"),
                    yes_no(locale, m.viewport.is_scalable),
                ),
                (
                    localized_label(locale, "Korrekt konfiguriert", "Correctly configured"),
                    yes_no(locale, m.viewport.is_properly_configured),
                ),
            ],
            touch_targets: vec![
                (
                    localized_label(locale, "Gesamt", "Total"),
                    m.touch_targets.total_targets.to_string(),
                ),
                (
                    localized_label(locale, "Ausreichend (≥44px)", "Adequate (≥44px)"),
                    m.touch_targets.adequate_targets.to_string(),
                ),
                (
                    localized_label(locale, "Zu klein", "Too small"),
                    m.touch_targets.small_targets.to_string(),
                ),
                (
                    localized_label(locale, "Zu eng beieinander", "Too close together"),
                    m.touch_targets.crowded_targets.to_string(),
                ),
            ],
            font_analysis: vec![
                (
                    localized_label(locale, "Basis-Schriftgröße", "Base font size"),
                    format!("{:.0}px", m.font_sizes.base_font_size),
                ),
                (
                    localized_label(locale, "Kleinste Schrift", "Smallest font"),
                    format!("{:.0}px", m.font_sizes.smallest_font_size),
                ),
                (
                    localized_label(locale, "Lesbarer Text", "Legible text"),
                    format!("{:.0}%", m.font_sizes.legible_percentage),
                ),
                (
                    localized_label(locale, "Relative Einheiten", "Relative units"),
                    yes_no(locale, m.font_sizes.uses_relative_units),
                ),
            ],
            content_sizing: vec![
                (
                    localized_label(locale, "Passt in Viewport", "Fits viewport"),
                    yes_no(locale, m.content_sizing.fits_viewport),
                ),
                (
                    localized_label(locale, "Kein hor. Scrollen", "No horizontal scroll"),
                    yes_no(locale, !m.content_sizing.has_horizontal_scroll),
                ),
                (
                    localized_label(locale, "Responsive Bilder", "Responsive images"),
                    yes_no(locale, m.content_sizing.uses_responsive_images),
                ),
                (
                    "Media Queries".to_string(),
                    yes_no(locale, m.content_sizing.uses_media_queries),
                ),
            ],
            issues: m
                .issues
                .iter()
                // The stored message is canonical English (#406); the wording
                // for this report's language is re-derived from kind + values.
                .map(|i| {
                    (
                        i.category.clone(),
                        i.severity,
                        i.localized_message(locale == "en"),
                    )
                })
                .collect(),
        }
    })
}
