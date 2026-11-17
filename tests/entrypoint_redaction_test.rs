// SPDX-License-Identifier: MPL-2.0
//! The container entrypoint logs its run parameters before it execs the
//! binary. These tests run the real script under `sh` (the final `exec`
//! fails here because `/app/squeakwell` does not exist, which is fine: the
//! log lines are already written) and pin that no secret part of
//! `VERISIMDB_URL` reaches the log, and that a blank value is reported the
//! way `VeriSimDbClient::from_env_value` treats it.

use std::path::Path;
use std::process::Command;

fn entrypoint_log(url: Option<&str>) -> String {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("container/entrypoint.sh");
    let data = std::env::temp_dir();
    let mut cmd = Command::new("sh");
    cmd.arg(&script)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("APP_DATA_DIR", &data);
    if let Some(u) = url {
        cmd.env("VERISIMDB_URL", u);
    }
    let out = cmd.output().expect("run sh");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn verisim_line(log: &str) -> String {
    log.lines()
        .find(|l| l.contains("VeriSimDB:"))
        .expect("VeriSimDB line")
        .to_owned()
}

#[test]
fn secrets_in_userinfo_path_query_and_fragment_are_not_logged() {
    for url in [
        "https://user:SECRET@db.example:8443/v1?token=SECRET#SECRET",
        "https://db.example:8443/v1?token=a@SECRET",
        "https://db.example:8443#a@SECRET",
    ] {
        let line = verisim_line(&entrypoint_log(Some(url)));
        assert!(!line.contains("SECRET"), "{url} leaked: {line}");
        assert!(
            line.contains("https://db.example:8443"),
            "{url} lost the host: {line}"
        );
    }
}

#[test]
fn an_invalid_scheme_prefix_is_not_logged() {
    for url in [
        "host?token=SECRET://example",
        "SECRET/x://y",
        "://SECRET",
        "9x://SECRET",
    ] {
        let line = verisim_line(&entrypoint_log(Some(url)));
        assert!(!line.contains("SECRET"), "{url} leaked: {line}");
        assert!(line.contains("not logged"), "{url}: {line}");
    }
}

#[test]
fn a_url_without_scheme_keeps_only_the_host() {
    let line = verisim_line(&entrypoint_log(Some("user:SECRET@db.example/x?t=SECRET")));
    assert!(!line.contains("SECRET"), "leaked: {line}");
    assert!(line.contains("db.example"), "lost the host: {line}");
}

#[test]
fn unset_empty_and_whitespace_values_warn_about_the_fallback() {
    assert!(verisim_line(&entrypoint_log(None)).contains("<unset>"));
    for blank in ["", "   ", "\t"] {
        let line = verisim_line(&entrypoint_log(Some(blank)));
        assert!(line.contains("<set but blank>"), "{blank:?}: {line}");
    }
}
