// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Ingest engine — accepts damaged databases and maps fragments to octad modalities.
// Handles: SQL dumps, JSON/CSV fragments, corrupted binary, partial WAL logs,
// orphaned indexes, detached TOAST data, broken foreign keys.

use anyhow::Result;

/// Ingest damaged data into the recovery working directory.
pub fn ingest(source: &str, format: &str, workdir: &str) -> Result<()> {
    std::fs::create_dir_all(workdir)?;

    let detected_format = if format == "auto" { detect_format(source)? } else { format.to_string() };
    println!("  Detected format: {}", detected_format);

    // TODO: per-format ingest handlers
    // Each handler scatters recoverable data across octad modalities:
    //   workdir/graph/      — RDF triples, edges, relationships
    //   workdir/vector/     — embeddings, feature vectors
    //   workdir/tensor/     — multi-dimensional numeric data
    //   workdir/semantic/   — type annotations, schema fragments
    //   workdir/document/   — text content, full-text fragments
    //   workdir/temporal/   — timestamps, version markers, WAL sequences
    //   workdir/provenance/ — origin markers, checksums, source metadata
    //   workdir/spatial/    — coordinates, geometries, spatial indexes

    for modality in ["graph", "vector", "tensor", "semantic", "document", "temporal", "provenance", "spatial"] {
        std::fs::create_dir_all(format!("{}/{}", workdir, modality))?;
    }

    println!("  Octad working directory prepared: {}", workdir);
    println!("  Ingested from: {}", source);
    Ok(())
}

fn detect_format(source: &str) -> Result<String> {
    let path = std::path::Path::new(source);
    if path.is_dir() { return Ok("directory".to_string()); }
    match path.extension().and_then(|e| e.to_str()) {
        Some("sql") => Ok("sql-dump".to_string()),
        Some("json" | "jsonl" | "ndjson") => Ok("json".to_string()),
        Some("csv" | "tsv") => Ok("csv".to_string()),
        Some("wal") => Ok("wal".to_string()),
        _ => Ok("binary".to_string()),
    }
}
