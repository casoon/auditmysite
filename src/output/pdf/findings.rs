//! Finding renderers for PDF reports.

use crate::output::localized::is_english;
use fluent_bundle::FluentValue;
use renderreport::components::advanced::{Divider, KeyValueList, List, WrongRightBlock};
use renderreport::components::text::Label;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;
use crate::wcag::Evidence;

use super::helpers::manual_recheck_instruction;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

/// Extract the first sentence from a text (up to first period + space, or full text).
/// Skips common German abbreviations like "z. B.", "d. h.", "u. a.".
/// Compact an HTML/selector evidence string for table cells: collapse
/// whitespace and replace `data:` URI bodies (huge unbreakable tokens that blow
/// up row heights) with `data:…`, then length-truncate. Keeps the meaningful
/// markup (tag, classes) while dropping the inline-asset noise.
pub(super) fn compact_html(s: &str, max: usize) -> String {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::with_capacity(collapsed.len());
    let mut rest = collapsed.as_str();
    while let Some(pos) = rest.find("data:") {
        result.push_str(&rest[..pos]);
        result.push_str("data:…");
        rest = &rest[pos + "data:".len()..];
        // Skip the URI body up to the closing attribute quote.
        if let Some(q) = rest.find(['"', '\'']) {
            rest = &rest[q..];
        } else {
            rest = "";
        }
    }
    result.push_str(rest);
    truncate_url(&result, max)
}

/// Abbreviations followed by ". " inside a sentence.
const SENTENCE_INNER_ABBREVIATIONS: &[&str] = &["bzw", "ggf", "inkl", "evtl", "vgl", "ca"];

pub(super) fn first_sentence(text: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(". ") {
        let pos = search_from + rel;
        // Check for single-letter abbreviation pattern: " X. " (e.g. "z. B.", "d. h.")
        if pos >= 2 {
            let bytes = text.as_bytes();
            let b0 = bytes[pos - 2];
            let b1 = bytes[pos - 1];
            if b0 == b' ' && b1.is_ascii_alphabetic() {
                search_from = pos + 2;
                continue;
            }
        }
        // Multi-letter abbreviations ("Skripte bzw. -Styles") end no
        // sentence either.
        let word = text[..pos]
            .rsplit(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("");
        if SENTENCE_INNER_ABBREVIATIONS.contains(&word) {
            search_from = pos + 2;
            continue;
        }
        return &text[..pos + 1];
    }
    text
}

#[cfg(test)]
fn customer_perspective_title(group: &FindingGroup, en: bool) -> String {
    let rule = group.finding.rule_id.as_str();
    let dimension = group.finding.dimension.as_str();
    let subcategory = group.finding.subcategory.as_str();
    let haystack = format!(
        "{} {} {} {}",
        rule,
        dimension,
        subcategory,
        group.title.to_lowercase()
    );

    let key = if haystack.contains("name_role")
        || haystack.contains("focus")
        || haystack.contains("keyboard")
        || haystack.contains("button")
        || haystack.contains("link_purpose")
        || haystack.contains("target")
        || haystack.contains("form")
    {
        "pdf-findings-perspective-operability"
    } else if haystack.contains("structure")
        || haystack.contains("heading")
        || haystack.contains("landmark")
        || haystack.contains("region")
        || haystack.contains("bypass")
        || haystack.contains("parsing")
        || haystack.contains("semant")
    {
        "pdf-findings-perspective-structure"
    } else if haystack.contains("contrast")
        || haystack.contains("color")
        || haystack.contains("image")
        || haystack.contains("alt")
        || haystack.contains("visibility")
    {
        "pdf-findings-perspective-perception"
    } else if dimension == "SEO"
        || haystack.contains("seo")
        || haystack.contains("schema")
        || haystack.contains("meta")
        || haystack.contains("ai")
    {
        "pdf-findings-perspective-discoverability"
    } else if dimension == "Security" || haystack.contains("security") {
        "pdf-findings-perspective-trust"
    } else if dimension == "Performance" || dimension == "Mobile" {
        "pdf-findings-perspective-loading"
    } else {
        "pdf-findings-perspective-customer-impact"
    };
    crate::i18n::text(key, en)
}

/// Label for the "BFSG/EAA relevance" meta row. Both acronyms are spelled out
/// once, at their first appearance in the report, then abbreviated on every
/// later card — readers who start at Chapter 02's first finding get the
/// expansion, later cards stay compact (Rule B, plain-language report plan).
fn bfsg_eaa_label(i18n: &I18n, acronyms_expanded: &mut bool) -> String {
    if !*acronyms_expanded {
        *acronyms_expanded = true;
        return i18n.t("pdf-findings-bfsg-eaa-expanded");
    }
    i18n.t("pdf-findings-bfsg-eaa")
}

pub(super) fn report_code_label(value: &str, i18n: &I18n) -> String {
    let key = match value {
        "very_high" => "pdf-findings-code-very-high",
        "high" => "severity-high",
        "medium" => "severity-medium",
        "low" => "severity-low",
        "very_low" => "pdf-findings-code-very-low",
        "immediate" => "pdf-findings-code-immediate",
        "quick_win" => "pdf-findings-code-quick-win",
        "normal" => "pdf-findings-code-normal",
        "manual_review_recommended" => "pdf-findings-code-manual-review-recommended",
        "automatically_confirmed" => "pdf-findings-code-automatically-confirmed",
        _ => return "n/a".to_string(),
    };
    i18n.t(key)
}

/// The origin of a finding's markup as one neutral line, or `None` for the
/// main document (#1109): which kind of frame it sits in, the known
/// provider(s) and where the fix belongs. A label, not a verdict: the
/// finding keeps its place in the report and the page is not called
/// conformant by it.
///
/// A cross-origin frame of an unknown provider names the host of its source
/// (the iframe's `src`, read from the listed occurrences) instead.
pub(super) fn origin_text(
    f: &crate::audit::normalized::NormalizedFinding,
    i18n: &I18n,
) -> Option<String> {
    use crate::model::FindingOrigin as O;
    let en = is_english(i18n);
    let mut hosts: Vec<&str> = f
        .occurrences
        .iter()
        .filter(|o| o.origin == O::CrossOriginFrame)
        .filter_map(|o| o.frame_host.as_deref())
        .collect();
    hosts.sort_unstable();
    hosts.dedup();
    let (key, list) = match (f.origin, f.providers.is_empty(), hosts.is_empty()) {
        (O::CrossOriginFrame, false, _) => {
            ("pdf-findings-origin-cross-origin", f.providers.join(", "))
        }
        (O::CrossOriginFrame, true, false) => {
            ("pdf-findings-origin-cross-origin-source", hosts.join(", "))
        }
        (O::CrossOriginFrame, true, true) => {
            ("pdf-findings-origin-cross-origin-unknown", String::new())
        }
        (O::SameSiteFrame, ..) => ("pdf-findings-origin-same-site", String::new()),
        (O::Mixed, false, _) => ("pdf-findings-origin-mixed-provider", f.providers.join(", ")),
        (O::Mixed, true, _) => ("pdf-findings-origin-mixed", String::new()),
        _ => return None,
    };
    Some(crate::i18n::text_args(
        key,
        &[("list", FluentValue::from(list))],
        en,
    ))
}

/// Small tracked-caps section caption inside a finding card — the same
/// visual language as the report's chapter eyebrows, scaled down. Used to
/// separate the finding card's secondary blocks (QA metadata, action
/// assessment) from each other without wrapping each one in its own frame.
fn section_caption(text: &str) -> Label {
    Label::new(text.to_uppercase())
        .bold()
        .with_size("8.5pt")
        .with_color(super::design::tokens::NEUTRAL)
}

/// Find a specific `computed`/`ax_tree` evidence value by field name.
fn evidence_value<'a>(evidence: &'a [Evidence], source: &str, field: &str) -> Option<&'a str> {
    evidence
        .iter()
        .find(|e| e.source == source && e.field.as_deref() == Some(field))
        .and_then(|e| e.value.as_deref())
        .filter(|v| !v.is_empty())
}

/// "Contrast 2.70:1 (required 4.5:1)" — combines the contrast rule's
/// `computed` evidence into one localized measured-value line. `None` when
/// the occurrence carries no computed contrast evidence (every rule other
/// than `contrast/text-*`).
fn contrast_measured_text(evidence: &[Evidence], en: bool) -> Option<String> {
    // `a11y-rules` liefert die Zahl allein (`2.56`), die fruehere lokale
    // Regel mit `:1`.
    let as_ratio = |v: &str| {
        let v = decimal(v, en);
        if v.ends_with(":1") {
            v
        } else {
            format!("{v}:1")
        }
    };
    let ratio = as_ratio(evidence_value(evidence, "computed", "contrast_ratio")?);
    let required = evidence_value(evidence, "computed", "required_ratio").map(as_ratio);
    Some(match required {
        Some(req) => crate::i18n::text_args(
            "pdf-findings-contrast-required",
            &[
                ("ratio", FluentValue::from(ratio)),
                ("required", FluentValue::from(req)),
            ],
            en,
        ),
        None => crate::i18n::text_args(
            "pdf-findings-contrast",
            &[("ratio", FluentValue::from(ratio))],
            en,
        ),
    })
}

/// The opacity a contrast occurrence was measured at, as written in the
/// evidence (`0.55`), when the shared rule marks the ratio as state-dependent
/// (casoon/barrierlab#261), together with the composited text colour.
fn measured_opacity(evidence: &[Evidence]) -> Option<(&str, Option<&str>)> {
    if evidence_value(evidence, "computed", "state_dependent") != Some("true") {
        return None;
    }
    let opacity = evidence_value(evidence, "computed", "opacity")?;
    Some((
        opacity,
        evidence_value(evidence, "computed", "effective_foreground"),
    ))
}

/// A measured number as written in the evidence (`2.56`), with the decimal
/// comma in German (#902).
fn decimal(value: &str, en: bool) -> String {
    if en {
        value.to_string()
    } else {
        value.replace('.', ",")
    }
}

/// "0.55: the ratio applies to this state (text colour after opacity
/// #8c8c8c)" for a contrast occurrence measured under `opacity` below 1
/// (#1112). `None` for every other occurrence.
fn opacity_text(evidence: &[Evidence], en: bool) -> Option<String> {
    let (opacity, color) = measured_opacity(evidence)?;
    Some(crate::i18n::text_args(
        "pdf-findings-opacity-value",
        &[
            ("opacity", FluentValue::from(decimal(opacity, en))),
            ("color", FluentValue::from(color.unwrap_or("–").to_string())),
        ],
        en,
    ))
}

/// The cross-page line of a contrast finding whose sampled occurrences were
/// measured under reduced opacity, with the distinct opacity values (#1112):
/// the site owner sees that opacity, not the colour, sets the ratio. `None`
/// when no sampled occurrence carries the state-dependent evidence.
pub(super) fn opacity_pattern_text(
    occurrences: &[crate::audit::normalized::OccurrenceDetail],
    en: bool,
) -> Option<String> {
    let mut values: Vec<&str> = occurrences
        .iter()
        .filter_map(|occ| measured_opacity(&occ.evidence).map(|(o, _)| o))
        .collect();
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        return None;
    }
    let values = values
        .iter()
        .map(|v| decimal(v, en))
        .collect::<Vec<_>>()
        .join(", ");
    Some(crate::i18n::text_args(
        "pdf-findings-opacity-pattern",
        &[("values", FluentValue::from(values))],
        en,
    ))
}

pub(super) fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    report_ts: i64,
    evidence_seq: &mut usize,
    acronyms_expanded: &mut bool,
) -> renderreport::engine::ReportBuilder {
    let color = super::design::severity_color(group.finding.severity);

    // The evidence entry of a finding (#1209): the analysis card in part 1
    // told its problem, impact, fix, priority and cause; this entry carries
    // only what proves it, so none of those texts is printed a second time.
    builder = add_title(builder, group, i18n, color);
    if let Some(origin) = origin_text(&group.finding, i18n) {
        builder =
            builder.add_component(Callout::info(origin).with_title(i18n.t("pdf-findings-origin")));
    }
    builder = add_qa_metadata(builder, group, i18n, acronyms_expanded);
    builder = add_action_assessment(builder, group, i18n);

    // Findings with no real DOM evidence (e.g. SEO heading findings, whose
    // `OccurrenceDetail` has no selector at all — see `aggregate_seo_findings`)
    // fall back to using the internal issue-type token as both `node_id` and
    // `selector`, which is how `build_location_hints`'s fallback resolves it.
    // Applying DOM-evidence components (element-type extraction, a "location"
    // table/snapshot, selector-pattern clustering) to that token produces
    // nonsense like "Element-Typen: 1× multiple" or "Fundstelle: multiple_h1" —
    // skip all of them when no occurrence actually has real location data.
    // The bare accessibility-tree number a screen-reader finding carries as
    // its selector is no location either (#1211).
    let has_real_location = group.representative_occurrences.iter().any(|occ| {
        occ.selector != occ.node_id && !occ.selector.chars().all(|c| c.is_ascii_digit())
    });

    // The validator's "selector" is a source position (`190:12271`) and its
    // node id the rule key, not an element: the element type comes from the
    // message when it names one, the position is shown as line and column.
    let is_html_conform = group.finding.rule_id.starts_with("html_conform.");

    // AffectedElements: element-type summary
    if has_real_location {
        builder = add_element_types(builder, group, i18n, is_html_conform);
    }

    builder = add_examples(builder, group, i18n);
    builder = add_affected_urls(builder, group, i18n);

    if has_real_location {
        builder = add_occurrences(
            builder,
            group,
            i18n,
            is_html_conform,
            report_ts,
            evidence_seq,
        );
    }

    builder.add_component(Divider {
        style: "solid".to_string(),
        thickness: "0pt".to_string(),
        color: Some("#ffffff".to_string()),
        spacing_above: "15pt".to_string(),
        spacing_below: "0pt".to_string(),
    })
}

/// "Critical — Title — WCAG 1.1.1 (A)": the severity prefix, the title and the
/// criterion with its level, or the title / rule id alone without a criterion.
fn finding_header(group: &FindingGroup, i18n: &I18n) -> String {
    let severity_prefix = format!(
        "{} ",
        i18n.t(match group.finding.severity {
            crate::wcag::Severity::Critical => "pdf-findings-severity-critical",
            crate::wcag::Severity::High => "pdf-findings-severity-high",
            crate::wcag::Severity::Medium => "pdf-findings-severity-medium",
            crate::wcag::Severity::Low => "pdf-findings-severity-low",
        })
    );

    if !group.finding.wcag_criterion.is_empty() {
        // Flag criteria WCAG 2.2 added (e.g. 2.5.8) inline — they are inside
        // this report's audit-scope ratio and EN 301 549 V4.1.1, but were not
        // part of WCAG 2.1 / EN 301 549 V3.2.1. A finding tagged just "AA"
        // would hide that (#572).
        let wcag22_suffix = if crate::wcag::coverage::is_wcag22_only(&group.finding.wcag_criterion)
        {
            ", WCAG 2.2"
        } else {
            ""
        };
        format!(
            "{}{} — WCAG {} ({}{})",
            severity_prefix,
            group.title,
            group.finding.wcag_criterion,
            group.finding.wcag_level,
            wcag22_suffix
        )
    } else if group.title.is_empty() {
        // Raw internal rule ids (e.g. "seo.headings.multiple_h1") are never
        // customer-facing prose — only fall back to one when there's truly no
        // title to show at all, and even then, don't also glue it onto a
        // perfectly good title as an unexplained suffix (#QA-039 report review).
        format!("{}{}", severity_prefix, group.finding.rule_id)
    } else {
        format!("{}{}", severity_prefix, group.title)
    }
}

fn add_title(
    builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    color: &str,
) -> renderreport::engine::ReportBuilder {
    builder.add_component(
        Label::new(finding_header(group, i18n))
            .bold()
            .with_size("14pt")
            .with_color(color),
    )
}

/// Secondary tier: audit-methodology metadata a developer doesn't need to
/// read on every finding — kept compact and visually subordinate (a small
/// caption, not a titled card) rather than continuing the primary strip's
/// weight.
fn add_qa_metadata(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    acronyms_expanded: &mut bool,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(section_caption(&i18n.t("pdf-findings-qa-metadata")));
    let mut meta_kv = KeyValueList::new()
        .add(i18n.t("pdf-findings-finding-id"), &group.finding.id)
        .add(
            i18n.t("pdf-findings-implementation-priority"),
            report_code_label(&group.finding.remediation_priority, i18n),
        )
        .add(
            i18n.t("pdf-findings-detection-confidence"),
            report_code_label(&group.finding.confidence, i18n),
        )
        .add(
            i18n.t("pdf-findings-false-positive-risk"),
            report_code_label(&group.finding.false_positive_risk, i18n),
        )
        .add(
            bfsg_eaa_label(i18n, acronyms_expanded),
            report_code_label(&group.finding.bfsg_relevance, i18n),
        )
        .add(
            i18n.t("finding-occurrences"),
            group.finding.occurrence_count.to_string(),
        );
    // The checker's rule id of an HTML conformance finding is no report text:
    // its title names the rule family (#902).
    if !group.finding.rule_id.starts_with("html_conform.") {
        meta_kv = meta_kv.add(i18n.t("pdf-findings-rule"), &group.finding.rule_id);
    }
    if let Some(url) = group
        .finding
        .help_url
        .as_ref()
        .or(group.finding.understanding_url.as_ref())
        .map(String::as_str)
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        meta_kv = meta_kv.add(i18n.t("finding-reference"), url);
    }
    builder.add_component(meta_kv)
}

fn add_action_assessment(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    // `expected_impact`/`complexity_reason` are re-derived here from the
    // stable `..._kind` in the run language rather than printed from the
    // `FindingGroup` string fields verbatim — those stay canonical English
    // (mirroring the JSON), so printing them directly leaked raw English
    // prose into otherwise fully German reports (#406).
    let expected_impact =
        crate::audit::normalized::expected_impact_text(&group.finding.expected_impact_kind, en);
    let complexity_reason =
        crate::audit::normalized::complexity_text(group.finding.complexity_kind, en);
    // The 1.3.1 self-test is about headings and landmarks; the content-model
    // finding is about invalid children of lists and tables, so it does not
    // apply there (#902).
    let recheck_instruction = if group.finding.rule_id.contains("html-content-model") {
        None
    } else {
        manual_recheck_instruction(&group.finding.wcag_criterion, en)
    };
    if !expected_impact.is_empty() || !complexity_reason.is_empty() || recheck_instruction.is_some()
    {
        builder = builder.add_component(section_caption(&i18n.t("pdf-findings-action-assessment")));
        let mut assessment = KeyValueList::new();
        if !expected_impact.is_empty() {
            assessment = assessment.add(i18n.t("pdf-findings-expected-effect"), &expected_impact);
        }
        if !complexity_reason.is_empty() {
            assessment =
                assessment.add(i18n.t("pdf-findings-complexity-reason"), &complexity_reason);
        }
        if group.finding.verification == "manual_review_recommended" {
            assessment = assessment.add(
                i18n.t("pdf-findings-verification"),
                i18n.t("pdf-findings-code-manual-review-recommended"),
            );
        }
        if let Some(instruction) = recheck_instruction {
            assessment = assessment.add(
                i18n.t("pdf-findings-self-test"),
                // The row key already says "self-test".
                instruction
                    .strip_prefix("Selbsttest: ")
                    .or_else(|| instruction.strip_prefix("Self-check: "))
                    .unwrap_or(&instruction),
            );
        }
        builder = builder.add_component(assessment);
    }
    builder
}

/// The selector of an occurrence as a location: line and column for a
/// validator finding, the compacted selector otherwise.
fn location_label(selector: &str, is_html_conform: bool, i18n: &I18n) -> String {
    match is_html_conform
        .then(|| crate::html_conform::message::parse_location(selector))
        .flatten()
    {
        Some((line, column)) => i18n.t_args(
            "finding-line-column",
            &[("line", line as i64), ("column", column as i64)],
        ),
        None => compact_html(selector, 48),
    }
}

/// Count occurrences per element type.
fn add_element_types(
    builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    is_html_conform: bool,
) -> renderreport::engine::ReportBuilder {
    let mut type_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for occ in &group.representative_occurrences {
        let tag = if is_html_conform {
            crate::html_conform::message::element_name(&occ.message).unwrap_or_default()
        } else {
            extract_element_type(&occ.selector).to_string()
        };
        if !tag.is_empty() {
            *type_counts.entry(tag).or_insert(0) += 1;
        }
    }
    if type_counts.is_empty() {
        return builder;
    }
    let summary = type_counts
        .iter()
        .map(|(tag, count)| format!("{}× {}", count, tag))
        .collect::<Vec<_>>()
        .join("  ·  ");
    builder.add_component(KeyValueList::new().add(i18n.t("finding-element-types"), summary))
}

fn add_examples(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    for example in &group.examples {
        builder = builder.add_component(
            WrongRightBlock::new(&example.bad, &example.good)
                .code()
                .with_labels(i18n.t("finding-wrong"), i18n.t("finding-right")),
        );
        if let Some(ref dec) = example.decorative {
            builder =
                builder.add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
        }
    }
    builder
}

fn add_affected_urls(
    builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if group.affected_urls.is_empty() || group.affected_urls.len() > 10 {
        return builder;
    }
    let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
    for url in &group.affected_urls {
        url_list = url_list.add_item(truncate_url(url, 70));
    }
    builder.add_component(url_list)
}

/// The examples of a finding with their count ("2 examples of 7
/// occurrences"), each as one snapshot: location, note, excerpt, DOM path and
/// measurement, then its evidence screenshot and suggested fix. The snapshot
/// carries what a table of the same rows would say, so there is no table
/// (#1211).
fn add_occurrences(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
    is_html_conform: bool,
    report_ts: i64,
    evidence_seq: &mut usize,
) -> renderreport::engine::ReportBuilder {
    let shown = group.representative_occurrences.len();
    let total = group.finding.occurrence_count;
    let heading = if shown < total {
        i18n.t_args(
            "pdf-findings-examples-of",
            &[("shown", shown as i64), ("total", total as i64)],
        )
    } else {
        i18n.t("finding-occurrences")
    };
    builder = builder.add_component(section_caption(&heading));
    for occ in &group.representative_occurrences {
        builder = builder.add_component(occurrence_snapshot(occ, i18n, is_html_conform));
        builder = add_evidence_screenshot(builder, occ, i18n, report_ts, evidence_seq);
        builder = add_suggested_fix(builder, occ, i18n);
    }
    builder
}

fn occurrence_snapshot(
    occ: &RepresentativeOccurrence,
    i18n: &I18n,
    is_html_conform: bool,
) -> SummaryBox {
    let en = is_english(i18n);
    let mut snapshot = SummaryBox::new(format!(
        "{}: {}",
        i18n.t("finding-occurrence"),
        if is_html_conform {
            location_label(&occ.selector, is_html_conform, i18n)
        } else {
            compact_html(&occ.selector, 60)
        }
    ));
    // The node id is an internal handle (accessibility-tree number,
    // the rule key of a validator finding, "document"): only a real
    // identifier is worth a row (#902).
    let node_id = occ.node_id.trim();
    let internal_handle =
        node_id.is_empty() || node_id == "document" || node_id.chars().all(|c| c.is_ascii_digit());
    if !is_html_conform && !internal_handle {
        snapshot = snapshot.add_item("Node", truncate_url(&occ.node_id, 75));
    }
    snapshot = snapshot.add_item(i18n.t("finding-note"), first_sentence(&occ.message));

    if let Some(html) = occ
        .html_snippet
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        snapshot = snapshot.add_item("HTML", compact_html(html, 110));
    }

    if let Some(path) = evidence_value(&occ.evidence, "ax_tree", "dom_path") {
        snapshot = snapshot.add_item(i18n.t("pdf-findings-dom-path"), compact_html(path, 90));
    }

    if let Some(measured) = contrast_measured_text(&occ.evidence, en) {
        snapshot = snapshot.add_item(i18n.t("pdf-findings-measured-value"), measured);
    }

    if let Some(opacity) = opacity_text(&occ.evidence, en) {
        snapshot = snapshot.add_item(i18n.t("pdf-findings-opacity"), opacity);
    }

    if crate::wcag::types::evidence_is_dark_color_scheme(&occ.evidence) {
        snapshot = snapshot.add_item(
            i18n.t("pdf-findings-color-scheme"),
            i18n.t("pdf-findings-color-scheme-dark"),
        );
    }
    snapshot
}

/// Evidence-grade findings: a cropped, highlighted screenshot of
/// the element behind this occurrence, if one was captured during
/// the audit (single-URL PDF only, capped at MAX_ELEMENT_CROPS
/// report-wide — see `PipelineConfig.capture_element_evidence`).
fn add_evidence_screenshot(
    mut builder: renderreport::engine::ReportBuilder,
    occ: &RepresentativeOccurrence,
    i18n: &I18n,
    report_ts: i64,
    evidence_seq: &mut usize,
) -> renderreport::engine::ReportBuilder {
    if let Some(bytes) = &occ.evidence_screenshot {
        let temp_path =
            std::env::temp_dir().join(format!("ams-evidence-{}-{}.png", report_ts, evidence_seq));
        if std::fs::write(&temp_path, bytes).is_ok() {
            let asset_name = format!("/auditmysite-evidence-{}.png", evidence_seq);
            builder = builder.asset(asset_name.clone(), temp_path);
            let caption = i18n.t(match occ.evidence_viewport {
                Some("mobile") => "pdf-findings-screenshot-mobile",
                _ => "pdf-findings-screenshot-desktop",
            });
            builder = builder.add_component(
                Image::new(asset_name)
                    .with_width("55%")
                    .with_caption(caption),
            );
        }
        *evidence_seq += 1;
    }
    builder
}

fn add_suggested_fix(
    builder: renderreport::engine::ReportBuilder,
    occ: &RepresentativeOccurrence,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    match occ
        .suggested_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(code) => builder.add_component(
            Callout::info(compact_html(code, 180)).with_title(i18n.t("finding-suggested-fix")),
        ),
        None => builder,
    }
}

/// Extract the HTML element type from a CSS selector path.
/// Examples: "div.main > img" → "img", "a#skip-link" → "a", "button" → "button"
fn extract_element_type(selector: &str) -> &str {
    let last_segment = selector.split('>').next_back().unwrap_or(selector).trim();
    let last_token = last_segment
        .split_whitespace()
        .last()
        .unwrap_or(last_segment);
    let end = last_token
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .unwrap_or(last_token.len());
    &last_token[..end]
}

#[cfg(test)]
mod tests {
    use super::customer_perspective_title;
    use crate::output::pdf::single_report::risk_and_strength_tests::test_report_view_model;
    use crate::output::report_model::FindingGroup;
    use crate::wcag::Severity;

    /// #1039: the German CSP fix has a "bzw." inside its first sentence.
    #[test]
    fn first_sentence_skips_inner_abbreviations() {
        assert_eq!(
            super::first_sentence("Skripte bzw. -Styles zulassen. Rest."),
            "Skripte bzw. -Styles zulassen."
        );
        assert_eq!(
            super::first_sentence("Erst z. B. dies. Dann das."),
            "Erst z. B. dies."
        );
    }

    /// Die Messwerte der geteilten Kontrastregel kommen ohne `:1`, die der
    /// frueheren lokalen mit; beide ergeben dieselbe Zeile.
    #[test]
    fn contrast_line_reads_shared_and_legacy_evidence() {
        use super::{contrast_measured_text, Evidence};
        let shared = [
            Evidence::computed("contrast_ratio", "2.56"),
            Evidence::computed("required_ratio", "4.5"),
        ];
        let legacy = [
            Evidence::computed("contrast_ratio", "2.56:1"),
            Evidence::computed("required_ratio", "4.5:1"),
        ];
        let expected = Some("Contrast 2.56:1 (required 4.5:1)".to_string());
        assert_eq!(contrast_measured_text(&shared, true), expected);
        assert_eq!(contrast_measured_text(&legacy, true), expected);
        // German writes the decimal comma (#902).
        assert_eq!(
            contrast_measured_text(&shared, false),
            Some("Kontrast 2,56:1 (erforderlich 4,5:1)".to_string())
        );
    }

    /// #1112: a contrast occurrence measured under opacity names the opacity
    /// and the composited colour; the cross-page line lists each opacity once.
    /// Without the state-dependent mark there is no line.
    #[test]
    fn opacity_lines_name_the_opacity_behind_the_ratio() {
        use super::{opacity_pattern_text, opacity_text, Evidence};
        let dimmed = vec![
            Evidence::computed("contrast_ratio", "3.90"),
            Evidence::computed("opacity", "0.55"),
            Evidence::computed("effective_foreground", "#8c8c8c"),
            Evidence::computed("state_dependent", "true"),
        ];
        assert_eq!(
            opacity_text(&dimmed, true).as_deref(),
            Some("0.55: the ratio applies to this state (text colour after opacity #8c8c8c)")
        );
        assert_eq!(
            opacity_text(&dimmed, false).as_deref(),
            Some("0,55: Das Verhältnis gilt für diesen Zustand (Textfarbe nach Deckkraft #8c8c8c)")
        );
        assert_eq!(opacity_text(&dimmed[..1], true), None);

        let occ = |evidence: Vec<Evidence>| crate::audit::normalized::OccurrenceDetail {
            evidence,
            ..Default::default()
        };
        let mut half = dimmed.clone();
        half[1] = Evidence::computed("opacity", "0.50");
        let occurrences = [occ(dimmed.clone()), occ(half), occ(dimmed), occ(vec![])];
        let line = opacity_pattern_text(&occurrences, true).unwrap();
        assert!(line.contains("(0.50, 0.55)"), "{line}");
        assert_eq!(opacity_pattern_text(&occurrences[3..], true), None);
    }

    /// #902-5: a BFSG-relevant WCAG violation must lift the screen-reader
    /// verdict, so no "no violations" callout sits next to it; findings with
    /// low relevance or still awaiting a manual check do not.
    #[test]
    fn bfsg_callout_never_clears_a_page_with_a_relevant_violation() {
        use crate::output::pdf::detail_modules::{
            bfsg_relevant_wcag_findings, effective_bfsg_verdict,
        };
        use crate::screen_reader::BfsgVerdict;

        let mut contrast = group("contrast/text-insufficient", "Contrast", "Accessibility");
        contrast.finding.bfsg_relevance = "medium".to_string();
        let mut html = group("html_conform.schema.html5", "Schema", "HTML Conformance");
        html.finding.bfsg_relevance = "low".to_string();
        let mut manual = group("a11y.x", "Manual", "Accessibility");
        manual.finding.bfsg_relevance = "high".to_string();
        manual.finding.verification = "manual_review_recommended".to_string();

        assert_eq!(bfsg_relevant_wcag_findings(&[html.clone(), manual]), 0);
        let relevant = bfsg_relevant_wcag_findings(&[contrast, html]);
        assert_eq!(relevant, 1);

        for sr in [
            BfsgVerdict::NoViolationsFound,
            BfsgVerdict::Incomplete,
            BfsgVerdict::NeedsReview,
            BfsgVerdict::NotEvaluated,
            BfsgVerdict::ViolationsFound,
        ] {
            assert_eq!(
                effective_bfsg_verdict(sr, relevant, 0),
                BfsgVerdict::ViolationsFound
            );
        }
        // Nothing relevant elsewhere: the screen-reader verdict stands.
        assert_eq!(
            effective_bfsg_verdict(BfsgVerdict::NoViolationsFound, 0, 0),
            BfsgVerdict::NoViolationsFound
        );
        // Low-severity reading-order notes alone are no violation (#952-2) ...
        assert_eq!(
            effective_bfsg_verdict(BfsgVerdict::ViolationsFound, 0, 0),
            BfsgVerdict::NeedsReview
        );
        // ... a high-severity screen-reader blocker still is.
        assert_eq!(
            effective_bfsg_verdict(BfsgVerdict::ViolationsFound, 0, 1),
            BfsgVerdict::ViolationsFound
        );
    }

    fn group(rule_id: &str, title: &str, dimension: &str) -> FindingGroup {
        let mut group = test_report_view_model().findings.all_findings.remove(0);
        group.title = title.to_string();
        group.user_impact =
            "Nutzer können die betroffene Funktion nicht zuverlässig verwenden.".to_string();
        let f = &mut group.finding;
        f.rule_id = rule_id.to_string();
        f.dimension = dimension.to_string();
        f.subcategory = String::new();
        f.severity = Severity::High;
        f.occurrence_count = 1;
        f.bfsg_relevance = String::new();
        f.verification = String::new();
        group
    }

    /// #1109: the origin line is a neutral label with the provider and where
    /// the fix belongs; a main-document finding has none. Both languages, the
    /// English one without German characters.
    #[test]
    fn origin_line_names_the_provider_and_the_fix_in_both_languages() {
        use crate::i18n::I18n;
        use crate::model::FindingOrigin as O;
        let en = I18n::new("en").unwrap();
        let de = I18n::new("de").unwrap();
        let finding = |origin: O, providers: &[&str], host: Option<&str>| {
            let mut f = group("buttons/name-missing", "Button", "Accessibility").finding;
            f.origin = origin;
            f.providers = providers.iter().map(|p| p.to_string()).collect();
            f.occurrences = vec![crate::audit::normalized::OccurrenceDetail {
                origin: if origin == O::FirstParty {
                    O::FirstParty
                } else {
                    O::CrossOriginFrame
                },
                frame_host: host.map(str::to_string),
                ..Default::default()
            }];
            f
        };
        let yt = finding(O::CrossOriginFrame, &["YouTube"], Some("www.youtube.com"));
        assert_eq!(
            super::origin_text(&yt, &en).as_deref(),
            Some("Third-party frame (provider YouTube), fix at the provider")
        );
        assert_eq!(
            super::origin_text(&yt, &de).as_deref(),
            Some("Drittanbieter-Frame (Anbieter YouTube), Behebung beim Anbieter")
        );
        // Unknown provider: the host of the iframe's `src` is named.
        let unknown = finding(O::CrossOriginFrame, &[], Some("ir-api.eqs.com"));
        assert_eq!(
            super::origin_text(&unknown, &en).as_deref(),
            Some("Third-party frame (source ir-api.eqs.com), fix at the provider of the embedded content")
        );
        assert_eq!(
            super::origin_text(&finding(O::CrossOriginFrame, &[], None), &en).as_deref(),
            Some("Third-party frame, fix at the provider of the embedded content")
        );
        assert_eq!(
            super::origin_text(&finding(O::FirstParty, &[], None), &en),
            None
        );
        for origin in [O::SameSiteFrame, O::CrossOriginFrame, O::Mixed] {
            for (providers, host) in [
                (vec![], None),
                (vec!["YouTube"], Some("www.youtube.com")),
                (vec![], Some("ir-api.eqs.com")),
            ] {
                let f = finding(origin, &providers, host);
                let text = super::origin_text(&f, &en).unwrap();
                assert!(
                    !text.contains(['\u{e4}', '\u{f6}', '\u{fc}', '\u{df}']),
                    "{text}"
                );
                assert!(!text.contains('$'), "{text}");
                assert_ne!(Some(text), super::origin_text(&f, &de));
            }
        }
    }

    #[test]
    fn the_finding_card_shows_the_origin_only_for_frame_findings() {
        use super::render_finding_technical;
        use crate::i18n::I18n;
        use crate::model::FindingOrigin as O;
        let i18n = I18n::new("en").unwrap();
        let card = |origin: O, providers: Vec<String>| {
            let mut g = group("buttons/name-missing", "Button name", "Accessibility");
            g.finding.origin = origin;
            g.finding.providers = providers;
            let mut seq = 0;
            let mut expanded = false;
            let builder = render_finding_technical(
                renderreport::engine::ReportBuilder::new("t"),
                &g,
                &i18n,
                0,
                &mut seq,
                &mut expanded,
            );
            serde_json::to_string(&builder.build().components).unwrap()
        };
        let frame = card(O::CrossOriginFrame, vec!["YouTube".to_string()]);
        assert!(frame.contains("Third-party frame (provider YouTube), fix at the provider"));
        let page = card(O::FirstParty, Vec::new());
        assert!(!page.contains("Third-party frame"));
        assert!(!page.contains("fix at the provider"));
    }

    #[test]
    fn finding_customer_perspective_maps_common_audit_topics() {
        assert_eq!(
            customer_perspective_title(
                &group(
                    "buttons/name-missing",
                    "Fehlende Name/Rolle",
                    "Accessibility"
                ),
                false,
            ),
            "Bedienbarkeit"
        );
        assert_eq!(
            customer_perspective_title(
                &group(
                    "lists/empty",
                    "Fehlende semantische Struktur",
                    "Accessibility"
                ),
                false,
            ),
            "Orientierung und Struktur"
        );
        assert_eq!(
            customer_perspective_title(
                &group(
                    "contrast/text-insufficient",
                    "Unzureichender Farbkontrast",
                    "Accessibility"
                ),
                false,
            ),
            "Wahrnehmbarkeit und Lesbarkeit"
        );
        assert_eq!(
            customer_perspective_title(
                &group("seo.headings.empty_heading", "Leere Überschrift", "SEO"),
                false,
            ),
            "Orientierung und Struktur"
        );
    }
}
