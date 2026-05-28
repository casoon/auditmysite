//! Versioned prompts for semantic AI evaluation.

pub const PROMPT_VERSION: &str = "1";

/// Prompt asking whether the heading hierarchy is logical.
///
/// Returns JSON: `{"plausible": bool, "concerns": ["..."], "suggestion": "..."}`.
pub fn heading_outline_prompt(headings_yaml: &str, locale: &str) -> String {
    if locale == "de" {
        format!(
            "Du bist ein Web-Accessibility-Experte. Analysiere die folgende Überschriften-Hierarchie \
einer Webseite und beurteile, ob sie für Screenreader-Nutzer nachvollziehbar und logisch ist.\n\n\
Überschriften:\n{headings_yaml}\n\n\
Antworte ausschließlich mit gültigem JSON in diesem Format:\n\
{{\"plausible\": true/false, \"concerns\": [\"...\"], \"suggestion\": \"...\"}}\n\
- plausible: true wenn die Hierarchie verständlich ist, false wenn es erhebliche Probleme gibt.\n\
- concerns: Liste konkreter Probleme (leeres Array wenn keine).\n\
- suggestion: Ein Satz mit dem wichtigsten Verbesserungsvorschlag (leer wenn keine Probleme)."
        )
    } else {
        format!(
            "You are a web accessibility expert. Analyze the following heading hierarchy of a web page \
and evaluate whether it is logical and navigable for screen reader users.\n\n\
Headings:\n{headings_yaml}\n\n\
Reply exclusively with valid JSON in this format:\n\
{{\"plausible\": true/false, \"concerns\": [\"...\"], \"suggestion\": \"...\"}}\n\
- plausible: true if the hierarchy is comprehensible, false if there are significant issues.\n\
- concerns: List of specific problems (empty array if none).\n\
- suggestion: One sentence with the most important improvement (empty string if no issues)."
        )
    }
}

/// Prompt asking for a brief evaluation from a blind user's perspective.
///
/// Returns plain text (3-5 sentences).
pub fn blind_user_perspective_prompt(aria_snapshot_yaml: &str, locale: &str) -> String {
    if locale == "de" {
        format!(
            "Du bist ein erfahrener Screenreader-Nutzer mit Sehbehinderung. \
Bewerte die folgende vereinfachte Accessibility-Struktur einer Webseite aus deiner Perspektive \
in 3-5 Sätzen. Fokussiere auf: Was funktioniert gut? Was ist verwirrend oder schwer zugänglich? \
Was würdest du dir wünschen?\n\nAccessibility-Struktur:\n{aria_snapshot_yaml}"
        )
    } else {
        format!(
            "You are an experienced screen reader user with a visual impairment. \
Evaluate the following simplified accessibility structure of a web page from your perspective \
in 3-5 sentences. Focus on: What works well? What is confusing or hard to access? \
What would you wish for?\n\nAccessibility structure:\n{aria_snapshot_yaml}"
        )
    }
}
