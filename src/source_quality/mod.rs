//! Source Quality Analysis
//!
//! Interprets data from existing audit modules (Accessibility, SEO, Security,
//! UX) to assess the quality of a website **as an information source**.
//!
//! Three dimensions are scored:
//! - **Substance** — Does the site treat its content as valuable?
//! - **Consistency** — Does the site maintain its own standards?
//! - **Authority** — Does the site present itself as a trustworthy source?
//!
//! **Disclaimer**: This is a purely technical assessment based on structural,
//! semantic, and metadata signals. It does NOT evaluate whether the content
//! itself is factually correct, complete, or up to date.

pub mod module;
pub use module::SourceQualityModule;

use serde::{Deserialize, Serialize};

use crate::audit::AuditReport;
use crate::seo::schema::SchemaType;
use crate::taxonomy::module_score_grade;

// ─── Public types ────────────────────────────────────────────────────────────

/// Complete source quality analysis for a single page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceQualityAnalysis {
    /// Overall source quality score (0–100)
    pub score: u32,
    /// Letter grade (A–F)
    pub grade: String,
    /// Substance dimension
    pub substance: DimensionScore,
    /// Consistency dimension (limited for single page, full in batch)
    pub consistency: DimensionScore,
    /// Authority dimension
    pub authority: DimensionScore,
    /// Always-present disclaimer
    pub disclaimer: String,
}

/// Score for a single quality dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Dimension name (Substanz / Konsistenz / Autorität)
    pub name: String,
    /// Score (0–100)
    pub score: u32,
    /// Short assessment
    pub label: String,
    /// Individual signals evaluated
    pub signals: Vec<QualitySignal>,
}

/// A single measurable signal contributing to a dimension score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySignal {
    /// What was checked
    pub name: String,
    /// Whether the signal is positive
    pub present: bool,
    /// Weight of this signal within its dimension (0.0–1.0)
    pub weight: f32,
    /// Human-readable detail
    pub detail: String,
}

const DISCLAIMER_DE: &str = "Diese Bewertung basiert ausschließlich auf technischen Signalen \
    (Struktur, Semantik, Metadaten, Sicherheit). Sie beurteilt nicht, ob die \
    dargestellten Inhalte inhaltlich korrekt, vollständig oder aktuell sind.";

const DISCLAIMER_EN: &str = "This assessment is based solely on technical signals \
    (structure, semantics, metadata, security). It does not judge whether the \
    presented content is factually correct, complete or up to date.";

fn disclaimer(en: bool) -> String {
    if en { DISCLAIMER_EN } else { DISCLAIMER_DE }.to_string()
}

// Dimension names are stable across runs but surface in the report.
fn substance_name(en: bool) -> &'static str {
    if en {
        "Substance"
    } else {
        "Substanz"
    }
}
fn consistency_name(en: bool) -> &'static str {
    if en {
        "Consistency"
    } else {
        "Konsistenz"
    }
}
fn authority_name(en: bool) -> &'static str {
    if en {
        "Authority"
    } else {
        "Autorität"
    }
}

// ─── Analysis entry point ────────────────────────────────────────────────────

/// Derive source quality from an existing audit report (single page).
pub fn analyze_source_quality(report: &AuditReport, locale: &str) -> SourceQualityAnalysis {
    let en = locale == "en";
    let substance = evaluate_substance(report, en);
    let consistency = evaluate_single_page_consistency(report, en);
    let authority = evaluate_authority(report, en);

    let score = weighted_average(&[
        (substance.score, 40),
        (consistency.score, 25),
        (authority.score, 35),
    ]);

    SourceQualityAnalysis {
        score,
        grade: module_score_grade(score).to_string(),
        substance,
        consistency,
        authority,
        disclaimer: disclaimer(en),
    }
}

/// Derive source quality for batch mode with cross-page consistency.
pub fn analyze_source_quality_batch(
    reports: &[AuditReport],
    locale: &str,
) -> SourceQualityAnalysis {
    let en = locale == "en";
    if reports.is_empty() {
        return empty_analysis(en);
    }

    // Average substance and authority across pages
    let substance_scores: Vec<DimensionScore> =
        reports.iter().map(|r| evaluate_substance(r, en)).collect();
    let authority_scores: Vec<DimensionScore> =
        reports.iter().map(|r| evaluate_authority(r, en)).collect();

    let avg_substance = average_dimensions(&substance_scores, substance_name(en), en);
    let avg_authority = average_dimensions(&authority_scores, authority_name(en), en);

    // Cross-page consistency (the real batch value)
    let consistency = evaluate_cross_page_consistency(reports, en);

    let score = weighted_average(&[
        (avg_substance.score, 35),
        (consistency.score, 30),
        (avg_authority.score, 35),
    ]);

    SourceQualityAnalysis {
        score,
        grade: module_score_grade(score).to_string(),
        substance: avg_substance,
        consistency,
        authority: avg_authority,
        disclaimer: disclaimer(en),
    }
}

// ─── Substance ───────────────────────────────────────────────────────────────

fn evaluate_substance(report: &AuditReport, en: bool) -> DimensionScore {
    let mut signals = Vec::new();

    // 1. Heading structure depth
    if let Some(seo) = &report.seo {
        let has_h1 = seo.headings.h1_count > 0;
        let depth = seo
            .headings
            .headings
            .iter()
            .map(|h| h.level)
            .max()
            .unwrap_or(0);
        let good_depth = depth >= 3;

        signals.push(QualitySignal {
            name: if en {
                "Heading structure".into()
            } else {
                "Überschriftenstruktur".into()
            },
            present: has_h1 && good_depth,
            weight: 0.20,
            detail: if has_h1 && good_depth {
                if en {
                    format!("Structured outline down to H{}", depth)
                } else {
                    format!("Strukturierte Gliederung bis H{}", depth)
                }
            } else if !has_h1 {
                if en {
                    "No H1 heading present".into()
                } else {
                    "Keine H1-Überschrift vorhanden".into()
                }
            } else if en {
                format!("Flat outline (only down to H{})", depth)
            } else {
                format!("Flache Gliederung (nur bis H{})", depth)
            },
        });

        // 2. Word count / content density
        let word_count = seo.technical.word_count;
        let substantial = word_count >= 300;
        signals.push(QualitySignal {
            name: if en {
                "Content volume".into()
            } else {
                "Inhaltsumfang".into()
            },
            present: substantial,
            weight: 0.15,
            detail: if en {
                format!(
                    "{} words{}",
                    word_count,
                    if substantial {
                        ""
                    } else {
                        " (heuristic: typically ≥ 300 words recommended)"
                    }
                )
            } else {
                format!(
                    "{} Wörter{}",
                    word_count,
                    if substantial {
                        ""
                    } else {
                        " (Heuristik: typisch ≥ 300 Wörter empfohlen)"
                    }
                )
            },
        });

        // 3. Schema.org structured data
        let has_schema = seo.structured_data.has_structured_data;
        let schema_types: Vec<&str> = seo
            .structured_data
            .types
            .iter()
            .map(|t| t.as_str())
            .collect();
        signals.push(QualitySignal {
            name: if en {
                "Structured data".into()
            } else {
                "Strukturierte Daten".into()
            },
            present: has_schema,
            weight: 0.20,
            detail: if has_schema {
                format!("Schema.org: {}", schema_types.join(", "))
            } else if en {
                "No structured data".into()
            } else {
                "Keine strukturierten Daten".into()
            },
        });

        // 4. Meta description
        let has_meta_desc = seo.meta.description.as_ref().is_some_and(|d| d.len() >= 50);
        signals.push(QualitySignal {
            name: if en {
                "Meta description".into()
            } else {
                "Meta-Beschreibung".into()
            },
            present: has_meta_desc,
            weight: 0.10,
            detail: if has_meta_desc {
                if en {
                    "Meaningful meta description present".into()
                } else {
                    "Aussagekräftige Meta-Beschreibung vorhanden".into()
                }
            } else if en {
                "Missing or too short meta description".into()
            } else {
                "Keine oder zu kurze Meta-Beschreibung".into()
            },
        });

        // 5. Language declaration
        let has_lang = seo.technical.has_lang;
        signals.push(QualitySignal {
            name: if en {
                "Language declaration".into()
            } else {
                "Sprachdeklaration".into()
            },
            present: has_lang,
            weight: 0.10,
            detail: if has_lang {
                if en {
                    format!(
                        "Language declared: {}",
                        seo.technical.lang.as_deref().unwrap_or("?")
                    )
                } else {
                    format!(
                        "Sprache deklariert: {}",
                        seo.technical.lang.as_deref().unwrap_or("?")
                    )
                }
            } else if en {
                "No language declaration".into()
            } else {
                "Keine Sprachdeklaration".into()
            },
        });
    }

    // 6. Accessibility — image alt text coverage
    let image_violations = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.1.1")
        .count();
    let good_alt = image_violations == 0;
    signals.push(QualitySignal {
        name: if en {
            "Image descriptions".into()
        } else {
            "Bildbeschreibungen".into()
        },
        present: good_alt,
        weight: 0.15,
        detail: if good_alt {
            if en {
                "All images have alternative text".into()
            } else {
                "Alle Bilder haben Alternativtexte".into()
            }
        } else if en {
            format!("{} images without alternative text", image_violations)
        } else {
            format!("{} Bilder ohne Alternativtext", image_violations)
        },
    });

    // 7. Landmark structure
    let landmark_violations = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule_name.contains("Landmark"))
        .count();
    let missing_main_in_ax = report
        .interactive_findings
        .iter()
        .any(|f| f.category == "Landmark" && f.message.contains("Kein <main>-Landmark"));
    let good_landmarks = landmark_violations == 0 && !missing_main_in_ax;
    signals.push(QualitySignal {
        name: if en {
            "Semantic structure".into()
        } else {
            "Semantische Struktur".into()
        },
        present: good_landmarks,
        weight: 0.10,
        detail: if good_landmarks {
            if en {
                "Correct landmark regions".into()
            } else {
                "Korrekte Landmark-Regionen".into()
            }
        } else if missing_main_in_ax {
            if en {
                "<main> landmark not detectable in the accessibility tree".into()
            } else {
                "<main>-Landmark im Accessibility Tree nicht nachweisbar".into()
            }
        } else if en {
            format!("{} structural issues", landmark_violations)
        } else {
            format!("{} Strukturprobleme", landmark_violations)
        },
    });

    build_dimension(substance_name(en), &signals, en)
}

// ─── Authority ───────────────────────────────────────────────────────────────

fn evaluate_authority(report: &AuditReport, en: bool) -> DimensionScore {
    let mut signals = Vec::new();

    // 1. HTTPS
    let has_https = report.url.starts_with("https://");
    signals.push(QualitySignal {
        name: "HTTPS".into(),
        present: has_https,
        weight: 0.15,
        detail: if has_https {
            if en {
                "Encrypted connection".into()
            } else {
                "Verschlüsselte Verbindung".into()
            }
        } else if en {
            "No HTTPS encryption".into()
        } else {
            "Keine HTTPS-Verschlüsselung".into()
        },
    });

    // 2. Security headers
    if let Some(sec) = &report.security {
        let header_count = [
            sec.headers.strict_transport_security.is_some(),
            sec.headers.content_security_policy.is_some(),
            sec.headers.x_content_type_options.is_some(),
            sec.headers.referrer_policy.is_some(),
        ]
        .iter()
        .filter(|&&b| b)
        .count();

        signals.push(QualitySignal {
            name: if en {
                "Security headers".into()
            } else {
                "Sicherheits-Header".into()
            },
            present: header_count >= 3,
            weight: 0.15,
            detail: if en {
                format!("{}/4 relevant security headers set", header_count)
            } else {
                format!("{}/4 relevante Security-Header gesetzt", header_count)
            },
        });
    }

    // 3. Schema.org Organization / Author
    if let Some(seo) = &report.seo {
        let has_org = seo.structured_data.types.iter().any(|t| {
            t.is_organization_like() || matches!(t, SchemaType::Person | SchemaType::WebSite)
        });
        signals.push(QualitySignal {
            name: if en {
                "Publisher identity".into()
            } else {
                "Herausgeber-Identität".into()
            },
            present: has_org,
            weight: 0.20,
            detail: if has_org {
                if en {
                    "Organization/publisher identified via Schema.org".into()
                } else {
                    "Organisation/Herausgeber per Schema.org identifiziert".into()
                }
            } else if en {
                "No publisher markup".into()
            } else {
                "Kein Herausgeber-Markup".into()
            },
        });

        // 4. Canonical URL
        let has_canonical = seo.technical.has_canonical;
        signals.push(QualitySignal {
            name: "Canonical URL".into(),
            present: has_canonical,
            weight: 0.10,
            detail: if has_canonical {
                if en {
                    "Canonical URL declared".into()
                } else {
                    "Kanonische URL deklariert".into()
                }
            } else if en {
                "No canonical URL".into()
            } else {
                "Keine Canonical-URL".into()
            },
        });

        // 5. Social meta / Open Graph
        let has_og = seo
            .social
            .open_graph
            .as_ref()
            .is_some_and(|og| og.title.is_some() && og.description.is_some());
        signals.push(QualitySignal {
            name: if en {
                "Social meta".into()
            } else {
                "Social-Meta".into()
            },
            present: has_og,
            weight: 0.10,
            detail: if has_og {
                if en {
                    "Open Graph metadata present".into()
                } else {
                    "Open Graph Metadaten vorhanden".into()
                }
            } else if en {
                "Incomplete social metadata".into()
            } else {
                "Unvollständige Social-Metadaten".into()
            },
        });
    }

    // 6. Accessibility score as quality signal
    let a11y_good = report.score >= 80.0;
    signals.push(QualitySignal {
        name: if en {
            "Accessibility".into()
        } else {
            "Barrierefreiheit".into()
        },
        present: a11y_good,
        weight: 0.15,
        detail: if en {
            format!(
                "Accessibility score: {:.0}{}",
                report.score,
                if a11y_good { "" } else { " (low)" }
            )
        } else {
            format!(
                "Accessibility-Score: {:.0}{}",
                report.score,
                if a11y_good { "" } else { " (niedrig)" }
            )
        },
    });

    // 7. Trust signals from UX module
    if let Some(ux) = &report.ux {
        let trust_good = ux.trust_signals.score >= 70;
        signals.push(QualitySignal {
            name: if en {
                "Trust signals".into()
            } else {
                "Vertrauenssignale".into()
            },
            present: trust_good,
            weight: 0.15,
            detail: if en {
                format!(
                    "UX trust score: {}{}",
                    ux.trust_signals.score,
                    if trust_good { "" } else { " (weak)" }
                )
            } else {
                format!(
                    "UX Trust-Score: {}{}",
                    ux.trust_signals.score,
                    if trust_good { "" } else { " (schwach)" }
                )
            },
        });
    }

    build_dimension(authority_name(en), &signals, en)
}

// ─── Consistency (single page) ───────────────────────────────────────────────

fn evaluate_single_page_consistency(report: &AuditReport, en: bool) -> DimensionScore {
    let mut signals = Vec::new();

    // 1. Heading hierarchy (no skips)
    if let Some(seo) = &report.seo {
        let no_heading_issues = seo.headings.issues.is_empty();
        signals.push(QualitySignal {
            name: if en {
                "Heading hierarchy".into()
            } else {
                "Überschriften-Hierarchie".into()
            },
            present: no_heading_issues,
            weight: 0.25,
            detail: if no_heading_issues {
                if en {
                    "Gapless heading hierarchy".into()
                } else {
                    "Lückenlose Überschriften-Hierarchie".into()
                }
            } else if en {
                format!("{} hierarchy issues", seo.headings.issues.len())
            } else {
                format!("{} Hierarchie-Probleme", seo.headings.issues.len())
            },
        });
    }

    // 2. All interactive elements named
    let unnamed_interactive = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "4.1.2" || v.rule == "1.1.1")
        .count();
    signals.push(QualitySignal {
        name: if en {
            "Named controls".into()
        } else {
            "Benannte Bedienelemente".into()
        },
        present: unnamed_interactive == 0,
        weight: 0.25,
        detail: if unnamed_interactive == 0 {
            if en {
                "All interactive elements correctly named".into()
            } else {
                "Alle interaktiven Elemente korrekt benannt".into()
            }
        } else if en {
            format!(
                "{} elements without an accessible name",
                unnamed_interactive
            )
        } else {
            format!("{} Elemente ohne zugänglichen Namen", unnamed_interactive)
        },
    });

    // 3. No critical WCAG violations
    let critical = report.statistics.critical;
    signals.push(QualitySignal {
        name: if en {
            "No critical errors".into()
        } else {
            "Keine kritischen Fehler".into()
        },
        present: critical == 0,
        weight: 0.25,
        detail: if critical == 0 {
            if en {
                "No critical accessibility violations".into()
            } else {
                "Keine kritischen Accessibility-Verstöße".into()
            }
        } else if en {
            format!("{} critical violations", critical)
        } else {
            format!("{} kritische Verstöße", critical)
        },
    });

    // 4. Language consistency
    if let Some(seo) = &report.seo {
        let has_lang = seo.technical.has_lang;
        signals.push(QualitySignal {
            name: if en {
                "Language consistency".into()
            } else {
                "Sprachkonsistenz".into()
            },
            present: has_lang,
            weight: 0.25,
            detail: if has_lang {
                if en {
                    "Language correctly declared".into()
                } else {
                    "Sprache korrekt deklariert".into()
                }
            } else if en {
                "Missing language declaration".into()
            } else {
                "Fehlende Sprachdeklaration".into()
            },
        });
    }

    build_dimension(consistency_name(en), &signals, en)
}

// ─── Consistency (batch / cross-page) ────────────────────────────────────────

fn evaluate_cross_page_consistency(reports: &[AuditReport], en: bool) -> DimensionScore {
    let total = reports.len() as f32;
    let mut signals = Vec::new();

    // 1. Score stability (low standard deviation = consistent)
    let scores: Vec<f32> = reports.iter().map(|r| r.score).collect();
    let mean = scores.iter().sum::<f32>() / total;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / total;
    let std_dev = variance.sqrt();
    let stable = std_dev < 15.0;

    signals.push(QualitySignal {
        name: if en {
            "Score stability".into()
        } else {
            "Score-Stabilität".into()
        },
        present: stable,
        weight: 0.20,
        detail: if en {
            format!(
                "Standard deviation: {:.1}{}",
                std_dev,
                if stable {
                    " (stable)"
                } else {
                    " (inconsistent)"
                }
            )
        } else {
            format!(
                "Standardabweichung: {:.1}{}",
                std_dev,
                if stable {
                    " (stabil)"
                } else {
                    " (inkonsistent)"
                }
            )
        },
    });

    // 2. Meta description coverage
    let with_meta: usize = reports
        .iter()
        .filter(|r| {
            r.seo
                .as_ref()
                .and_then(|s| s.meta.description.as_ref())
                .is_some_and(|d| d.len() >= 50)
        })
        .count();
    let meta_pct = (with_meta as f32 / total * 100.0) as u32;
    signals.push(QualitySignal {
        name: if en {
            "Meta descriptions".into()
        } else {
            "Meta-Beschreibungen".into()
        },
        present: meta_pct >= 90,
        weight: 0.15,
        detail: if en {
            format!("{}% of pages with a meta description", meta_pct)
        } else {
            format!("{}% der Seiten mit Meta-Beschreibung", meta_pct)
        },
    });

    // 3. Schema.org coverage
    let with_schema: usize = reports
        .iter()
        .filter(|r| {
            r.seo
                .as_ref()
                .is_some_and(|s| s.structured_data.has_structured_data)
        })
        .count();
    let schema_pct = (with_schema as f32 / total * 100.0) as u32;
    signals.push(QualitySignal {
        name: if en {
            "Structured data".into()
        } else {
            "Strukturierte Daten".into()
        },
        present: schema_pct >= 80,
        weight: 0.15,
        detail: if en {
            format!("{}% of pages with Schema.org", schema_pct)
        } else {
            format!("{}% der Seiten mit Schema.org", schema_pct)
        },
    });

    // 4. Language declaration coverage
    let with_lang: usize = reports
        .iter()
        .filter(|r| r.seo.as_ref().is_some_and(|s| s.technical.has_lang))
        .count();
    let lang_pct = (with_lang as f32 / total * 100.0) as u32;
    signals.push(QualitySignal {
        name: if en {
            "Language declaration".into()
        } else {
            "Sprachdeklaration".into()
        },
        present: lang_pct >= 95,
        weight: 0.15,
        detail: if en {
            format!("{}% of pages with a language declaration", lang_pct)
        } else {
            format!("{}% der Seiten mit Sprachdeklaration", lang_pct)
        },
    });

    // 5. Security header consistency
    let with_hsts: usize = reports
        .iter()
        .filter(|r| {
            r.security
                .as_ref()
                .is_some_and(|s| s.headers.strict_transport_security.is_some())
        })
        .count();
    let hsts_pct = (with_hsts as f32 / total * 100.0) as u32;
    signals.push(QualitySignal {
        name: if en {
            "HSTS coverage".into()
        } else {
            "HSTS-Abdeckung".into()
        },
        present: hsts_pct >= 95,
        weight: 0.15,
        detail: if en {
            format!("{}% of pages with HSTS", hsts_pct)
        } else {
            format!("{}% der Seiten mit HSTS", hsts_pct)
        },
    });

    // 6. No pages with critical violations
    let pages_with_critical: usize = reports.iter().filter(|r| r.statistics.critical > 0).count();
    let clean_pct = ((total as usize - pages_with_critical) as f32 / total * 100.0) as u32;
    signals.push(QualitySignal {
        name: if en {
            "Error-free pages".into()
        } else {
            "Fehlerfreie Seiten".into()
        },
        present: pages_with_critical == 0,
        weight: 0.20,
        detail: if en {
            format!("{}% of pages without critical errors", clean_pct)
        } else {
            format!("{}% der Seiten ohne kritische Fehler", clean_pct)
        },
    });

    build_dimension(consistency_name(en), &signals, en)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn build_dimension(name: &str, signals: &[QualitySignal], en: bool) -> DimensionScore {
    if signals.is_empty() {
        return DimensionScore {
            name: name.to_string(),
            score: 0,
            label: no_data_label(en),
            signals: vec![],
        };
    }

    // Normalize weights to sum to 1.0
    let total_weight: f32 = signals.iter().map(|s| s.weight).sum();
    let score = if total_weight > 0.0 {
        let raw: f32 = signals
            .iter()
            .map(|s| {
                if s.present {
                    s.weight / total_weight * 100.0
                } else {
                    0.0
                }
            })
            .sum();
        raw.round() as u32
    } else {
        0
    };

    DimensionScore {
        name: name.to_string(),
        score,
        label: score_to_label(score, en),
        signals: signals.to_vec(),
    }
}

fn average_dimensions(dims: &[DimensionScore], name: &str, en: bool) -> DimensionScore {
    if dims.is_empty() {
        return DimensionScore {
            name: name.to_string(),
            score: 0,
            label: no_data_label(en),
            signals: vec![],
        };
    }

    let avg = dims.iter().map(|d| d.score).sum::<u32>() / dims.len() as u32;

    // Merge signals: take the first report's signals as template, show coverage
    let signals = if let Some(first) = dims.first() {
        first.signals.clone()
    } else {
        vec![]
    };

    DimensionScore {
        name: name.to_string(),
        score: avg,
        label: score_to_label(avg, en),
        signals,
    }
}

fn weighted_average(items: &[(u32, u32)]) -> u32 {
    let total_weight: u32 = items.iter().map(|(_, w)| w).sum();
    if total_weight == 0 {
        return 0;
    }
    let sum: u32 = items.iter().map(|(s, w)| s * w).sum();
    (sum as f64 / total_weight as f64).round() as u32
}

fn no_data_label(en: bool) -> String {
    if en { "No data" } else { "Keine Daten" }.to_string()
}

fn score_to_label(score: u32, en: bool) -> String {
    if en {
        match score {
            90..=100 => "Excellent",
            75..=89 => "Good",
            60..=74 => "Needs improvement",
            40..=59 => "Inadequate",
            _ => "Critical",
        }
    } else {
        match score {
            90..=100 => "Sehr gut",
            75..=89 => "Gut",
            60..=74 => "Verbesserungswürdig",
            40..=59 => "Ausbaufähig",
            _ => "Kritisch",
        }
    }
    .to_string()
}

fn empty_analysis(en: bool) -> SourceQualityAnalysis {
    SourceQualityAnalysis {
        score: 0,
        grade: "F".into(),
        substance: DimensionScore {
            name: substance_name(en).to_string(),
            score: 0,
            label: no_data_label(en),
            signals: vec![],
        },
        consistency: DimensionScore {
            name: consistency_name(en).to_string(),
            score: 0,
            label: no_data_label(en),
            signals: vec![],
        },
        authority: DimensionScore {
            name: authority_name(en).to_string(),
            score: 0,
            label: no_data_label(en),
            signals: vec![],
        },
        disclaimer: disclaimer(en),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::InteractiveFinding;
    use crate::audit::AuditReport;
    use crate::audit::ViolationStatistics;
    use crate::cli::WcagLevel;
    use crate::taxonomy::Severity;
    use crate::wcag::WcagResults;

    fn minimal_report() -> AuditReport {
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
            seo: None,
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
    fn test_minimal_report_produces_scores() {
        let report = minimal_report();
        let analysis = analyze_source_quality(&report, "de");
        assert!(analysis.score <= 100);
        assert!(!analysis.disclaimer.is_empty());
        assert!(!analysis.grade.is_empty());
    }

    #[test]
    fn test_empty_batch_returns_zero() {
        let analysis = analyze_source_quality_batch(&[], "de");
        assert_eq!(analysis.score, 0);
    }

    #[test]
    fn test_batch_with_reports() {
        let reports = vec![minimal_report(), minimal_report()];
        let analysis = analyze_source_quality_batch(&reports, "de");
        assert!(analysis.score <= 100);
        assert_eq!(analysis.consistency.name, "Konsistenz");
    }

    #[test]
    fn test_grade_mapping() {
        assert_eq!(module_score_grade(95), "A");
        assert_eq!(module_score_grade(80), "B");
        assert_eq!(module_score_grade(65), "C");
        assert_eq!(module_score_grade(45), "D");
        assert_eq!(module_score_grade(20), "F");
    }

    #[test]
    fn ax_missing_main_prevents_positive_landmark_signal() {
        let mut report = minimal_report();
        report.interactive_findings.push(InteractiveFinding {
            category: "Landmark".to_string(),
            maps_to_finding: None,
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Kein <main>-Landmark gefunden.".to_string(),
            fix_suggestion: None,
        });

        let analysis = analyze_source_quality(&report, "de");
        let signal = analysis
            .substance
            .signals
            .iter()
            .find(|s| s.name == "Semantische Struktur")
            .expect("semantic structure signal");
        assert!(!signal.present);
        assert!(signal.detail.contains("Accessibility Tree"));
    }

    #[test]
    fn test_weighted_average() {
        assert_eq!(weighted_average(&[(100, 50), (0, 50)]), 50);
        assert_eq!(weighted_average(&[(100, 100)]), 100);
        assert_eq!(weighted_average(&[]), 0);
    }

    #[test]
    fn english_locale_has_no_german_chars() {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        // A bare SEO profile so every "missing"/"weak" branch contributes a string.
        let mut report = minimal_report();
        report.score = 50.0;
        report.seo = Some(SeoAnalysis {
            meta: MetaTags::default(),
            headings: HeadingStructure::default(),
            technical: TechnicalSeo::default(),
            social: SocialTags::default(),
            structured_data: StructuredData::default(),
            score: 40,
            content_profile: None,
            robots: None,
            page_health: None,
            serp: None,
            meta_issues: vec![],
            image_efficiency: None,
        });

        let analysis = analyze_source_quality(&report, "en");
        let dims = [
            &analysis.substance,
            &analysis.consistency,
            &analysis.authority,
        ];
        for dim in dims {
            let mut texts = vec![dim.name.clone(), dim.label.clone()];
            for s in &dim.signals {
                texts.push(s.name.clone());
                texts.push(s.detail.clone());
            }
            for t in texts {
                assert!(
                    !t.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                    "German characters in EN output: {t:?}"
                );
            }
        }
        assert!(!analysis
            .disclaimer
            .contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
    }
}
