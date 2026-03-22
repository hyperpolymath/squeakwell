#![forbid(unsafe_code)]
// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SqueakWell — database recovery through cross-modal constraint propagation.
// Your database is squealing? SqueakWell makes it well.
//
// Uses VeriSimDB's octad (8 modalities) as independent witnesses that
// cross-check and reconstruct each other. Progressive VQL-UT type levels
// act as a ratchet — data only gets more consistent, never less.
// Recovery is complete when cross-modal drift approaches zero.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod abi;
mod engine;
mod ingest;
mod manifest;

/// SqueakWell — stop your database squealing.
/// Cross-modal constraint propagation recovery.
#[derive(Parser)]
#[command(name = "squeakwell", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest a damaged database, dump, or fragment set.
    Ingest {
        /// Path to damaged data (SQL dump, directory of fragments, WAL, etc.)
        source: String,
        /// Source format: auto | sql-dump | json | csv | wal | binary | directory
        #[arg(short, long, default_value = "auto")]
        format: String,
        /// Working directory for the recovery octad
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
    },

    /// Run recovery: progressive constraint propagation across all 5 phases.
    Recover {
        /// Working directory containing ingested octad
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
        /// Maximum VQL-UT level to target (1-10, default 6)
        #[arg(short, long, default_value = "6")]
        target_level: u8,
        /// Stop when drift drops below this threshold (0.0-1.0)
        #[arg(short = 'd', long, default_value = "0.05")]
        drift_threshold: f64,
        /// Maximum iterations before giving up
        #[arg(long, default_value = "100")]
        max_iterations: u32,
    },

    /// Show current recovery status: drift scores, phase, confidence.
    Status {
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
    },

    /// Show per-entity confidence scores and flag low-confidence entities.
    Confidence {
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
        /// Show only entities below this confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        below: f64,
    },

    /// Export the human review queue — entities that couldn't be auto-resolved.
    Review {
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
        /// Output format: human | json | csv
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Export the recovered database.
    Export {
        #[arg(short, long, default_value = ".squeakwell")]
        workdir: String,
        /// Output format: verisimdb | sql | json | csv
        #[arg(short, long, default_value = "verisimdb")]
        format: String,
        /// Output path
        #[arg(short, long)]
        output: String,
    },

    /// Explain the 5 recovery phases and how they work.
    Explain,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { source, format, workdir } => {
            println!("SqueakWell: ingesting damaged data from {} (format: {})", source, format);
            ingest::ingest(&source, &format, &workdir)?;
        }
        Commands::Recover { workdir, target_level, drift_threshold, max_iterations } => {
            println!("SqueakWell: starting recovery");
            println!("  Target level: {}/10, drift threshold: {}, max iterations: {}",
                target_level, drift_threshold, max_iterations);
            engine::recover(&workdir, target_level, drift_threshold, max_iterations)?;
        }
        Commands::Status { workdir } => {
            engine::print_status(&workdir)?;
        }
        Commands::Confidence { workdir, below } => {
            engine::print_confidence(&workdir, below)?;
        }
        Commands::Review { workdir, format } => {
            engine::print_review_queue(&workdir, &format)?;
        }
        Commands::Export { workdir, format, output } => {
            engine::export(&workdir, &format, &output)?;
        }
        Commands::Explain => explain_phases(),
    }
    Ok(())
}

fn explain_phases() {
    println!("=== SqueakWell: 5-Phase Recovery ===");
    println!();
    println!("  PHASE 1 — LOOSE ACCEPTANCE (VQL-UT Levels 1-3)");
    println!("    Scatter fragments across octad modalities.");
    println!("    Accept anything structurally valid.");
    println!("    Drift score: ~0.8 (high — modalities disagree)");
    println!();
    println!("  PHASE 2 — CROSS-MODAL INFERENCE");
    println!("    Use populated modalities to infer missing ones.");
    println!("    Document → Graph (NER), Document → Vector (encode),");
    println!("    Graph → Semantic (type inference), Provenance → Temporal.");
    println!("    Drift score: ~0.5 (filling in, contradictions emerging)");
    println!();
    println!("  PHASE 3 — CONFLICT RESOLUTION");
    println!("    Resolve contradictions between observed and inferred data.");
    println!("    Arbitrate via: provenance authority, temporal recency,");
    println!("    semantic type validity, cardinality consistency.");
    println!("    Drift score: ~0.3 (contradictions resolved)");
    println!();
    println!("  PHASE 4 — TYPE TIGHTENING (VQL-UT Levels 4-6)");
    println!("    Apply null-safety, injection-proofing, result-type checking.");
    println!("    Data failing these levels flagged for human review.");
    println!("    Drift score: ~0.15");
    println!();
    println!("  PHASE 5 — CONVERGENCE (VQL-UT Levels 7-10)");
    println!("    Cardinality bounds, effect tracking, temporal consistency,");
    println!("    linearity. Formally verified recovery.");
    println!("    Drift score: approaches 0.0");
    println!();
    println!("  Recovery is complete when drift < threshold for all entities.");
    println!("  Entities that can't converge go to the human review queue.");
}
