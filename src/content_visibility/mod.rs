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

pub mod module;
pub use module::ContentVisibilityModule;

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

pub fn analyze_content_visibility(report: &AuditReport, locale: &str) -> ContentVisibilityAnalysis {
    let en = locale == "en";
    let mut out = ContentVisibilityAnalysis::default();

    if let Some(seo) = &report.seo {
        out.organic_visibility = build_organic_visibility(seo, en);

        let lb_signals = build_local_business(seo, en);
        out.local_business = lb_signals;

        out.eeat = build_eeat(seo, report.source_quality.as_ref(), en);
        out.content_depth = build_content_depth(seo, report.source_quality.as_ref(), en);
        out.topical_authority = build_topical_authority(
            seo,
            report.ai_visibility.as_ref(),
            report.patterns.as_ref(),
            en,
        );
    }

    out.finish()
}

// ─── Area builders ───────────────────────────────────────────────────────────

fn build_organic_visibility(seo: &crate::seo::SeoAnalysis, en: bool) -> Vec<ContentSignal> {
    let mut signals = Vec::new();
    let meta = &seo.meta;
    let tech = &seo.technical;

    // Title
    match meta.title.as_deref() {
        None => signals.push(
            ContentSignal::violation(
                ContentArea::Seo,
                EvidenceConfidence::High,
                if en { "Title missing" } else { "Title fehlt" },
                if en {
                    "No <title> element found — the page appears in SERPs without a title."
                } else {
                    "Kein <title>-Element gefunden — Seite wird in SERPs ohne Titel angezeigt."
                },
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
                if en {
                    "Title too short"
                } else {
                    "Title zu kurz"
                },
                if en {
                    format!("Title has {} characters (recommended: 30–60).", t.len())
                } else {
                    format!("Title hat {} Zeichen (empfohlen: 30–60).", t.len())
                },
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
                if en {
                    "Title too long"
                } else {
                    "Title zu lang"
                },
                if en {
                    format!(
                        "Title has {} characters — may be truncated in SERPs.",
                        t.len()
                    )
                } else {
                    format!(
                        "Title hat {} Zeichen — wird in SERPs möglicherweise abgeschnitten.",
                        t.len()
                    )
                },
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
                if en {
                    "Title present & optimal"
                } else {
                    "Title vorhanden & optimal"
                },
                if en {
                    format!("{} characters.", t.len())
                } else {
                    format!("{} Zeichen.", t.len())
                },
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
                if en {
                    "Description missing"
                } else {
                    "Description fehlt"
                },
                if en {
                    "No meta description — Google may auto-generate a snippet."
                } else {
                    "Keine Meta-Description — Google generiert ggf. automatisch einen Snippet."
                },
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
                if en {
                    "Description outside optimal length"
                } else {
                    "Description außerhalb optimaler Länge"
                },
                if en {
                    format!("{} characters (recommended: 120–160).", d.len())
                } else {
                    format!("{} Zeichen (empfohlen: 120–160).", d.len())
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.description"),
            ),
        ),
        Some(_) => signals.push(ContentSignal::pass(
            ContentArea::Seo,
            EvidenceConfidence::High,
            if en {
                "Description present & optimal"
            } else {
                "Description vorhanden & optimal"
            },
            if en {
                format!(
                    "{} characters.",
                    meta.description.as_ref().map(|d| d.len()).unwrap_or(0)
                )
            } else {
                format!(
                    "{} Zeichen.",
                    meta.description.as_ref().map(|d| d.len()).unwrap_or(0)
                )
            },
        )),
    }

    // H1
    match seo.headings.h1_count {
        0 => signals.push(
            ContentSignal::violation(
                ContentArea::Seo,
                EvidenceConfidence::High,
                if en { "H1 missing" } else { "H1 fehlt" },
                if en {
                    "No H1 heading — the page's primary topic is not structurally marked up."
                } else {
                    "Keine H1-Überschrift — primäres Thema der Seite nicht strukturell ausgezeichnet."
                },
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
                if en { "Exactly one H1" } else { "Genau eine H1" },
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
                if en {
                    "Multiple H1 elements"
                } else {
                    "Mehrere H1-Elemente"
                },
                if en {
                    format!("{n} H1 tags found — exactly one H1 per page is recommended.")
                } else {
                    format!("{n} H1-Tags gefunden — empfohlen ist genau eine H1 pro Seite.")
                },
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
            if en { "HTTPS active" } else { "HTTPS aktiv" },
            if en {
                "The page is served over an encrypted connection."
            } else {
                "Seite wird über eine verschlüsselte Verbindung ausgeliefert."
            },
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::HttpHeader,
            EvidenceConfidence::High,
        ))
    } else {
        ContentSignal::violation(
            ContentArea::Seo,
            EvidenceConfidence::High,
            if en { "No HTTPS" } else { "Kein HTTPS" },
            if en {
                "Page not encrypted — ranking disadvantage and a browser warning for users."
            } else {
                "Seite nicht verschlüsselt — Ranking-Nachteil, Browser-Warnung für Nutzer."
            },
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
            if en {
                "Canonical URL set"
            } else {
                "Canonical URL gesetzt"
            },
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
            if en {
                "Canonical missing"
            } else {
                "Canonical fehlt"
            },
            if en {
                "No <link rel=\"canonical\"> — duplicate-content risk across URL variants."
            } else {
                "Kein <link rel=\"canonical\"> — Duplicate-Content-Risiko bei URL-Varianten."
            },
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
                if en {
                    "Rich snippet potential detected"
                } else {
                    "Rich-Snippet-Potenzial erkannt"
                },
                if en {
                    format!(
                        "Structured data enables rich snippets: {}.",
                        seo.structured_data.rich_snippets_potential.join(", ")
                    )
                } else {
                    format!(
                        "Strukturierte Daten ermöglichen Rich Snippets: {}.",
                        seo.structured_data.rich_snippets_potential.join(", ")
                    )
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data.rich_snippets"),
            ),
        );
    }

    signals
}

fn build_local_business(seo: &crate::seo::SeoAnalysis, en: bool) -> Vec<ContentSignal> {
    use crate::assessment::AssessmentLevel;

    let lb_schema = seo.content_profile.as_ref().and_then(|cp| {
        cp.schema_inventory
            .schemas
            .iter()
            .find(|s| s.schema_type == "LocalBusiness")
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
            if en {
                "LocalBusiness schema not found"
            } else {
                "LocalBusiness-Schema nicht gefunden"
            },
            if en {
                "No LocalBusiness JSON-LD on this page — local search signals cannot be checked."
            } else {
                "Kein LocalBusiness JSON-LD auf dieser Seite — lokale Suchsignale nicht prüfbar."
            },
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
                (
                    !v["address"]["postalCode"].is_null(),
                    !v["address"]["addressLocality"].is_null(),
                    !v["telephone"].is_null(),
                    !v["geo"]["latitude"].is_null() || !v["geo"]["longitude"].is_null(),
                    !v["sameAs"].is_null(),
                    !v["aggregateRating"].is_null(),
                )
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
            if en {
                "NAP complete (name, address, phone)"
            } else {
                "NAP vollständig (Name, Adresse, Telefon)"
            },
            if en {
                "LocalBusiness contains name, street, postal code, city and phone number."
            } else {
                "LocalBusiness enthält Name, Straße, PLZ, Ort und Telefonnummer."
            },
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
            if en {
                "NAP incomplete"
            } else {
                "NAP unvollständig"
            },
            if en {
                format!("Missing fields: {}.", missing.join(", "))
            } else {
                format!("Fehlende Felder: {}.", missing.join(", "))
            },
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
            if en {
                "Geo coordinates present"
            } else {
                "Geo-Koordinaten vorhanden"
            },
            if en {
                "latitude and longitude set in LocalBusiness.geo."
            } else {
                "latitude und longitude in LocalBusiness.geo gesetzt."
            },
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.geo"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            if en {
                "Geo coordinates missing"
            } else {
                "Geo-Koordinaten fehlen"
            },
            if en {
                "LocalBusiness.geo.latitude/longitude not set — maps integration limited."
            } else {
                "LocalBusiness.geo.latitude/longitude nicht gesetzt — Maps-Integration eingeschränkt."
            },
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
            if en {
                "sameAs authority links present"
            } else {
                "sameAs-Autoritätslinks vorhanden"
            },
            if en {
                "LocalBusiness references external authority sources (Wikidata, social, etc.)."
            } else {
                "LocalBusiness verweist auf externe Autoritätsquellen (Wikidata, Social, etc.)."
            },
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.sameAs"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            if en { "sameAs missing" } else { "sameAs fehlt" },
            if en {
                "No sameAs links — knowledge panel linking to external sources is not possible."
            } else {
                "Keine sameAs-Links — Knowledge-Panel-Verknüpfung mit externen Quellen nicht möglich."
            },
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
                if en {
                    "aggregateRating present"
                } else {
                    "aggregateRating vorhanden"
                },
                if en {
                    "Rating data in schema — enables a star snippet in SERPs."
                } else {
                    "Bewertungsdaten in Schema — ermöglicht Sterne-Snippet in SERPs."
                },
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
    en: bool,
) -> Vec<ContentSignal> {
    let mut signals = Vec::new();

    // Organization schema
    let has_org = seo
        .structured_data
        .types
        .iter()
        .any(SchemaType::is_organization_like);
    signals.push(if has_org {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            if en {
                "Organization schema detected"
            } else {
                "Organization-Schema erkannt"
            },
            if en {
                "E-E-A-T indicator: the entity is identified as an organization."
            } else {
                "Hinweis auf E-E-A-T: Entität ist als Organisation ausgewiesen."
            },
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("Organization"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::Medium,
            if en {
                "No Organization schema"
            } else {
                "Kein Organization-Schema"
            },
            if en {
                "No indicator of institutional authority."
            } else {
                "Fehlender Hinweis auf institutionelle Autorität."
            },
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
                if en {
                    "Author in Article schema"
                } else {
                    "Autor in Article-Schema"
                },
                if en {
                    format!(
                        "E-E-A-T indicator: the author is identified{}",
                        if author_name.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", author_name)
                        }
                    )
                } else {
                    format!(
                        "Hinweis auf E-E-A-T: Autor ist ausgewiesen{}",
                        if author_name.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", author_name)
                        }
                    )
                },
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
                if en {
                    "No author in Article schema"
                } else {
                    "Kein Autor in Article-Schema"
                },
                if en {
                    "Article schema without an author field — E-E-A-T signal missing."
                } else {
                    "Article-Schema ohne author-Feld — E-E-A-T-Signal fehlt."
                },
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
                if en {
                    "Publication date present"
                } else {
                    "Veröffentlichungsdatum vorhanden"
                },
                if en {
                    "datePublished set in Article schema — recency can be signaled."
                } else {
                    "datePublished in Article-Schema gesetzt — Aktualität signalisierbar."
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.datePublished")
                    .with_value(article.content["datePublished"].as_str().unwrap_or(if en {
                        "(present)"
                    } else {
                        "(vorhanden)"
                    })),
            )
        } else {
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::High,
                if en {
                    "No publication date"
                } else {
                    "Kein Veröffentlichungsdatum"
                },
                if en {
                    "datePublished missing — Google cannot derive recency from the schema."
                } else {
                    "datePublished fehlt — Google kann Aktualität nicht aus Schema ableiten."
                },
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
                if en {
                    "Strong authority signals"
                } else {
                    "Starke Autoritätssignale"
                },
                if en {
                    format!("Source quality authority: {}/100.", auth_score)
                } else {
                    format!("Source-Quality-Autorität: {}/100.", auth_score)
                },
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else if auth_score >= 40 {
            ContentSignal::warning(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                if en {
                    "Weak authority signals"
                } else {
                    "Schwache Autoritätssignale"
                },
                if en {
                    format!(
                        "Source quality authority: {}/100 — review imprint, linking or schema.",
                        auth_score
                    )
                } else {
                    format!(
                        "Source-Quality-Autorität: {}/100 — Impressum, Verlinkung oder Schema prüfen.",
                        auth_score
                    )
                },
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            ContentSignal::violation(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                if en {
                    "Hardly any authority signals detected"
                } else {
                    "Kaum Autoritätssignale erkannt"
                },
                if en {
                    format!("Source quality authority: {}/100.", auth_score)
                } else {
                    format!("Source-Quality-Autorität: {}/100.", auth_score)
                },
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
    en: bool,
) -> Vec<ContentSignal> {
    let mut signals = Vec::new();
    let tech = &seo.technical;

    // Word count
    let wc = tech.word_count;
    signals.push(if wc >= 600 {
        ContentSignal::positive(
            ContentArea::Seo,
            EvidenceConfidence::High,
            if en {
                "Good content depth"
            } else {
                "Gute Inhaltstiefe"
            },
            if en {
                format!("{wc} words — in-depth content recognizable.")
            } else {
                format!("{wc} Wörter — tiefergehender Inhalt erkennbar.")
            },
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
            if en {
                "Sufficient word count"
            } else {
                "Ausreichende Wortanzahl"
            },
            if en {
                format!("{wc} words.")
            } else {
                format!("{wc} Wörter.")
            },
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
            if en { "Little content" } else { "Wenig Inhalt" },
            if en {
                format!("{wc} words — thin-content risk (recommended: ≥ 300).")
            } else {
                format!("{wc} Wörter — Thin-Content-Risiko (empfohlen: ≥ 300).")
            },
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
            if en {
                "Good internal linking"
            } else {
                "Gute interne Verlinkung"
            },
            if en {
                format!("{il} internal links — helps crawlers discover related pages.")
            } else {
                format!("{il} interne Links — hilft Crawlern beim Entdecken verwandter Seiten.")
            },
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
            if en {
                "Internal linking present"
            } else {
                "Interne Verlinkung vorhanden"
            },
            if en {
                format!("{il} internal links.")
            } else {
                format!("{il} interne Links.")
            },
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links"),
        )
    } else {
        ContentSignal::warning(
            ContentArea::Seo,
            EvidenceConfidence::High,
            if en {
                "Hardly any internal links"
            } else {
                "Kaum interne Links"
            },
            if en {
                format!("{il} internal link(s) — the page appears isolated.")
            } else {
                format!("{il} interne Link(s) — Seite wirkt isoliert.")
            },
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
            if en {
                "Language defined"
            } else {
                "Sprache definiert"
            },
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
            if en {
                "Language not defined"
            } else {
                "Sprache nicht definiert"
            },
            if en {
                "No lang attribute on the <html> element — internationalization issue."
            } else {
                "Kein lang-Attribut am <html>-Element — Internationalisierungsproblem."
            },
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
                if en {
                    "Good content substance"
                } else {
                    "Gute inhaltliche Substanz"
                },
                if en {
                    format!("Substance score: {sub_score}/100.")
                } else {
                    format!("Substanz-Score: {sub_score}/100.")
                },
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            ContentSignal::warning(
                ContentArea::SourceQuality,
                EvidenceConfidence::Medium,
                if en {
                    "Weak content substance"
                } else {
                    "Schwache inhaltliche Substanz"
                },
                if en {
                    format!(
                        "Substance score: {sub_score}/100 — review content, structure or sources."
                    )
                } else {
                    format!(
                        "Substanz-Score: {sub_score}/100 — Inhalt, Struktur oder Quellen prüfen."
                    )
                },
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
    patterns: Option<&crate::patterns::PatternAnalysis>,
    en: bool,
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
                if en {
                    "Broad schema coverage"
                } else {
                    "Breite Schema-Abdeckung"
                },
                if en {
                    format!("{schema_count} schemas — thematic diversity recognizable.")
                } else {
                    format!("{schema_count} Schemas — thematische Vielfalt erkennbar.")
                },
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
                if en {
                    "Schema data present"
                } else {
                    "Schema-Daten vorhanden"
                },
                if en {
                    format!("{schema_count} schema(s) found.")
                } else {
                    format!("{schema_count} Schema(s) gefunden.")
                },
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
                if en {
                    "FAQPage schema present"
                } else {
                    "FAQPage-Schema vorhanden"
                },
                if en {
                    "Topic coverage signaled by FAQ questions — good for featured snippets."
                } else {
                    "Themenabdeckung durch FAQ-Fragen signalisiert — gut für Featured Snippets."
                },
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
                if en {
                    "BreadcrumbList detected"
                } else {
                    "BreadcrumbList erkannt"
                },
                if en {
                    "Page is embedded in a content cluster — indicator of thematic structure."
                } else {
                    "Seite ist in einem Inhalts-Cluster eingebettet — Hinweis auf thematische Struktur."
                },
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
                if en {
                    "Multilingual content (hreflang)"
                } else {
                    "Mehrsprachige Inhalte (hreflang)"
                },
                if en {
                    format!("{} language variants linked.", seo.technical.hreflang.len())
                } else {
                    format!(
                        "{} Sprachvarianten verknüpft.",
                        seo.technical.hreflang.len()
                    )
                },
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
                    if en {
                        "Good LLM chunk quality"
                    } else {
                        "Gute LLM-Chunk-Qualität"
                    },
                    if en {
                        format!("AI chunk score: {chunk_score}/100 — content well structured for AI citability.")
                    } else {
                        format!("AI-Chunk-Score: {chunk_score}/100 — Inhalt für KI-Zitierbarkeit gut strukturiert.")
                    },
                )
                .with_evidence(ContentEvidence::new(
                    EvidenceSource::Computed,
                    EvidenceConfidence::Medium,
                )),
            );
        }
    }

    // Internal link cluster — proxy for site embeddedness
    let internal_links = seo.technical.internal_links;
    if internal_links >= 10 {
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                if en {
                    "Internal link mesh recognizable"
                } else {
                    "Internes Linkgeflecht erkennbar"
                },
                if en {
                    format!(
                        "{internal_links} internal links — the page is embedded in a topic cluster."
                    )
                } else {
                    format!(
                        "{internal_links} interne Links — Seite ist in ein Themencluster eingebettet."
                    )
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value(internal_links.to_string()),
            ),
        );
    } else if internal_links >= 3 {
        signals.push(
            ContentSignal::pass(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                if en {
                    "Internal linking present"
                } else {
                    "Interne Verlinkung vorhanden"
                },
                if en {
                    format!("{internal_links} internal links found.")
                } else {
                    format!("{internal_links} interne Links gefunden.")
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value(internal_links.to_string()),
            ),
        );
    } else if internal_links == 0 {
        signals.push(
            ContentSignal::warning(
                ContentArea::Seo,
                EvidenceConfidence::Medium,
                if en {
                    "No internal links"
                } else {
                    "Keine internen Links"
                },
                if en {
                    "Page appears isolated — missing cluster embedding weakens topical authority."
                } else {
                    "Seite wirkt isoliert — fehlende Cluster-Einbettung schwächt topische Autorität."
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value("0".to_string()),
            ),
        );
    }

    // Heading diversity — proxy for semantic topic coverage
    let h2_plus_count = seo
        .headings
        .headings
        .iter()
        .filter(|h| h.level >= 2)
        .count();
    if h2_plus_count >= 5 {
        signals.push(
            ContentSignal::positive(
                ContentArea::Content,
                EvidenceConfidence::Medium,
                if en {
                    "Thematic structure recognizable"
                } else {
                    "Thematische Gliederung erkennbar"
                },
                if en {
                    format!("{h2_plus_count} subheadings (H2+) — broad topic coverage visible in the single run.")
                } else {
                    format!("{h2_plus_count} Unterüberschriften (H2+) — breite Themenabdeckung im Single-Durchlauf sichtbar.")
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    } else if h2_plus_count >= 2 {
        signals.push(
            ContentSignal::pass(
                ContentArea::Content,
                EvidenceConfidence::Medium,
                if en {
                    "Basic structure via headings present"
                } else {
                    "Grundstruktur durch Überschriften vorhanden"
                },
                if en {
                    format!("{h2_plus_count} subheadings (H2+) found.")
                } else {
                    format!("{h2_plus_count} Unterüberschriften (H2+) gefunden.")
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    } else {
        signals.push(
            ContentSignal::warning(
                ContentArea::Content,
                EvidenceConfidence::Low,
                if en {
                    "Flat content structure"
                } else {
                    "Flache Inhaltsstruktur"
                },
                if en {
                    "Fewer than 2 subheadings — topic coverage not recognizable in the single run."
                } else {
                    "Weniger als 2 Unterüberschriften — Themenabdeckung im Single-Durchlauf nicht erkennbar."
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    }

    // Content-type schema — identifies what kind of content this page represents
    let has_content_schema = seo.structured_data.types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Article
                | SchemaType::BlogPosting
                | SchemaType::NewsArticle
                | SchemaType::HowTo
                | SchemaType::WebPage
                | SchemaType::Product
                | SchemaType::Recipe
                | SchemaType::Event
                | SchemaType::VideoObject
        )
    });
    if has_content_schema {
        let schema_label = seo
            .structured_data
            .types
            .iter()
            .find(|t| {
                matches!(
                    t,
                    SchemaType::Article
                        | SchemaType::BlogPosting
                        | SchemaType::NewsArticle
                        | SchemaType::HowTo
                        | SchemaType::WebPage
                        | SchemaType::Product
                        | SchemaType::Recipe
                        | SchemaType::Event
                        | SchemaType::VideoObject
                )
            })
            .map(|t| format!("{t:?}"))
            .unwrap_or_default();
        signals.push(
            ContentSignal::positive(
                ContentArea::Seo,
                EvidenceConfidence::High,
                if en {
                    "Content type identified via schema"
                } else {
                    "Inhaltstyp durch Schema identifiziert"
                },
                if en {
                    format!("{schema_label} schema present — page type clearly recognizable for search engines.")
                } else {
                    format!("{schema_label}-Schema vorhanden — Seitentyp für Suchmaschinen eindeutig erkennbar.")
                },
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data.type")
                    .with_value(schema_label),
            ),
        );
    }

    // Accordion/DisclosureMenu patterns → structured FAQ-like topic coverage
    if let Some(pats) = patterns {
        let has_structured_content = pats
            .recognized
            .iter()
            .any(|p| p.pattern == "Accordion" || p.pattern == "DisclosureMenu");
        if has_structured_content {
            signals.push(
                ContentSignal::pass(
                    ContentArea::Content,
                    EvidenceConfidence::Medium,
                    if en {
                        "Structured content sections detected"
                    } else {
                        "Strukturierte Inhaltsabschnitte erkannt"
                    },
                    if en {
                        "Accordion or disclosure pattern found — indicator of thematically structured content."
                    } else {
                        "Accordion oder Disclosure-Pattern gefunden — Hinweis auf thematisch gegliederte Inhalte."
                    },
                )
                .with_evidence(
                    ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::Medium)
                        .with_field("patterns.recognized"),
                ),
            );
        }
    }

    // Always: not-testable for true authority
    signals.push(ContentSignal::new(
        ContentArea::Content,
        AssessmentLevel::NotTestable,
        EvidenceConfidence::High,
        if en {
            "True topical authority (not testable in the single run)"
        } else {
            "Echte Topical Authority (nicht prüfbar im Single-Durchlauf)"
        },
        if en {
            "Backlinks, SERP positions, historical performance and domain age \
             cannot be assessed automatically."
        } else {
            "Backlinks, SERP-Positionen, historische Performance und Domain-Alter \
             können automatisiert nicht bewertet werden."
        },
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
            image_efficiency: None,
        }
    }

    #[test]
    fn organic_visibility_all_pass() {
        let seo = minimal_seo();
        let signals = build_organic_visibility(&seo, false);
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
        let signals = build_organic_visibility(&seo, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Violation && s.title.contains("Title fehlt")));
    }

    #[test]
    fn local_business_not_testable_when_absent() {
        let seo = minimal_seo();
        let signals = build_local_business(&seo, false);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].level, AssessmentLevel::NotTestable);
    }

    #[test]
    fn topical_authority_always_has_not_testable() {
        let seo = minimal_seo();
        let signals = build_topical_authority(&seo, None, None, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::NotTestable));
    }

    #[test]
    fn content_depth_word_count_signals() {
        let mut seo = minimal_seo();
        seo.technical.word_count = 50;
        let signals = build_content_depth(&seo, None, false);
        assert!(signals.iter().any(|s| s.level.is_problem()));
    }

    #[test]
    fn finish_counts_signals_and_problems() {
        let analysis = ContentVisibilityAnalysis {
            organic_visibility: vec![
                ContentSignal::pass(ContentArea::Seo, EvidenceConfidence::High, "A", ""),
                ContentSignal::violation(ContentArea::Seo, EvidenceConfidence::High, "B", ""),
            ],
            ..ContentVisibilityAnalysis::default()
        };
        let analysis = analysis.finish();
        assert_eq!(analysis.signal_count, 2);
        assert_eq!(analysis.problem_count, 1);
    }

    #[test]
    fn organic_visibility_title_too_short_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("Short".to_string()); // 5 chars < 30
        let signals = build_organic_visibility(&seo, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Warning && s.title.contains("Title zu kurz")));
    }

    #[test]
    fn organic_visibility_title_too_long_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("A".repeat(61)); // 61 chars > 60
        let signals = build_organic_visibility(&seo, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Warning && s.title.contains("Title zu lang")));
    }

    #[test]
    fn organic_visibility_description_too_short_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.description = Some("Too short".to_string()); // < 120
        let signals = build_organic_visibility(&seo, false);
        assert!(signals.iter().any(|s| {
            s.level == AssessmentLevel::Warning
                && s.title.contains("Description außerhalb optimaler Länge")
        }));
    }

    #[test]
    fn organic_visibility_description_too_long_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.description = Some("A".repeat(161)); // > 160
        let signals = build_organic_visibility(&seo, false);
        assert!(signals.iter().any(|s| {
            s.level == AssessmentLevel::Warning
                && s.title.contains("Description außerhalb optimaler Länge")
        }));
    }

    #[test]
    fn eeat_no_organization_schema_is_warning() {
        let seo = minimal_seo();
        let signals = build_eeat(&seo, None, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Warning && s.title.contains("Kein Organization")));
    }

    fn make_json_ld(
        schema_type: &str,
        content: serde_json::Value,
    ) -> crate::seo::schema::JsonLdSchema {
        crate::seo::schema::JsonLdSchema {
            schema_type: schema_type.to_string(),
            schema_types: vec![schema_type.to_string()],
            content,
            is_valid: true,
        }
    }

    #[test]
    fn eeat_with_organization_schema_is_positive() {
        use crate::seo::schema::StructuredData;
        use crate::seo::SchemaType;
        let mut seo = minimal_seo();
        seo.structured_data = StructuredData {
            types: vec![SchemaType::Organization],
            json_ld: vec![make_json_ld(
                "Organization",
                serde_json::json!({"name": "Casoon"}),
            )],
            ..Default::default()
        };
        let signals = build_eeat(&seo, None, false);
        assert!(signals.iter().any(|s| {
            s.level == AssessmentLevel::Positive && s.title.contains("Organization-Schema")
        }));
    }

    #[test]
    fn local_business_nap_incomplete_is_warning() {
        use crate::seo::schema::StructuredData;
        use crate::seo::SchemaType;
        let mut seo = minimal_seo();
        seo.structured_data = StructuredData {
            types: vec![SchemaType::LocalBusiness],
            json_ld: vec![make_json_ld(
                "LocalBusiness",
                serde_json::json!({"name": "Testfirma"}), // missing address fields
            )],
            ..Default::default()
        };
        let signals = build_local_business(&seo, false);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::Warning && s.title.contains("NAP unvollständig")));
    }

    fn report_with_seo(seo: crate::seo::SeoAnalysis) -> AuditReport {
        use crate::audit::ViolationStatistics;
        use crate::cli::WcagLevel;
        use crate::wcag::WcagResults;
        AuditReport {
            url: "https://example.com".into(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            wcag_results: WcagResults::new(),
            score: 95.0,
            grade: "A".into(),
            certificate: "SEHR GUT".into(),
            statistics: ViolationStatistics {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                total: 0,
            },
            nodes_analyzed: 100,
            duration_ms: 1000,
            performance: None,
            seo: Some(seo),
            security: None,
            mobile: None,
            budget_violations: vec![],
            ux: None,
            journey: None,
            dark_mode: None,
            source_quality: None,
            ai_visibility: None,
            content_visibility: None,
            tech_stack: None,
            page_screenshots: None,
            dual_viewport: None,
            viewport_scores: None,
            throttled_performance: vec![],
            patterns: None,
            screenshot_status: Default::default(),
            best_practices: None,
            consent_banner_detected: false,
            consent_banner_cmp: None,
            consent_banner_dismissed: false,
            accessibility_journey: None,
            interactive_findings: Vec::new(),
            advisory_findings: Vec::new(),
            screen_reader_audit: None,
        }
    }

    #[test]
    fn english_locale_has_no_german_chars() {
        // Use a deliberately problematic SEO profile so every code path
        // (violations, warnings, passes) contributes visible strings.
        let mut seo = minimal_seo();
        seo.meta.title = None;
        seo.meta.description = None;
        seo.headings.h1_count = 0;
        seo.technical.https = false;
        seo.technical.has_canonical = false;
        seo.technical.has_lang = false;
        seo.technical.word_count = 50;
        seo.technical.internal_links = 0;
        let report = report_with_seo(seo);

        let analysis = analyze_content_visibility(&report, "en");
        let all = analysis
            .organic_visibility
            .iter()
            .chain(&analysis.local_business)
            .chain(&analysis.eeat)
            .chain(&analysis.content_depth)
            .chain(&analysis.topical_authority);
        for sig in all {
            let combined = format!("{}{}", sig.title, sig.detail);
            assert!(
                !combined.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                "German characters in EN output: {combined:?}"
            );
        }
    }
}
