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
    assert!(svc.get("ports").is_none(), "a batch CLI exposes no ports");
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
