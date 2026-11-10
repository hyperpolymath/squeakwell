// SPDX-License-Identifier: MPL-2.0
//! The container stack definitions must be valid TOML 1.0. A multi-line
//! inline table with a trailing comma (the shape the estate template ships)
//! is rejected by TOML 1.0 parsers; this test pins the rendered files.

use std::path::Path;

fn parse(path: &str) -> toml::Value {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("read {path}: {e}"));
    toml::from_str(&text).unwrap_or_else(|e| panic!("{path} is not valid TOML: {e}"))
}

#[test]
fn compose_toml_parses_and_names_the_batch_service() {
    let v = parse("container/compose.toml");
    let svc = &v["services"]["squeakwell"];
    assert_eq!(svc["restart"].as_str(), Some("no"));
    assert_eq!(svc["environment"]["APP_DATA_DIR"].as_str(), Some("/data"));
    // The VeriSimDB target and the per-batch session key must both reach the
    // container; without them the client falls back to localhost and every
    // batch would share one session id.
    assert_eq!(svc["environment"]["VERISIMDB_URL"].as_str(), Some("${VERISIMDB_URL}"));
    assert_eq!(svc["environment"]["SQUEAKWELL_BATCH_ID"].as_str(), Some("${BATCH_ID}"));
    assert!(svc.get("ports").is_none(), "a batch CLI exposes no ports");
    // One named volume per batch, mounted at the workdir, and declared.
    let volumes: Vec<&str> =
        svc["volumes"].as_array().expect("volumes array").iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(volumes, vec!["squeakwell-data-${BATCH_ID}:/data"]);
    assert!(
        v["volumes"].get("squeakwell-data-${BATCH_ID}").is_some(),
        "the per-batch volume must be declared under [volumes]"
    );
}

#[test]
fn manifest_toml_parses_with_no_listening_sockets() {
    let v = parse("container/manifest.toml");
    assert_eq!(v["metadata"]["name"].as_str(), Some("squeakwell"));
    let listen = v["security"]["network"]["listen_tcp"].as_array().expect("listen_tcp array");
    assert!(listen.is_empty(), "squeakwell listens on no port");
}

#[test]
fn vordr_toml_parses_with_health_probe_disabled() {
    let v = parse("container/vordr.toml");
    assert_eq!(v["health"]["enabled"].as_bool(), Some(false));
    assert_eq!(v["crash_detection"]["max_restarts"].as_integer(), Some(0));
}

#[test]
fn session_id_is_per_batch_when_a_batch_id_is_supplied() {
    use squeakwell::engine::session_id_for;
    // Two batches on the same mount path must not share a VeriSimDB session.
    let a = session_id_for("/data", Some("20261109T120000Z-1"));
    let b = session_id_for("/data", Some("20261109T120000Z-2"));
    assert_ne!(a, b);
    assert!(a.starts_with("sq:b:") && a.len() == 17);
    // Batch and workdir IDs live in separate namespaces: matching inputs,
    // including a workdir that spells out the old "batch:" prefix, never
    // share a session.
    assert_ne!(session_id_for("foo", None), session_id_for("/data", Some("foo")));
    assert_ne!(session_id_for("batch:foo", None), session_id_for("/data", Some("foo")));
    // Workdir IDs are unchanged in shape, so existing sessions stay reachable.
    let w = session_id_for("/data", None);
    assert!(w.starts_with("sq:") && !w.starts_with("sq:b:") && w.len() == 15);
    // Without a batch id the id is a stable function of the workdir alone,
    // and a blank batch id counts as absent.
    assert_eq!(session_id_for("/data", None), session_id_for("/data", Some("  ")));
    assert_ne!(session_id_for("/data", None), session_id_for("/other", None));
}
