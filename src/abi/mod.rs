// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// ABI module for SqueakWell.
// Idris2 proofs for recovery correctness:
//
//   MonotonicConvergence:
//     ∀ phase p, iteration i.
//       drift(p, i+1) ≤ drift(p, i)
//     Constraint propagation never increases drift.
//
//   IdentityPreservation:
//     ∀ entity e, modality m₁ m₂.
//       infer(e, m₁) consistent_with observe(e, m₂)
//     Cross-modal inference preserves entity identity.
//
//   DeterministicResolution:
//     ∀ conflict c, input I.
//       resolve(c, I) = resolve(c, I)
//     Same conflict + same input → same resolution. No randomness.
//
//   TypeRatchetSoundness:
//     ∀ entity e, level n.
//       passes(e, n) → ∀ m ≤ n. passes(e, m)
//     Passing level N implies passing all levels below N.
//
//   TerminationGuarantee:
//     ∀ workload w. ∃ i. drift(w, i) < threshold ∨ i = max_iterations
//     Recovery always terminates (either converges or gives up).
