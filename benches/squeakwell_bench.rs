// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Criterion benchmarks for squeakwell.
// Three benchmarks:
//   1. ingest_octad_setup      — cost of creating the 8-modality working tree.
//   2. recover_stub_five_phase — cost of the five-phase recovery loop (stub).
//   3. engine_query_batch      — cost of status + confidence + review queries.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

/// Creates a fresh temporary workdir and returns the TempDir guard + path string.
/// The guard must be kept alive for the duration of the benchmark iteration.
fn temp_workdir() -> (TempDir, String) {
    let tmp = tempfile::tempdir().expect("bench: create temp workdir");
    let path = tmp.path().to_str().unwrap().to_owned();
    (tmp, path)
}

// ─── Benchmark 1: octad workdir setup cost ──────────────────────────────────

/// Measures how long it takes to ingest a source file and set up the full
/// 8-modality octad directory tree from scratch.
fn bench_ingest_octad_setup(c: &mut Criterion) {
    c.bench_function("ingest_octad_setup", |b| {
        b.iter(|| {
            let (_tmp, wd) = temp_workdir();
            squeakwell::ingest::ingest(
                black_box("dump.sql"),
                black_box("sql-dump"),
                black_box(&wd),
            )
            .expect("bench ingest should not fail");
        });
    });
}

// ─── Benchmark 2: five-phase recovery loop ──────────────────────────────────

/// Measures the cost of the five-phase recovery orchestration.
/// The engine is a stub, so this benchmarks the dispatch overhead.
fn bench_recover_five_phases(c: &mut Criterion) {
    // Pre-create a single workdir shared across iterations to isolate just
    // the recovery loop cost (not the ingest setup).
    let (_tmp, wd) = temp_workdir();
    squeakwell::ingest::ingest("seed.json", "json", &wd)
        .expect("bench: seed ingest failed");

    c.bench_function("recover_stub_five_phase", |b| {
        b.iter(|| {
            squeakwell::engine::recover(
                black_box(&wd),
                black_box(6),
                black_box(0.05),
                black_box(100),
            )
            .expect("bench recover should not fail");
        });
    });
}

// ─── Benchmark 3: engine query batch ────────────────────────────────────────

/// Measures the combined cost of status + confidence + review_queue queries.
/// These are the read-path operations called during monitoring.
fn bench_engine_query_batch(c: &mut Criterion) {
    let (_tmp, wd) = temp_workdir();
    squeakwell::ingest::ingest("data.csv", "csv", &wd)
        .expect("bench: seed ingest failed");

    c.bench_function("engine_query_batch", |b| {
        b.iter(|| {
            squeakwell::engine::print_status(black_box(&wd))
                .expect("print_status");
            squeakwell::engine::print_confidence(black_box(&wd), black_box(0.8))
                .expect("print_confidence");
            squeakwell::engine::print_review_queue(black_box(&wd), black_box("json"))
                .expect("print_review_queue");
        });
    });
}

criterion_group!(
    benches,
    bench_ingest_octad_setup,
    bench_recover_five_phases,
    bench_engine_query_batch,
);
criterion_main!(benches);
