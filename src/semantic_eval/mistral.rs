//! Mistral API evaluator for semantic accessibility checks.
//!
//! Uses the existing `reqwest` dependency — no additional crates required.

use serde::Deserialize;
use tracing::warn;

use crate::audit::normalized::AdvisoryFinding;

pub struct MistralEvaluator {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

// ── Mistral API response types ────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Deserialize)]
struct HeadingOutlineResponse {
    plausible: bool,
    #[serde(default)]
    concerns: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    suggestion: String,
}

// ── Implementation ────────────────────────────────────────────────────────────

impl MistralEvaluator {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    /// Send a chat completion request to the Mistral API and return the text response.
    async fn chat(
        &self,
        system: Option<&str>,
        user: &str,
        max_tokens: u32,
    ) -> crate::error::Result<String> {
        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": user
        }));

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": max_tokens
        });

        let response = self
            .client
            .post("https://api.mistral.ai/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let chat: ChatResponse = response.json().await?;
        let content = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }

    /// Evaluate the heading outline of the page for logical structure.
    pub async fn evaluate_heading_outline(
        &self,
        tree: &crate::accessibility::AXTree,
    ) -> crate::error::Result<Vec<AdvisoryFinding>> {
        let headings = tree.headings();
        if headings.len() < 2 {
            return Ok(Vec::new());
        }

        // Build YAML-like heading list
        let mut yaml = String::new();
        for h in &headings {
            let level = h.heading_level().unwrap_or(0);
            let name = h.name.as_deref().unwrap_or("").replace('\'', "\\'");
            yaml.push_str(&format!("- level: {level}\n  text: '{name}'\n"));
        }

        let prompt = crate::semantic_eval::prompts::heading_outline_prompt(&yaml, "de");
        let raw = match self.chat(None, &prompt, 512).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Mistral heading_outline request failed: {}", e);
                return Ok(Vec::new());
            }
        };

        // Extract JSON from response (may have markdown code fences)
        let json_str = extract_json(&raw);
        let parsed: HeadingOutlineResponse = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(e) => {
                warn!("Mistral heading_outline JSON parse failed: {}", e);
                return Ok(Vec::new());
            }
        };

        if parsed.plausible {
            return Ok(Vec::new());
        }

        let findings: Vec<AdvisoryFinding> = parsed
            .concerns
            .into_iter()
            .map(|concern| AdvisoryFinding {
                category: "heading_outline".to_string(),
                message: concern,
                source: "mistral".to_string(),
                confidence: 0.9,
            })
            .collect();

        Ok(findings)
    }

    /// Evaluate the page from a blind user's perspective.
    pub async fn evaluate_blind_user_perspective(
        &self,
        tree: &crate::accessibility::AXTree,
    ) -> crate::error::Result<Vec<AdvisoryFinding>> {
        // Build minimal ARIA snapshot: role + name, max 50 nodes
        let mut yaml = String::new();
        let mut count = 0usize;
        for node in tree.iter() {
            if count >= 50 {
                break;
            }
            if let Some(name) = &node.name {
                if name.trim().is_empty() {
                    continue;
                }
                let role = node.role.as_deref().unwrap_or("unknown");
                let safe_name = name.replace('\'', "\\'");
                yaml.push_str(&format!("- role: {role}\n  name: '{safe_name}'\n"));
                count += 1;
            }
        }

        if yaml.is_empty() {
            return Ok(Vec::new());
        }

        let prompt = crate::semantic_eval::prompts::blind_user_perspective_prompt(&yaml, "de");
        let response_text = match self.chat(None, &prompt, 512).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Mistral blind_user_perspective request failed: {}", e);
                return Ok(Vec::new());
            }
        };

        if response_text.trim().is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![AdvisoryFinding {
            category: "blind_user_perspective".to_string(),
            message: response_text,
            source: "mistral".to_string(),
            confidence: 0.8,
        }])
    }
}

/// Extract the first JSON object from a string (strips markdown code fences if present).
fn extract_json(s: &str) -> String {
    let s = s.trim();
    // Strip markdown fences like ```json ... ```
    let s = if s.starts_with("```") {
        let inner = s.trim_start_matches('`');
        let inner = inner.trim_start_matches("json").trim_start_matches('\n');
        inner.trim_end_matches('`').trim_end_matches('\n')
    } else {
        s
    };
    s.to_string()
}
