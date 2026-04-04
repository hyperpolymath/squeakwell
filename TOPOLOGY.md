<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# TOPOLOGY.md — squeakwell

## Purpose

Octad-Recover (working title) is a database recovery and reconstruction tool using VeriSimDB's octad model and VQL-UT's progressive type system to reassemble damaged, fragmented, or inconsistent datasets through cross-modal constraint propagation. Enables recovery from incomplete or corrupted data via formal verification and constraint solving.

## Module Map

```
squeakwell/
├── src/                 # Rust implementation
│   ├── octad/          # Octad model core
│   ├── constraint/     # Constraint propagation engine
│   ├── recovery/       # Recovery algorithms
│   └── veridb/         # VeriSimDB integration
├── tests/              # Comprehensive test suite
├── benches/            # Performance benchmarks
└── Cargo.toml          # Rust package manifest
```

## Data Flow

```
[Damaged Dataset] ──► [Fragment Detection] ──► [Constraint Mapping] ──► [Octad Model]
                                                       ↓
                                              [Type Inference (VQL-UT)] ──► [Reconstruction]
                                                       ↓
                                              [Verification] ──► [Recovered Data]
```

## Key Components

- **Octad model**: VeriSimDB's data structure representation
- **Constraint propagation**: Cross-modal consistency enforcement
- **Type system**: VQL-UT progressive types guide recovery
- **Verification**: Formal proofs of recovery correctness
