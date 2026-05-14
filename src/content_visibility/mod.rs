//! Content Visibility Analysis — issue #54.
//!
//! Aggregates signals from SEO, Source Quality, AI Visibility and Pattern
//! Detection into five thematic areas. Each area produces [`ContentSignal`]
//! items with structured evidence so every claim is traceable to its source.
//!
//! # Areas
//! 1. **Organic Visibility** — title, description, H1, canonical, HTTPS, rich snippets
//! 2. **Local Business** — NAP completeness, geo, sameAs, aggregateRating
//! 3. **E-E-A-T** — author/date in schema, organization, authority signals
//! 4. **Content Depth** — word count, internal links, language
//! 5. **Topical Authority** — schema coverage, breadcrumb, FAQ, heuristic note
//!
//! # Language note
//! Signal titles are intentionally hedged ("Hinweis auf …", "erkennbar",
//! "im Single-Durchlauf") to avoid overstating what automated analysis can prove.

use serde::{Deserialize, Serialize};

use crate::assessment::{
    ContentArea, ContentEvidence, ContentSignal, EvidenceConfidence, EvidenceSource,
};
use crate::audit::AuditReport;
use crate::seo::profile::SchemaExtracted;
use crate::seo::SchemaType;

// ─── Top-level struct ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentVisibilityAnalysis {
    /// Signals about search-engine-facing metadata and indexability.
    pub organic_visibility: Vec<ContentSignal>,
    /// Signals derived from LocalBusiness schema (skipped if absent).
    pub local_business: Vec<ContentSignal>,
    /// E-E-A-T indicators: author, date, organization, authority.
    pub eeat: Vec<ContentSignal>,
    /// Content depth and engagement signals.
    pub content_depth: Vec<ContentSignal>,
    /// Topical authority heuristics (single-run, not definitive).
    pub topical_authority: Vec<ContentSignal>,
    /// Total number of signals across all areas.
    pub signal_count: usize,
    /// Number of signals with level == Violation or Warning.
    pub problem_count: usize,
}

impl ContentVisibilityAnalysis {
    fn finish(mut self) -> Self {
        let all: Vec<&ContentSignal> = self
            .organic_visibility
            .iter()
            .chain(&self.local_business)
            .chain(&self.eeat)
            .chain(&self.content_depth)
            .chain(&self.topical_authority)
            .collect();
        self.signal_count = all.len();
        self.problem_count = all.iter().filter(|s| s.level.is_problem()).count();
        self
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn analyze_content_visibility(report: &AuditReport) -> ContentVisibilityAnalysis {
    let mut out = ContentVisibilityAnalysis::default();

    if let Some(seo) = &report.seo {
        out.organic_visibility = build_organic_visibility(seo);

        let lb_signals = build_local_business(seo);
        out.local_business = lb_signals;

        out.eeat = build_eeat(seo, report.source_quality.as_ref());
        out.content_depth = build_content_depth(seo, report.source_quality.as_ref());
        out.topical_authority =
            build_topical_authority(seo, report.ai_visibility.as_ref(), report.patterns.as_ref());
    }

    out.finish()
}

// ─── Area builders ───────────────────────────────────────────────────────────

fn build_organic_visibility(seo: &crate::seo::SeoAnalysis) -> Vec<ContentSignal> {
    let mut signals = Vec::new();
    let meta = &seo.meta;
    let tech = &seo.technical;

    // Title
    match meta.title.as_deref() {
        None => signals.push(
            ContentSignal::violation(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Title fehlt",
                "Kein <title>-Element gefunden — Seite wird in SERPs ohne Titel angezeigt.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title"),
            ),
        ),
        Some(t) if t.len() < 30 => signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Title zu kurz",
                format!("Title hat {} Zeichen (empfohlen: 30–60).", t.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
        Some(t) if t.len() > 60 => signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Title zu lang",
                format!(
                    "Title hat {} Zeichen — wird in SERPs möglicherweise abgeschnitten.",
                    t.len()
                ),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
        Some(t) => signals.push(
            ContentSignal::pass(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Title vorhanden & optimal",
                format!("{} Zeichen.", t.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
    }

    // Description
    match meta.description.as_deref() {
        None => signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Description fehlt",
                "Keine Meta-Description — Google generiert ggf. automatisch einen Snippet.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.description"),
            ),
        ),
        Some(d) if d.len() < 120 || d.len() > 160 => signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Description außerhalb optimaler Länge",
                format!("{} Zeichen (empfohlen: 120–160).", d.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.description"),
            ),
        ),
        Some(_) => signals.push(ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Description vorhanden & optimal",
            format!(
                "{} Zeichen.",
                meta.description.as_ref().map(|d| d.len()).unwrap_or(0)
            ),
        )),
    }

    // H1
    match seo.headings.h1_count {
        0 => signals.push(
            ContentSignal::violation(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "H1 fehlt",
                "Keine H1-Überschrift — primäres Thema der Seite nicht strukturell ausgezeichnet.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1"),
            ),
        ),
        1 => signals.push(
            ContentSignal::pass(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Genau eine H1",
                seo.headings.h1_text.clone().unwrap_or_default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1")
                    .with_value(seo.headings.h1_text.as_deref().unwrap_or("")),
            ),
        ),
        n => signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Mehrere H1-Elemente",
                format!("{n} H1-Tags gefunden — empfohlen ist genau eine H1 pro Seite."),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1"),
            ),
        ),
    }

    // HTTPS
    signals.push(if tech.https {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "HTTPS aktiv",
            "Seite wird über eine verschlüsselte Verbindung ausgeliefert.",
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::HttpHeader,
            EvidenceConfidence::High,
        ))
    } else {
        ContentSignal::violation(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Kein HTTPS",
            "Seite nicht verschlüsselt — Ranking-Nachteil, Browser-Warnung für Nutzer.",
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::HttpHeader,
            EvidenceConfidence::High,
        ))
    });

    // Canonical
    signals.push(if tech.has_canonical {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Canonical URL gesetzt",
            tech.canonical_url.clone().unwrap_or_default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                .with_field("link[rel=canonical]")
                .with_value(tech.canonical_url.as_deref().unwrap_or("")),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Canonical fehlt",
            "Kein <link rel=\"canonical\"> — Duplicate-Content-Risiko bei URL-Varianten.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                .with_field("link[rel=canonical]"),
        )
    });

    // Rich snippets
    if !seo.structured_data.rich_snippets_potential.is_empty() {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                "Rich-Snippet-Potenzial erkannt",
                format!(
                    "Strukturierte Daten ermöglichen Rich Snippets: {}.",
                    seo.structured_data.rich_snippets_potential.join(", ")
                ),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data.rich_snippets"),
            ),
        );
    }

    signals
}

fn build_local_business(seo: &crate::seo::SeoAnalysis) -> Vec<ContentSignal> {
    use crate::assessment::AssessmentLevel;

    let lb_schema = seo
        .content_profile
        .as_ref()
        .and_then(|cp| {
            cp.schema_inventory.schemas.iter().find(|s| {
                s.schema_type == "LocalBusiness"
                    || seo
                        .structured_data
                        .types
                        .contains(&SchemaType::LocalBusiness)
            })
        })
        .or_else(|| {
            seo.content_profile.as_ref().and_then(|cp| {
                cp.schema_inventory
                    .schemas
                    .iter()
                    .find(|s| s.schema_type == "LocalBusiness")
            })
        });

    let raw = seo
        .structured_data
        .json_ld
        .iter()
        .find(|s| s.schema_type == "LocalBusiness")
        .map(|s| &s.content);

    if raw.is_none()
        && !seo
            .structured_data
            .types
            .contains(&SchemaType::LocalBusiness)
    {
        return vec![ContentSignal::new(
            ContentArea::Seo,
            AssessmentLevel::NotTestable,
            EvidenceConfidence::High,
            "LocalBusiness-Schema nicht gefunden",
            "Kein LocalBusiness JSON-LD auf dieser Seite — lokale Suchsignale nicht prüfbar.",
        )];
    }

    let mut signals = Vec::new();
    let v = match raw {
        Some(r) => r,
        None => return signals,
    };

    // Extract fields from LocalBusiness extracted data if available
    let (postal_ok, locality_ok, phone_ok, has_geo, has_same_as, has_rating) =
        if let Some(schema) = lb_schema {
            if let SchemaExtracted::LocalBusiness {
                postal_code,
                locality,
                phone,
                latitude,
                longitude,
                same_as,
                aggregate_rating,
                ..
            } = &schema.extracted
            {
                (
                    postal_code.is_some(),
                    locality.is_some(),
                    phone.is_some(),
                    latitude.is_some() || longitude.is_some(),
                    !same_as.is_empty(),
                    aggregate_rating.is_some(),
                )
            } else {
                (false, false, false, false, false, false)
            }
        } else {
            (
                !v["address"]["postalCode"].is_null(),
                !v["address"]["addressLocality"].is_null(),
                !v["telephone"].is_null(),
                !v["geo"]["latitude"].is_null() || !v["geo"]["longitude"].is_null(),
                !v["sameAs"].is_null(),
                !v["aggregateRating"].is_null(),
            )
        };

    let has_name = !v["name"].is_null();
    let has_street = !v["address"]["streetAddress"].is_null();
    let nap_complete = has_name && has_street && postal_ok && locality_ok && phone_ok;

    signals.push(if nap_complete {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "NAP vollständig (Name, Adresse, Telefon)",
            "LocalBusiness enthält Name, Straße, PLZ, Ort und Telefonnummer.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.address"),
        )
    } else {
        let missing: Vec<&str> = [
            (!has_name).then_some("name"),
            (!has_street).then_some("streetAddress"),
            (!postal_ok).then_some("postalCode"),
            (!locality_ok).then_some("addressLocality"),
            (!phone_ok).then_some("telephone"),
        ]
        .into_iter()
        .flatten()
        .collect();
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "NAP unvollständig",
            format!("Fehlende Felder: {}.", missing.join(", ")),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.address"),
        )
    });

    signals.push(if has_geo {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Geo-Koordinaten vorhanden",
            "latitude und longitude in LocalBusiness.geo gesetzt.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.geo"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            "Geo-Koordinaten fehlen",
            "LocalBusiness.geo.latitude/longitude nicht gesetzt — Maps-Integration eingeschränkt.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::Medium)
                .with_field("LocalBusiness.geo"),
        )
    });

    signals.push(if has_same_as {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "sameAs-Autoritätslinks vorhanden",
            "LocalBusiness verweist auf externe Autoritätsquellen (Wikidata, Social, etc.).",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.sameAs"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            "sameAs fehlt",
            "Keine sameAs-Links — Knowledge-Panel-Verknüpfung mit externen Quellen nicht möglich.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::Medium)
                .with_field("LocalBusiness.sameAs"),
        )
    });

    if has_rating {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "aggregateRating vorhanden",
                "Bewertungsdaten in Schema — ermöglicht Sterne-Snippet in SERPs.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("LocalBusiness.aggregateRating"),
            ),
        );
    }

    signals
}

fn build_eeat(
    seo: &crate::seo::SeoAnalysis,
    source_quality: Option<&crate::source_quality::SourceQualityAnalysis>,
) -> Vec<ContentSignal> {
    let mut signals = Vec::new();

    // Organization schema
    let has_org = seo
        .structured_data
        .types
        .iter()
        .any(|t| matches!(t, SchemaType::Organization | SchemaType::LocalBusiness));
    signals.push(if has_org {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Organization-Schema erkannt",
            "Hinweis auf E-E-A-T: Entität ist als Organisation ausgewiesen.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("Organization"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            "Kein Organization-Schema",
            "Fehlender Hinweis auf institutionelle Autorität.",
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::JsonLd,
            EvidenceConfidence::Medium,
        ))
    });

    // Author in Article schema
    let article_schema = seo.structured_data.json_ld.iter().find(|s| {
        matches!(
            s.schema_type.as_str(),
            "Article" | "BlogPosting" | "NewsArticle"
        )
    });
    if let Some(article) = article_schema {
        let has_author = !article.content["author"].is_null();
        let has_date = !article.content["datePublished"].is_null();

        signals.push(if has_author {
            let author_name = article.content["author"]["name"]
                .as_str()
                .or_else(|| article.content["author"].as_str())
                .unwrap_or("");
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Autor in Article-Schema",
                format!(
                    "Hinweis auf E-E-A-T: Autor ist ausgewiesen{}",
                    if author_name.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", author_name)
                    }
                ),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.author")
                    .with_value(author_name),
            )
        } else {
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Kein Autor in Article-Schema",
                "Article-Schema ohne author-Feld — E-E-A-T-Signal fehlt.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.author"),
            )
        });

        signals.push(if has_date {
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Veröffentlichungsdatum vorhanden",
                "datePublished in Article-Schema gesetzt — Aktualität signalisierbar.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.datePublished")
                    .with_value(
                        article.content["datePublished"]
                            .as_str()
                            .unwrap_or("(vorhanden)"),
                    ),
            )
        } else {
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Kein Veröffentlichungsdatum",
                "datePublished fehlt — Google kann Aktualität nicht aus Schema ableiten.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.datePublished"),
            )
        });
    }

    // Source quality authority
    if let Some(sq) = source_quality {
        let auth_score = sq.authority.score;
        signals.push(if auth_score >= 70 {
            ContentSignal::positive(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                "Starke Autoritätssignale",
                format!("Source-Quality-Autorität: {}/100.", auth_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else if auth_score >= 40 {
            ContentSignal::warning(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                "Schwache Autoritätssignale",
                format!(
                    "Source-Quality-Autorität: {}/100 — Impressum, Verlinkung oder Schema prüfen.",
                    auth_score
                ),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            ContentSignal::violation(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                "Kaum Autoritätssignale erkannt",
                format!("Source-Quality-Autorität: {}/100.", auth_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        });
    }

    signals
}

fn build_content_depth(
    seo: &crate::seo::SeoAnalysis,
    source_quality: Option<&crate::source_quality::SourceQualityAnalysis>,
) -> Vec<ContentSignal> {
    let mut signals = Vec::new();
    let tech = &seo.technical;

    // Word count
    let wc = tech.word_count;
    signals.push(if wc >= 600 {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Gute Inhaltstiefe",
            format!("{wc} Wörter — tiefergehender Inhalt erkennbar."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    } else if wc >= 300 {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Ausreichende Wortanzahl",
            format!("{wc} Wörter."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Wenig Inhalt",
            format!("{wc} Wörter — Thin-Content-Risiko (empfohlen: ≥ 300)."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    });

    // Internal links
    let il = tech.internal_links;
    signals.push(if il >= 5 {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Gute interne Verlinkung",
            format!("{il} interne Links — hilft Crawlern beim Entdecken verwandter Seiten."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links")
                .with_value(il.to_string()),
        )
    } else if il >= 2 {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Interne Verlinkung vorhanden",
            format!("{il} interne Links."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Kaum interne Links",
            format!("{il} interne Link(s) — Seite wirkt isoliert."),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links"),
        )
    });

    // Language
    signals.push(if tech.has_lang {
        ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Sprache definiert",
            tech.lang.clone().unwrap_or_default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::DomAttribute, EvidenceConfidence::High)
                .with_field("html[lang]")
                .with_value(tech.lang.as_deref().unwrap_or("")),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            "Sprache nicht definiert",
            "Kein lang-Attribut am <html>-Element — Internationalisierungsproblem.",
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::DomAttribute, EvidenceConfidence::High)
                .with_field("html[lang]"),
        )
    });

    // Source quality substance
    if let Some(sq) = source_quality {
        let sub_score = sq.substance.score;
        signals.push(if sub_score >= 70 {
            ContentSignal::positive(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                "Gute inhaltliche Substanz",
                format!("Substanz-Score: {sub_score}/100."),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            ContentSignal::warning(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                "Schwache inhaltliche Substanz",
                format!("Substanz-Score: {sub_score}/100 — Inhalt, Struktur oder Quellen prüfen."),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        });
    }

    signals
}

fn build_topical_authority(
    seo: &crate::seo::SeoAnalysis,
    ai_visibility: Option<&crate::ai_visibility::AiVisibilityAnalysis>,
    _patterns: Option<&crate::patterns::PatternAnalysis>,
) -> Vec<ContentSignal> {
    use crate::assessment::AssessmentLevel;

    let mut signals = Vec::new();

    // Schema coverage (thematic richness)
    let schema_count = seo.structured_data.json_ld.len();
    if schema_count >= 3 {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                "Breite Schema-Abdeckung",
                format!("{schema_count} Schemas — thematische Vielfalt erkennbar."),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data")
                    .with_value(schema_count.to_string()),
            ),
        );
    } else if schema_count > 0 {
        signals.push(
            ContentSignal::pass(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                "Schema-Daten vorhanden",
                format!("{schema_count} Schema(s) gefunden."),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data"),
            ),
        );
    }

    // FAQ schema → topic coverage signal
    let has_faq = seo.structured_data.types.contains(&SchemaType::FAQPage);
    if has_faq {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "FAQPage-Schema vorhanden",
                "Themenabdeckung durch FAQ-Fragen signalisiert — gut für Featured Snippets.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("FAQPage"),
            ),
        );
    }

    // BreadcrumbList → content hierarchy
    let has_breadcrumb = seo
        .structured_data
        .types
        .contains(&SchemaType::BreadcrumbList);
    if has_breadcrumb {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "BreadcrumbList erkannt",
                "Seite ist in einem Inhalts-Cluster eingebettet — Hinweis auf thematische Struktur.",
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("BreadcrumbList"),
            ),
        );
    }

    // Hreflang → multilingual content authority
    if seo.technical.has_hreflang {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                "Mehrsprachige Inhalte (hreflang)",
                format!(
                    "{} Sprachvarianten verknüpft.",
                    seo.technical.hreflang.len()
                ),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("link[rel=alternate][hreflang]"),
            ),
        );
    }

    // AI visibility chunk quality → LLM-citability
    if let Some(ai) = ai_visibility {
        let chunk_score = ai.chunks.dimension.score;
        if chunk_score >= 70 {
            signals.push(
                ContentSignal::positive(
                    ContentArea::AiVisibility,
                    EvidenceConfidence::Medium,
                    "Gute LLM-Chunk-Qualität",
                    format!("AI-Chunk-Score: {chunk_score}/100 — Inhalt für KI-Zitierbarkeit gut strukturiert."),
                )
                .with_evidence(ContentEvidence::new(
                    EvidenceSource::Computed,
                    EvidenceConfidence::Medium,
                )),
            );
        }
    }

    // Always: not-testable for true authority
    signals.push(ContentSignal::new(
        ContentArea::Content,
        AssessmentLevel::NotTestable,
        EvidenceConfidence::High,
        "Echte Topical Authority (nicht prüfbar im Single-Durchlauf)",
        "Backlinks, SERP-Positionen, historische Performance und Domain-Alter \
         können automatisiert nicht bewertet werden.",
    ));

    signals
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::AssessmentLevel;

    fn minimal_seo() -> crate::seo::SeoAnalysis {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        SeoAnalysis {
            meta: MetaTags {
                title: Some("Test Page Example Site Accessibility".to_string()),
                description: Some("A test description that is long enough for SEO purposes and fits perfectly within the recommended 120 to 160 character limit for meta descriptions.".to_string()),
                ..Default::default()
            },
            headings: HeadingStructure {
                h1_count: 1,
                h1_text: Some("Test Page".to_string()),
                ..Default::default()
            },
            technical: TechnicalSeo {
                https: true,
                has_canonical: true,
                canonical_url: Some("https://example.com/test".to_string()),
                has_lang: true,
                lang: Some("de".to_string()),
                word_count: 450,
                internal_links: 4,
                ..Default::default()
            },
            social: SocialTags::default(),
            structured_data: StructuredData::default(),
            score: 80,
            content_profile: None,
            robots: None,
            page_health: None,
            serp: None,
            meta_issues: vec![],
        }
    }

    #[test]
    fn organic_visibility_all_pass() {
        let seo = minimal_seo();
        let signals = build_organic_visibility(&seo);
        let problems: Vec<_> = signals.iter().filter(|s| s.level.is_problem()).collect();
        assert!(
            problems.is_empty(),
            "Expected no problems, got: {:?}",
            problems.iter().map(|s| &s.title).collect::<Vec<_>>()
        );
    }

    #[test]
    fn organic_visibility_missing_title() {
        let mut seo = minimal_seo();
        seo.meta.title = None;
        let signals = build_organic_visibility(&seo);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Violation && s.title.contains("Title fehlt")));
    }

    #[test]
    fn local_business_not_testable_when_absent() {
        let seo = minimal_seo();
        let signals = build_local_business(&seo);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].level, AssessmentLevel::NotTestable);
    }

    #[test]
    fn topical_authority_always_has_not_testable() {
        let seo = minimal_seo();
        let signals = build_topical_authority(&seo, None, None);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::NotTestable));
    }

    #[test]
    fn content_depth_word_count_signals() {
        let mut seo = minimal_seo();
        seo.technical.word_count = 50;
        let signals = build_content_depth(&seo, None);
        assert!(signals.iter().any(|s| s.level.is_problem()));
    }

    #[test]
    fn finish_counts_signals_and_problems() {
        let mut analysis = ContentVisibilityAnalysis::default();
        analysis.organic_visibility = vec![
            ContentSignal::pass(ContentArea::Seo, EvidenceConfidence::High, "A", ""),
            ContentSignal::violation(ContentArea::Seo, EvidenceConfidence::High, "B", ""),
        ];
        let analysis = analysis.finish();
        assert_eq!(analysis.signal_count, 2);
        assert_eq!(analysis.problem_count, 1);
    }
}
