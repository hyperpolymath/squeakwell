// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// E2E tests for squeakwell — full pipeline from ingest through engine to output.
// Each test exercises the complete ingest → recover → export flow using a
// temporary working directory so nothing leaks between tests.

use std::path::Path;
use tempfile::TempDir;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Creates a fresh temporary directory to serve as the squeakwell workdir.
fn scratch_dir() -> TempDir {
    tempfile::tempdir().expect("create temp workdir")
}

// ─── Ingest pipeline ────────────────────────────────────────────────────────

/// Ingesting a non-existent path should still create the octad working tree.
#[test]
fn ingest_creates_octad_workdir() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("workdir");
    let workdir_str = workdir.to_str().unwrap();

    squeakwell::ingest::ingest("nonexistent.sql", "sql-dump", workdir_str)
        .expect("ingest should succeed even without source file");

    // The eight octad modalities must all be created.
    for modality in ["graph", "vector", "tensor", "semantic", "document", "temporal", "provenance", "spatial"] {
        let modal_path = workdir.join(modality);
        assert!(modal_path.exists(), "modality directory missing: {}", modality);
    }
}

/// Ingest with format "auto" on a .json path must detect the format correctly.
#[test]
fn ingest_auto_detect_json_extension() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("wd_json");
    let workdir_str = workdir.to_str().unwrap();

    // The function prints the detected format; we verify it succeeds without
    // panicking and that the workdir structure is complete.
    squeakwell::ingest::ingest("dump.json", "auto", workdir_str)
        .expect("auto-detect json ingest should succeed");

    assert!(workdir.join("document").exists());
}

/// Ingest with explicit format "csv" must still create the full octad layout.
#[test]
fn ingest_explicit_csv_format_creates_full_octad() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("wd_csv");
    let workdir_str = workdir.to_str().unwrap();

    squeakwell::ingest::ingest("data.csv", "csv", workdir_str)
        .expect("explicit csv ingest should succeed");

    assert!(workdir.join("provenance").exists());
    assert!(workdir.join("spatial").exists());
}

/// Ingest of a binary file (unknown extension) must succeed via binary fallback.
#[test]
fn ingest_binary_fallback_for_unknown_extension() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("wd_bin");
    let workdir_str = workdir.to_str().unwrap();

    squeakwell::ingest::ingest("database.mdb", "auto", workdir_str)
        .expect("binary fallback ingest should succeed");

    assert!(workdir.join("graph").exists());
}

// ─── Recovery engine ────────────────────────────────────────────────────────

/// Running recover on an empty workdir must complete all 5 phases without error.
#[test]
fn recover_all_five_phases_complete() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("recover_test");
    let workdir_str = workdir.to_str().unwrap();

    // Prepare workdir via ingest first.
    squeakwell::ingest::ingest("data.sql", "sql-dump", workdir_str)
        .expect("ingest phase");

    squeakwell::engine::recover(workdir_str, 6, 0.05, 10)
        .expect("recover should complete all phases");
}

/// Requesting target_level 1 limits recovery to phase 1 VQL-UT checks only.
#[test]
fn recover_target_level_one_does_not_error() {
    let tmp = scratch_dir();
    let workdir = tmp.path().join("recover_level1");
    let workdir_str = workdir.to_str().unwrap();

    squeakwell::ingest::ingest("data.wal", "wal", workdir_str).expect("ingest");
    squeakwell::engine::recover(workdir_str, 1, 0.5, 5).expect("recover at level 1");
}

/// print_status must succeed without panicking on an empty workdir.
#[test]
fn status_on_empty_workdir_returns_ok() {
    let tmp = scratch_dir();
    let workdir_str = tmp.path().to_str().unwrap();

    squeakwell::engine::print_status(workdir_str).expect("print_status should not fail");
}

/// export with format "json" must return Ok even for a stub workdir.
#[test]
fn export_json_format_returns_ok() {
    let tmp = scratch_dir();
    let workdir_str = tmp.path().to_str().unwrap();
    let output = tmp.path().join("out.json");

    squeakwell::engine::export(
        workdir_str,
        "json",
        output.to_str().unwrap(),
    )
    .expect("export json should return Ok");
}
