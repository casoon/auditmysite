//! Declared browser state (#787): cookies and localStorage a run starts
//! from, read from a Playwright-style `storage-state` JSON file.
//!
//! The state is part of the measurement: its digest and counts go into the
//! report's measurement profile, its values never do — the file may hold a
//! login.

use std::fmt;
use std::path::Path;

use chromiumoxide::cdp::browser_protocol::network::{CookieParam, SetCookiesParams};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chromiumoxide::Page;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::error::{AuditError, Result};

#[derive(Deserialize)]
struct StateFile {
    #[serde(default)]
    cookies: Vec<StateCookie>,
    #[serde(default)]
    origins: Vec<StateOrigin>,
}

#[derive(Deserialize)]
struct StateCookie {
    name: String,
    value: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    secure: Option<bool>,
    #[serde(default, rename = "httpOnly")]
    http_only: Option<bool>,
}

#[derive(Deserialize)]
struct StateOrigin {
    origin: String,
    #[serde(default, rename = "localStorage")]
    local_storage: Vec<StateItem>,
}

#[derive(Deserialize)]
struct StateItem {
    name: String,
    value: String,
}

/// A loaded state. `Debug` shows the identity, never a value.
#[derive(Clone)]
pub struct StorageState {
    file: std::sync::Arc<StateFile>,
    /// SHA-256 of the file bytes, hex.
    pub digest: String,
}

impl fmt::Debug for StorageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StorageState")
            .field("digest", &self.digest)
            .field("cookies", &self.cookie_count())
            .field("origins", &self.origin_count())
            .finish()
    }
}

impl StorageState {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|e| {
            AuditError::ConfigError(format!("cannot read storage state {}: {e}", path.display()))
        })?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let file: StateFile = serde_json::from_slice(bytes).map_err(|e| {
            // The message names the position, not the content.
            AuditError::ConfigError(format!(
                "storage state is not valid (expected cookies[] and origins[]): line {}, column {}",
                e.line(),
                e.column()
            ))
        })?;
        Ok(Self {
            file: std::sync::Arc::new(file),
            digest: format!("{:x}", Sha256::digest(bytes)),
        })
    }

    pub fn cookie_count(&self) -> usize {
        self.file.cookies.len()
    }

    pub fn origin_count(&self) -> usize {
        self.file.origins.len()
    }

    /// Script run on every new document: writes the declared localStorage
    /// entries of the document's own origin.
    fn storage_script(&self) -> Option<String> {
        let by_origin: Vec<serde_json::Value> = self
            .file
            .origins
            .iter()
            .filter(|o| !o.local_storage.is_empty())
            .map(|o| {
                serde_json::json!({
                    "origin": o.origin,
                    "items": o.local_storage.iter()
                        .map(|i| [i.name.as_str(), i.value.as_str()])
                        .collect::<Vec<_>>(),
                })
            })
            .collect();
        if by_origin.is_empty() {
            return None;
        }
        Some(format!(
            "(function () {{ try {{ var all = {}; for (var i = 0; i < all.length; i++) {{ \
             if (all[i].origin !== location.origin) continue; \
             for (var j = 0; j < all[i].items.length; j++) {{ \
             localStorage.setItem(all[i].items[j][0], all[i].items[j][1]); }} }} }} \
             catch (e) {{}} }})();",
            serde_json::Value::Array(by_origin)
        ))
    }

    /// Before navigation: set the declared cookies and register the storage
    /// script. A cookie the browser rejects is logged by name only.
    pub async fn apply(&self, page: &Page, url: &str) {
        let cookies: Vec<CookieParam> = self
            .file
            .cookies
            .iter()
            .map(|c| {
                let mut p = CookieParam::new(c.name.clone(), c.value.clone());
                match &c.domain {
                    Some(domain) => p.domain = Some(domain.clone()),
                    None => p.url = Some(url.to_string()),
                }
                p.path = Some(c.path.clone().unwrap_or_else(|| "/".to_string()));
                p.secure = c.secure;
                p.http_only = c.http_only;
                p
            })
            .collect();
        if !cookies.is_empty() {
            if let Err(e) = page.execute(SetCookiesParams::new(cookies)).await {
                warn!("Declared cookies were not set: {e}");
            }
        }
        if let Some(script) = self.storage_script() {
            if let Err(e) = page
                .execute(AddScriptToEvaluateOnNewDocumentParams::new(script))
                .await
            {
                warn!("Declared localStorage was not registered: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"{"cookies":[{"name":"sid","value":"SECRET","domain":"example.com"}],
        "origins":[{"origin":"https://example.com","localStorage":[{"name":"k","value":"v"}]}]}"#;

    #[test]
    fn debug_and_errors_never_show_values() {
        let state = StorageState::from_bytes(FILE.as_bytes()).unwrap();
        assert_eq!((state.cookie_count(), state.origin_count()), (1, 1));
        assert!(!format!("{state:?}").contains("SECRET"));
        let err = StorageState::from_bytes(b"{\"cookies\": [SECRET").unwrap_err();
        assert!(!err.to_string().contains("SECRET"));
    }

    #[test]
    fn digest_follows_the_bytes() {
        let a = StorageState::from_bytes(FILE.as_bytes()).unwrap();
        let b = StorageState::from_bytes(FILE.replace("\"v\"", "\"w\"").as_bytes()).unwrap();
        assert_ne!(a.digest, b.digest);
        assert_eq!(
            a.digest,
            StorageState::from_bytes(FILE.as_bytes()).unwrap().digest
        );
    }

    #[test]
    fn storage_script_is_bound_to_the_declared_origin() {
        let state = StorageState::from_bytes(FILE.as_bytes()).unwrap();
        let script = state.storage_script().unwrap();
        assert!(script.contains("https://example.com") && script.contains("location.origin"));
        let cookies_only =
            StorageState::from_bytes(br#"{"cookies":[{"name":"a","value":"b"}]}"#).unwrap();
        assert!(cookies_only.storage_script().is_none());
    }
}
