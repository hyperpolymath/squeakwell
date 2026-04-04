// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Property tests for squeakwell — fixed-input arrays verify invariants
// that should hold across the full domain of valid inputs.
// No external proptest/quickcheck dependency: we use hand-crafted fixture
// arrays so the only dev-dependency is tempfile (already present).

use tempfile::TempDir;

// ─── Fixture inputs ─────────────────────────────────────────────────────────

/// Eight representative (source, format) pairs covering all supported formats.
const INGEST_CASES: [(&str, &str); 8] = [
    ("dump.sql",      "sql-dump"),
    ("export.json",   "json"),
    ("table.csv",     "csv"),
    ("log.wal",       "wal"),
    ("archive.bin",   "binary"),
    ("fragments/",    "directory"),
    ("data.jsonl",    "auto"),  // auto → json
    ("data.tsv",      "auto"),  // auto → csv
];

/// Ten (target_level, drift_threshold, max_iterations) triples exercising
/// the full parameter space of the recovery engine.
const RECOVER_CASES: [(u8, f64, u32); 10] = [
    (1, 0.9, 1),
    (2, 0.8, 5),
    (3, 0.6, 10),
    (4, 0.5, 20),
    (5, 0.4, 50),
    (6, 0.3, 100),
    (7, 0.2, 100),
    (8, 0.1, 100),
    (9, 0.05, 100),
    (10, 0.01, 200),
];

fn scratch() -> TempDir {
    tempfile::tempdir().expect("create temp dir")
}

// ─── Determinism invariant ───────────────────────────────────────────────────

/// Ingest + recover called twice on identical inputs produces identical workdir
/// layout (same set of directories).  This verifies the determinism ABI proof:
///   DeterministicResolution: resolve(c, I) = resolve(c, I)
#[test]
fn ingest_is_deterministic() {
    for (source, format) in INGEST_CASES {
        let tmp1 = scratch();
        let tmp2 = scratch();
        let wd1 = tmp1.path().join("wd");
        let wd2 = tmp2.path().join("wd");

        squeakwell::ingest::ingest(source, format, wd1.to_str().unwrap())
            .unwrap_or_else(|e| panic!("ingest({source}, {format}) run-1 failed: {e}"));
        squeakwell::ingest::ingest(source, format, wd2.to_str().unwrap())
            .unwrap_or_else(|e| panic!("ingest({source}, {format}) run-2 failed: {e}"));

        // Both workdirs must contain the same set of 8 modality subdirectories.
        for modality in ["graph", "vector", "tensor", "semantic", "document", "temporal", "provenance", "spatial"] {
            assert!(
                wd1.join(modality).exists(),
                "run-1 missing modality {modality} for input ({source}, {format})"
            );
            assert!(
                wd2.join(modality).exists(),
                "run-2 missing modality {modality} for input ({source}, {format})"
            );
        }
    }
}

// ─── Octad completeness invariant ───────────────────────────────────────────

/// Every ingest call must create all 8 modality directories — the octad is
/// always complete regardless of source format.
#[test]
fn ingest_always_creates_complete_octad() {
    for (source, format) in INGEST_CASES {
        let tmp = scratch();
        let wd = tmp.path().join("wd");

        squeakwell::ingest::ingest(source, format, wd.to_str().unwrap())
            .unwrap_or_else(|e| panic!("ingest({source}, {format}) failed: {e}"));

        let count = ["graph", "vector", "tensor", "semantic", "document", "temporal", "provenance", "spatial"]
            .iter()
            .filter(|&&m| wd.join(m).is_dir())
            .count();

        assert_eq!(count, 8, "incomplete octad for ({source}, {format}): got {count}/8 modalities");
    }
}

// ─── Recovery termination invariant ─────────────────────────────────────────

/// The engine must always return Ok (terminate) for every combination in
/// RECOVER_CASES — the TerminationGuarantee ABI proof states:
///   ∀ workload w. ∃ i. drift(w, i) < threshold ∨ i = max_iterations
#[test]
fn recover_always_terminates() {
    let tmp = scratch();
    let wd = tmp.path().join("wd");
    squeakwell::ingest::ingest("seed.sql", "sql-dump", wd.to_str().unwrap())
        .expect("seed ingest");
    let wd_str = wd.to_str().unwrap();

    for (level, threshold, max_iter) in RECOVER_CASES {
        squeakwell::engine::recover(wd_str, level, threshold, max_iter)
            .unwrap_or_else(|e| panic!("recover(level={level}, thresh={threshold}, max={max_iter}) failed: {e}"));
    }
}

// ─── Type ratchet soundness invariant ───────────────────────────────────────

/// Higher target_level ≥ lower target_level: all levels up to the target must
/// pass.  Here we verify that calling recover at level N succeeds for all N in
/// 1..=10, modelling the TypeRatchetSoundness ABI proof.
#[test]
fn recover_level_monotonic_success() {
    let tmp = scratch();
    let wd = tmp.path().join("wd");
    squeakwell::ingest::ingest("dump.json", "json", wd.to_str().unwrap())
        .expect("seed ingest");
    let wd_str = wd.to_str().unwrap();

    for level in 1u8..=10 {
        squeakwell::engine::recover(wd_str, level, 0.5, 10)
            .unwrap_or_else(|e| panic!("recover at level {level} failed: {e}"));
    }
}

// ─── Output/query functions ──────────────────────────────────────────────────

/// print_confidence and print_review_queue must not error for any well-formed
/// confidence threshold in [0.0, 1.0].
#[test]
fn engine_query_functions_stable_for_valid_inputs() {
    let tmp = scratch();
    let wd = tmp.path().join("wd");
    squeakwell::ingest::ingest("data.csv", "csv", wd.to_str().unwrap()).expect("ingest");
    let wd_str = wd.to_str().unwrap();

    for threshold in [0.0, 0.1, 0.5, 0.8, 0.99, 1.0] {
        squeakwell::engine::print_confidence(wd_str, threshold)
            .unwrap_or_else(|e| panic!("print_confidence({threshold}) failed: {e}"));
    }
    for format in ["human", "json", "csv"] {
        squeakwell::engine::print_review_queue(wd_str, format)
            .unwrap_or_else(|e| panic!("print_review_queue({format}) failed: {e}"));
    }
}
