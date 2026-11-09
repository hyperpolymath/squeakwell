// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// squeakwell::verisim — VeriSimDB client for recovery session persistence.
//
// Persists recovery state and per-entity confidence scores to VeriSimDB so
// that long-running recoveries can survive process restarts, be inspected by
// Hypatia rules, and be replayed for audit.
//
// ## Collections
//
//   squeakwell:sessions     — one document per recovery session
//   squeakwell:entities     — one document per entity, keyed by entity_id
//   squeakwell:phase-events — append-only phase transition log
//
// ## Fail-open semantics
//
// All operations return `anyhow::Result<()>`. Callers that can tolerate
// VeriSimDB unavailability should call `.ok()` on the result. The recovery
// engine uses this pattern: VeriSimDB is best-effort persistence, not a
// hard dependency. Recoveries complete even when VeriSimDB is unreachable.
//
// ## Environment variable
//
//   VERISIMDB_URL — override base URL (default: http://localhost:8080).
//                   An empty value is treated as unset (compose files pass
//                   `${VERISIMDB_URL}` through verbatim, so "unset" arrives
//                   as "").

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::env;

/// VeriSimDB REST client.
///
/// Wraps the three collection write patterns used by SqueakWell:
/// - `persist_session`  — upsert a recovery session snapshot
/// - `persist_entity`   — upsert per-entity confidence score
/// - `append_phase_event` — append a phase-transition event
///
/// All writes use HTTP PUT to `/v1/<collection>/<id>` (idempotent upserts).
pub struct VeriSimDbClient {
    base_url: String,
}

impl VeriSimDbClient {
    /// Create a new client, reading `VERISIMDB_URL` from the environment.
    ///
    /// Falls back to `http://localhost:8080` if the variable is unset,
    /// non-Unicode or blank. Otherwise trims surrounding whitespace and
    /// trailing slashes, as described by [`Self::from_env_value`].
    pub fn new() -> Self {
        Self::from_env_value(env::var("VERISIMDB_URL").ok())
    }

    /// Build a client from the raw `VERISIMDB_URL` value (`None` = unset).
    /// Trims surrounding whitespace, then uses `http://localhost:8080` for
    /// absent or blank values. Removes trailing slashes from the resulting
    /// base URL without validating it or making a network request.
    pub fn from_env_value(value: Option<String>) -> Self {
        let base_url = value
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "http://localhost:8080".to_owned());
        // Strip trailing slash for consistent URL construction
        Self { base_url: base_url.trim_end_matches('/').to_owned() }
    }

    /// The base URL this client sends requests to (no trailing slash).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // -------------------------------------------------------------------------
    // Public API
    // -------------------------------------------------------------------------

    /// Persist (upsert) a recovery session snapshot.
    ///
    /// `session_id` uniquely identifies this recovery workdir.
    /// `state` is the serialised `RecoveryState` from the engine.
    ///
    /// Fails open: callers should call `.ok()` if VeriSimDB is optional.
    pub fn persist_session(&self, session_id: &str, state: &Value) -> Result<()> {
        let url = self.doc_url("squeakwell:sessions", session_id);
        self.put(&url, state)
            .with_context(|| format!("VeriSimDB: persist_session failed for {session_id}"))
    }

    /// Persist (upsert) per-entity confidence score.
    ///
    /// `entity_id` is the entity's unique identifier within the recovery workdir.
    /// `confidence` is the serialised `EntityConfidence` from the engine.
    ///
    /// Fails open: callers should call `.ok()` if VeriSimDB is optional.
    pub fn persist_entity(&self, entity_id: &str, confidence: &Value) -> Result<()> {
        let url = self.doc_url("squeakwell:entities", entity_id);
        self.put(&url, confidence)
            .with_context(|| format!("VeriSimDB: persist_entity failed for {entity_id}"))
    }

    /// Append a phase-transition event to the event log.
    ///
    /// Uses a composite `<session_id>:phase<n>:<ts_ms>` key for
    /// chronological ordering within a session.
    ///
    /// Fails open: callers should call `.ok()` if VeriSimDB is optional.
    pub fn append_phase_event(
        &self,
        session_id: &str,
        phase: u8,
        event: &Value,
    ) -> Result<()> {
        let ts_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let doc_id = format!("{session_id}:phase{phase}:{ts_ms}");
        let url = self.doc_url("squeakwell:phase-events", &doc_id);
        self.put(&url, event)
            .with_context(|| format!("VeriSimDB: append_phase_event failed for {doc_id}"))
    }

    /// Retrieve a session snapshot by session ID.
    ///
    /// Returns `None` if the document does not exist or VeriSimDB is unavailable.
    pub fn get_session(&self, session_id: &str) -> Option<Value> {
        let url = self.doc_url("squeakwell:sessions", session_id);
        self.get(&url).ok()
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    /// Build the full document URL: `<base>/v1/<collection>/<encoded_id>`.
    fn doc_url(&self, collection: &str, id: &str) -> String {
        let encoded_id = urlencoding::encode(id);
        format!("{}/v1/{}/{}", self.base_url, collection, encoded_id)
    }

    /// HTTP PUT — idempotent document upsert.
    fn put(&self, url: &str, body: &Value) -> Result<()> {
        let response = ureq::put(url)
            .header("Content-Type", "application/json")
            .send_json(body)
            .with_context(|| format!("VeriSimDB: PUT {url} failed"))?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            anyhow::bail!("VeriSimDB: PUT {url} returned HTTP {}", status.as_u16())
        }
    }

    /// HTTP GET — document fetch.
    fn get(&self, url: &str) -> Result<Value> {
        let response = ureq::get(url)
            .call()
            .with_context(|| format!("VeriSimDB: GET {url} failed"))?;

        let status = response.status();
        if status.is_success() {
            let body: Value = response
                .into_body()
                .read_json()
                .with_context(|| format!("VeriSimDB: failed to parse JSON from GET {url}"))?;
            Ok(body)
        } else {
            anyhow::bail!("VeriSimDB: GET {url} returned HTTP {}", status.as_u16())
        }
    }
}

impl Default for VeriSimDbClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors for common document shapes
// ---------------------------------------------------------------------------

/// Build a session document from a recovery state value.
///
/// The returned object contains `session_id`, `workdir`, and `updated_at`
/// (the current UTC time in RFC 3339 format). The supplied `state` is nested
/// under the `state` key, including when it is not a JSON object.
pub fn session_doc(session_id: &str, workdir: &str, state: &Value) -> Value {
    let now = chrono::Utc::now().to_rfc3339();
    json!({
        "session_id": session_id,
        "workdir": workdir,
        "updated_at": now,
        "state": state,
    })
}

/// Build a phase-event document.
///
/// Captures the phase number, iteration, drift, and timestamp for the
/// squeakwell:phase-events append log.
pub fn phase_event_doc(session_id: &str, phase: u8, iteration: u32, drift: f64) -> Value {
    json!({
        "session_id": session_id,
        "phase": phase,
        "iteration": iteration,
        "drift": drift,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::VeriSimDbClient;

    #[test]
    fn blank_url_falls_back_like_unset() {
        let unset = VeriSimDbClient::from_env_value(None);
        let empty = VeriSimDbClient::from_env_value(Some(String::new()));
        let blank = VeriSimDbClient::from_env_value(Some("   ".to_owned()));
        assert_eq!(unset.base_url(), "http://localhost:8080");
        assert_eq!(empty.base_url(), unset.base_url());
        assert_eq!(blank.base_url(), unset.base_url());
    }

    #[test]
    fn set_url_is_kept_without_trailing_slash() {
        let c = VeriSimDbClient::from_env_value(Some("https://verisim.example/".to_owned()));
        assert_eq!(c.base_url(), "https://verisim.example");
        assert_eq!(c.doc_url("squeakwell:sessions", "sq:1"), "https://verisim.example/v1/squeakwell:sessions/sq%3A1");
    }
}
