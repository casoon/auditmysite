use crate::audit::normalized::AuditContext;
use crate::commerce::{commerce_finding_text, CommercePageKind};
use crate::i18n::I18n;
use crate::output::localized::is_english;
use crate::output::report_model::{CommercePresentation, CommerceProductRow};
use crate::taxonomy::SeverityExt;

/// Only produces a section when there is something substantive to say: real
/// product-completeness data (a Product Detail page), or at least one
/// missing mandatory/trust-page link. A page that carries neither (e.g. a
/// shop's Category/Other page where every trust link is already present)
/// has nothing beyond "all six links exist" to add and stays silent instead
/// of padding the single-report with an near-empty chapter (per explicit
/// product decision, 2026-09-01: build a Commerce chapter for the single
/// report too, but only when it has real content).
pub(super) fn build_commerce_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<CommercePresentation> {
    let c = normalized.raw_commerce?;
    let en = is_english(i18n);

    let missing_trust_page = !c.trust_pages.impressum
        || !c.trust_pages.agb
        || !c.trust_pages.widerruf
        || !c.trust_pages.versand
        || !c.trust_pages.zahlungsarten
        || !c.trust_pages.kontakt;
    if c.product.is_none() && !missing_trust_page {
        return None;
    }

    let page_kind_label = match c.page_kind {
        CommercePageKind::ProductDetail => {
            if en {
                "Product detail page"
            } else {
                "Produktdetailseite"
            }
        }
        CommercePageKind::Category => {
            if en {
                "Category page"
            } else {
                "Kategorieseite"
            }
        }
        CommercePageKind::Other => {
            if en {
                "Other shop page"
            } else {
                "Sonstige Shop-Seite"
            }
        }
    }
    .to_string();

    let product = c.product.as_ref().map(|p| CommerceProductRow {
        score: p.score,
        price: p.price.as_ref().map(|price| match &price.currency {
            Some(currency) => format!("{} {currency}", price.value),
            None => price.value.clone(),
        }),
        availability: p.availability.clone(),
        has_shipping_details: p.shipping.has_shipping_details,
        has_return_policy: p.returns.has_return_policy,
        rating: p.reviews.rating_value.clone(),
    });

    let trust_page_label = |key: &str| -> &'static str {
        match (key, en) {
            ("impressum", true) => "Imprint",
            ("impressum", false) => "Impressum",
            ("agb", true) => "Terms (AGB)",
            ("agb", false) => "AGB",
            ("widerruf", true) => "Right of withdrawal / returns",
            ("widerruf", false) => "Widerruf / Retoure",
            ("versand", true) => "Shipping info",
            ("versand", false) => "Versand / Lieferung",
            ("zahlungsarten", true) => "Payment methods",
            ("zahlungsarten", false) => "Zahlungsarten",
            (_, true) => "Contact",
            (_, false) => "Kontakt",
        }
    };
    let trust_pages = vec![
        (
            trust_page_label("impressum").to_string(),
            c.trust_pages.impressum,
        ),
        (trust_page_label("agb").to_string(), c.trust_pages.agb),
        (
            trust_page_label("widerruf").to_string(),
            c.trust_pages.widerruf,
        ),
        (
            trust_page_label("versand").to_string(),
            c.trust_pages.versand,
        ),
        (
            trust_page_label("zahlungsarten").to_string(),
            c.trust_pages.zahlungsarten,
        ),
        (
            trust_page_label("kontakt").to_string(),
            c.trust_pages.kontakt,
        ),
    ];

    let findings = c
        .findings
        .iter()
        .map(|f| {
            let severity_label = if en {
                f.severity.label_en()
            } else {
                f.severity.label()
            };
            (
                severity_label.to_string(),
                commerce_finding_text(f.kind, en),
            )
        })
        .collect();

    Some(CommercePresentation {
        page_kind_label,
        product,
        trust_pages,
        findings,
    })
}
