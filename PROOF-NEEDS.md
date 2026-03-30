# PROOF-NEEDS.md
<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->

## Current State

- **LOC**: ~3,470
- **Languages**: Rust, Idris2, Zig
- **Existing ABI proofs**: `src/interface/abi/*.idr` (template-level)
- **Dangerous patterns**: None detected

## What Needs Proving

### Data Recovery Engine (src/engine/)
- Octad recovery — data reconstruction from partial records
- Prove: recovery algorithm produces correct reconstructions when sufficient data is available
- Prove: recovery reports failure (not corrupt data) when data is insufficient

### Ingestion Pipeline (src/ingest/)
- Data ingestion and validation
- Prove: ingestion preserves data integrity (no silent corruption)

### Manifest Handling (src/manifest/)
- Recovery manifest specification
- Prove: manifest accurately describes recoverable data state

### ABI Layer (src/abi/)
- Rust ABI module — should be backed by Idris2 contracts

## Recommended Prover

- **Idris2** for recovery correctness invariants

## Priority

**MEDIUM** — Data recovery tool where incorrect recovery is worse than no recovery. The "never return corrupt data" invariant is safety-critical and should be proven.

## Template ABI Cleanup (2026-03-29)

Template ABI removed -- was creating false impression of formal verification.
The removed files (Types.idr, Layout.idr, Foreign.idr) contained only RSR template
scaffolding with unresolved {{PROJECT}}/{{AUTHOR}} placeholders and no domain-specific proofs.
