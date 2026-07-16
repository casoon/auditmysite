use super::*;

pub(in crate::output::pdf) fn render_source_quality(
    mut builder: renderreport::engine::ReportBuilder,
    sq: &crate::source_quality::SourceQualityAnalysis,
    _is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::source_quality::{
        source_quality_dimension_label, source_quality_dimension_name, source_quality_disclaimer,
        source_quality_signal_text,
    };

    // The struct carries canonical English; re-derive everything in the run language.
    let en = i18n.locale() == "en";
    let disclaimer = source_quality_disclaimer(en);

    let sq_title = i18n.t("pdf-sq-section-title");
    builder = builder.add_component(Section::new(&sq_title).with_level(3));
    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), sq.score)
                .with_description(super::score_band_label(sq.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(format!(
                "{}: {}",
                i18n.t("pdf-sq-overview-title"),
                disclaimer
            ))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "source_quality",
            sq.score,
            &disclaimer,
        ));

    if sq.score >= 80 {
        return builder.add_component(Callout::success(i18n.t("pdf-sq-success")));
    }

    for dim in [&sq.substance, &sq.consistency, &sq.authority] {
        let dim_name = source_quality_dimension_name(dim.kind, en);
        let dim_label = source_quality_dimension_label(dim.score, en);
        builder = builder.add_component(Section::new(dim_name).with_level(3));

        builder = builder.add_component(
            ScoreCard::new(
                format!("{} · 0–100", score_quality_label(dim.score)),
                dim.score,
            )
            .with_description(&dim_label)
            .with_thresholds(70, 50),
        );

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ]);

            for signal in &dim.signals {
                let (name, detail) =
                    source_quality_signal_text(signal.kind, signal.present, &signal.values, en);
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![name, status.to_string(), detail]);
            }
            builder = builder.add_component(table);
        }
    }

    builder
}

// ─── Tech Stack ─────────────────────────────────────────────────────────────

pub(in crate::output::pdf) fn render_tech_stack(
    mut builder: renderreport::engine::ReportBuilder,
    ts: &crate::tech_stack::TechStackAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::tech_stack::Confidence;

    let ts_title = i18n.t("pdf-ts-section-title");
    let ts_takeaway = match (ts.detected.is_empty(), i18n.locale() == "en") {
        (true, true) => "No common technologies were detected.".to_string(),
        (true, false) => "Keine gängigen Technologien erkannt.".to_string(),
        (false, true) => format!("{} technologies detected.", ts.detected.len()),
        (false, false) => format!("{} Technologien erkannt.", ts.detected.len()),
    };
    builder = super::module_chapter_opener(builder, &ts_title, &ts_takeaway, is_first);
    builder = builder.add_component(
        ScoreCard::new(super::module_score_caption(i18n), ts.score)
            .with_description(super::score_band_label(ts.score, i18n))
            .with_thresholds(75, 40),
    );

    if !ts.detected.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ts-technology")).with_width("28%"),
            TableColumn::new(i18n.t("pdf-serp-category")).with_width("22%"),
            TableColumn::new("Version").with_width("18%"),
            TableColumn::new(i18n.t("pdf-ts-confidence")).with_width("14%"),
            TableColumn::new("Detail").with_width("18%"),
        ])
        .with_title(i18n.t("pdf-ts-detected-title"));

        for tech in &ts.detected {
            let confidence = match tech.confidence {
                Confidence::High => i18n.t("pdf-ts-confidence-high"),
                Confidence::Medium => i18n.t("pdf-ts-confidence-medium"),
                Confidence::Low => i18n.t("pdf-ts-confidence-low"),
            };
            table = table.add_row(vec![
                tech.name.clone(),
                format!("{:?}", tech.category),
                tech.version.clone().unwrap_or_else(|| "—".to_string()),
                confidence.to_string(),
                tech.signals.join(", "),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !ts.findings.is_empty() {
        let mut findings_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ts-finding")).with_width("35%"),
            TableColumn::new(i18n.t("pdf-ts-severity")).with_width("15%"),
            TableColumn::new("Detail").with_width("50%"),
        ])
        .with_title(i18n.t("pdf-ts-findings-title"));

        for finding in &ts.findings {
            findings_table = findings_table.add_row(vec![
                finding.title.clone(),
                finding.severity.label().to_string(),
                finding.detail.clone(),
            ]);
        }
        builder = builder.add_component(findings_table);
    }

    builder
}

// ─── AI Visibility ──────────────────────────────────────────────────────────

pub(in crate::output::pdf) fn render_ai_visibility(
    mut builder: renderreport::engine::ReportBuilder,
    av: &crate::ai_visibility::AiVisibilityAnalysis,
    _is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::ai_visibility::{
        ai_chunk_recommendation, ai_chunk_section_heading, ai_dimension_label, ai_disclaimer,
        ai_kg_link_suggestion_reason, ai_signal_text,
    };

    // The struct carries canonical English; re-derive everything in the run language.
    let en = i18n.locale() == "en";
    let disclaimer = ai_disclaimer(en);

    let indicator_note_ai = i18n.t("pdf-ai-indicator-note");
    let ai_title = i18n.t("pdf-ai-section-title");
    builder = builder.add_component(Section::new(&ai_title).with_level(3));
    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), av.score)
                .with_description(super::score_band_label(av.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(format!(
                "{}: {}",
                i18n.t("pdf-seo-indicator-title"),
                indicator_note_ai
            ))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(
            Label::new(format!(
                "{}: {}",
                i18n.t("pdf-ai-overview-title"),
                disclaimer
            ))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "ai_visibility",
            av.score,
            &disclaimer,
        ));

    if av.score >= 80 {
        return builder.add_component(Callout::success(i18n.t("pdf-ai-success")));
    }

    // Render each dimension
    for (dim, title) in [
        (&av.readability.dimension, i18n.t("pdf-ai-readability")),
        (&av.citation.dimension, i18n.t("pdf-ai-citability")),
        (&av.chunks.dimension, i18n.t("pdf-ai-tech-readability")),
        (
            &av.knowledge_graph.dimension,
            i18n.t("pdf-seo-profile-structured-data"),
        ),
        (&av.policy.dimension, i18n.t("pdf-ai-policy")),
    ] {
        builder = builder.add_component(Section::new(title).with_level(3));
        let dim_label = ai_dimension_label(dim.score, en);
        let mut dim_kv = KeyValueList::new().add(
            i18n.t("label-heuristic-indicator"),
            format!("~{} / 100 — {}", dim.score, score_quality_label(dim.score)),
        );
        if !dim_label.is_empty() {
            dim_kv = dim_kv.add("Basis", &dim_label);
        }
        builder = builder.add_component(dim_kv);

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ]);

            for signal in dim.signals.iter().take(5) {
                let (name, detail) =
                    ai_signal_text(signal.kind, signal.present, &signal.values, en);
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![name, status.to_string(), detail]);
            }
            builder = builder.add_component(table);
            if dim.signals.len() > 5 {
                let more_note = i18n.t_args(
                    "pdf-ai-more-signals",
                    &[("count", (dim.signals.len() - 5).to_string())],
                );
                builder = builder.add_component(Callout::info(&more_note));
            }
        }
    }

    // Chunk sections summary
    if !av.chunks.sections.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-content-sections")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-section-col")).with_width("70%"),
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(i18n.t("pdf-ai-words-col")).with_width("15%"),
        ])
        .with_title(i18n.t("pdf-ai-sections-title"));

        for section in &av.chunks.sections {
            // Synthetic headings re-derive in the run language; real headings pass through.
            let heading = match section.heading_kind {
                Some(kind) => ai_chunk_section_heading(kind, en),
                None => section.heading.clone(),
            };
            table = table.add_row(vec![
                heading,
                format!("H{}", section.level),
                section.word_count.to_string(),
            ]);
        }
        builder = builder.add_component(table);
        let recommendation = ai_chunk_recommendation(
            av.chunks.recommendation_kind,
            av.chunks.recommendation_counts,
            en,
        );
        builder = builder
            .add_component(Callout::info(&recommendation).with_title(i18n.t("pdf-ai-rec-title")));
    }

    // Knowledge graph entities
    if !av.knowledge_graph.entities.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-detected-entities")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-entity-col")),
            TableColumn::new(i18n.t("pdf-perf-min-type")),
            TableColumn::new(i18n.t("pdf-seo-ie-source")),
        ])
        .with_title(i18n.t("pdf-ai-entities-title"));

        for entity in &av.knowledge_graph.entities {
            table = table.add_row(vec![
                &entity.name,
                &entity.entity_type,
                &entity.source.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Knowledge graph relationships
    if !av.knowledge_graph.relationships.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-subject-col")),
            TableColumn::new(i18n.t("pdf-ai-relation-col")),
            TableColumn::new(i18n.t("pdf-ai-object-col")),
        ])
        .with_title(i18n.t("pdf-ai-relations-title"));

        for rel in &av.knowledge_graph.relationships {
            table = table.add_row(vec![&rel.subject, &rel.predicate, &rel.object]);
        }
        builder = builder.add_component(table);
    }

    // Link suggestions
    if !av.knowledge_graph.link_suggestions.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-link-suggestions")).with_level(3));

        use crate::ai_visibility::KgSuggestionKind;
        let mut list = List::new();
        for suggestion in &av.knowledge_graph.link_suggestions {
            // Re-derive the reason (and synthetic entity label) in the run language.
            let reason = ai_kg_link_suggestion_reason(
                suggestion.kind,
                &suggestion.entity,
                suggestion.internal_links.unwrap_or(0),
                en,
            );
            let entity = match suggestion.kind {
                KgSuggestionKind::FewInternalLinks => {
                    if en {
                        "Page".to_string()
                    } else {
                        "Seite".to_string()
                    }
                }
                KgSuggestionKind::TopicOnlyHeading => suggestion.entity.clone(),
            };
            list = list.add_item(format!("{}: {}", entity, reason));
        }
        builder = builder.add_component(list);
    }

    // AI Policy details
    if av.policy.blocks_all {
        builder = builder.add_component(
            Callout::warning(i18n.t("pdf-ai-policy-blocks-all-body"))
                .with_title(i18n.t("pdf-ai-policy-blocks-all-title")),
        );
    } else if av.policy.blocks_ai_citation {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-ai-policy-limited-title"));
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add("Status", i18n.t("pdf-ai-policy-limited-body"));
        builder = builder.add_component(kv);
    } else if av.policy.blocks_ai_training {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-ai-policy"));
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add("Status", i18n.t("pdf-ai-policy-training-body"));
        builder = builder.add_component(kv);
    }

    builder
}

// ─── Content Visibility & Trust ─────────────────────────────────────────────

pub(in crate::output::pdf) fn render_content_visibility(
    mut builder: renderreport::engine::ReportBuilder,
    cv: &crate::content_visibility::ContentVisibilityAnalysis,
    _is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::assessment::AssessmentLevel;
    use crate::content_visibility::content_visibility_signal_text;

    // The struct carries canonical English; re-derive title/detail in the run language.
    let en = i18n.locale() == "en";

    // Localize a signal's title/detail via its stored kind + values, falling back
    // to the canonical-English struct strings for any signal without a kind.
    let localized = |s: &crate::assessment::ContentSignal| -> (String, String) {
        match s.cv_kind {
            Some(kind) => content_visibility_signal_text(kind, &s.cv_values, en),
            None => (s.title.clone(), s.detail.clone()),
        }
    };

    let cv_title = i18n.t("pdf-cv-section-title");
    let score = (cv.signal_count.saturating_sub(cv.problem_count) * 100)
        .checked_div(cv.signal_count)
        .unwrap_or(100) as u32;

    builder = builder.add_component(Section::new(&cv_title).with_level(3));

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), score)
                .with_description(super::score_band_label(score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(module_customer_context(
            i18n,
            "content_visibility",
            score,
            &i18n.t("pdf-cv-overview-body"),
        ))
        .add_component(TextBlock::new(i18n.t_args(
            "pdf-cv-signals-analyzed",
            &[
                ("signals", cv.signal_count.to_string()),
                ("problems", cv.problem_count.to_string()),
            ],
        )));

    let areas: Vec<(String, &[crate::assessment::ContentSignal])> = vec![
        (
            i18n.t("pdf-cv-area-organic-visibility"),
            &cv.organic_visibility,
        ),
        (i18n.t("pdf-cv-area-local-business"), &cv.local_business),
        (i18n.t("pdf-cv-area-eeat"), &cv.eeat),
        (i18n.t("pdf-cv-area-content-depth"), &cv.content_depth),
        (
            i18n.t("pdf-cv-area-topical-authority"),
            &cv.topical_authority,
        ),
    ];

    let mut not_testable_rows: Vec<ChecklistRow> = Vec::new();

    for (area_name, signals) in &areas {
        let visible: Vec<_> = signals
            .iter()
            .filter(|s| s.level != AssessmentLevel::NotTestable)
            .collect();

        // Collect NotTestable signals across all areas
        for s in signals
            .iter()
            .filter(|s| s.level == AssessmentLevel::NotTestable)
        {
            let (title, detail) = localized(s);
            not_testable_rows.push(ChecklistRow::new(&title, &detail).with_status("info"));
        }

        if visible.is_empty() {
            continue;
        }

        builder = builder.add_component(Section::new(area_name.clone()).with_level(3));

        for signal in visible {
            let conf_prefix = match signal.confidence {
                crate::assessment::EvidenceConfidence::High => "● ",
                crate::assessment::EvidenceConfidence::Medium => "◐ ",
                crate::assessment::EvidenceConfidence::Low => "○ ",
            };
            let (title, detail) = localized(signal);
            let body = format!("{}{}", conf_prefix, detail);

            builder = builder.add_component(match signal.level {
                AssessmentLevel::Pass | AssessmentLevel::Positive => {
                    Callout::success(&body).with_title(&title)
                }
                AssessmentLevel::Warning => Callout::warning(&body).with_title(&title),
                AssessmentLevel::Violation => Callout::warning(&body).with_title(&title),
                AssessmentLevel::NotTestable => unreachable!(),
            });

            if !signal.evidence.is_empty() {
                let mut kv = KeyValueList::new();
                for ev in &signal.evidence {
                    let mut detail = String::new();
                    if let Some(ref fp) = ev.field_path {
                        detail.push_str(fp);
                    }
                    if let Some(ref val) = ev.value_excerpt {
                        if !detail.is_empty() {
                            detail.push_str(": ");
                        }
                        detail.push_str(val);
                    }
                    // No real field_path/value_excerpt to show — omit the row
                    // entirely rather than falling back to the debug-derived
                    // source label as both key and value (e.g. "HttpHeader
                    // HttpHeader", zero added information).
                    if detail.is_empty() {
                        continue;
                    }
                    let source_label = format!("{:?}", ev.source);
                    kv = kv.add(&source_label, &detail);
                }
                if !kv.items.is_empty() {
                    builder = builder.add_component(kv);
                }
            }
        }
    }

    if !not_testable_rows.is_empty() {
        let title = i18n.t("pdf-cv-manual-review-title");
        builder = builder
            .add_component(Section::new(&title).with_level(3))
            .add_component(ChecklistPanel::new(not_testable_rows).with_title(&title));
    }

    builder
}

pub(in crate::output::pdf) fn render_best_practices(
    mut builder: renderreport::engine::ReportBuilder,
    bp: &crate::best_practices::BestPracticesAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let bp_clean =
        bp.console_errors.error_count == 0 && !bp.vulnerable_libraries.has_vulnerabilities;
    let bp_takeaway = match (bp_clean, i18n.locale() == "en") {
        (true, true) => "No console errors or known-vulnerable libraries detected.",
        (true, false) => "Keine Konsolenfehler oder bekannten verwundbaren Bibliotheken erkannt.",
        (false, true) => "Console errors or vulnerable libraries were detected — see below.",
        (false, false) => "Konsolenfehler oder verwundbare Bibliotheken erkannt — siehe unten.",
    };
    builder = super::module_chapter_opener(builder, "Best Practices", bp_takeaway, is_first);
    builder = builder.add_component(
        ScoreCard::new(super::module_score_caption(i18n), bp.score)
            .with_description(super::score_band_label(bp.score, i18n))
            .with_thresholds(75, 40),
    );

    if bp.score >= 90
        && bp.console_errors.error_count == 0
        && !bp.vulnerable_libraries.has_vulnerabilities
    {
        return builder.add_component(TextBlock::new(i18n.t("pdf-bp-success")));
    }

    // Console errors
    if bp.console_errors.error_count > 0 {
        let title = i18n.t("pdf-bp-console-errors-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(i18n.t("pdf-bp-message-col")).with_width("85%"),
        ])
        .with_title(&title);
        for error in &bp.console_errors.errors {
            table = table.add_row(vec![error.level.clone(), error.message.clone()]);
        }
        builder = builder.add_component(table);
    }

    // Vulnerable libraries
    if bp.vulnerable_libraries.has_vulnerabilities {
        let title = i18n.t("pdf-bp-vuln-libs-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-bp-lib-col")).with_width("20%"),
            TableColumn::new("Version").with_width("15%"),
            TableColumn::new(i18n.t("pdf-bp-severity-col")).with_width("15%"),
            TableColumn::new(i18n.t("pdf-ph-issue")).with_width("35%"),
            TableColumn::new(i18n.t("pdf-bp-fix-col")).with_width("15%"),
        ])
        .with_title(&title);
        for lib in &bp.vulnerable_libraries.vulnerable {
            table = table.add_row(vec![
                lib.name.clone(),
                lib.version.clone(),
                lib.severity.clone(),
                lib.description.clone(),
                lib.safe_version.clone(),
            ]);
        }
        builder = builder.add_component(table);
    } else if !bp.vulnerable_libraries.detected.is_empty() {
        builder = builder.add_component(TextBlock::new(i18n.t("pdf-bp-libs-up-to-date")));
    }

    builder
}
