// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// ABI module for SqueakWell.
//
// STATUS (2026-11-04): no Idris2 proofs exist in this repository yet. The
// template ABI was removed on 2026-03-29 because it gave a false impression
// of formal verification (see PROOF-NEEDS.adoc). This file records the
// invariants the engine is designed to satisfy, in the form that an honest
// mechanisation can actually target. The proof route, prover homes and
// the rungs that already exist upstream are in docs/ROUTE-PLAN.adoc.
//
// The five invariants, restated after adversarial review against
// verisimdb/formal/*.v and vcl-ut/src/core/*.idr:
//
//   Progress (replaces "MonotonicConvergence"):
//     Real-valued drift is NOT monotone under fill-then-resolve: filling a
//     missing modality turns an unmeasurable pair into a measurable,
//     possibly disagreeing one, so drift can rise in Phase 2. The honest
//     measure is lexicographic on naturals:
//       mu(o) = (|missing o|, nconflicts o)
//     and the theorem is: step o = o  \/  mu(step o) <lex mu(o).
//     Guarded fill (None -> Some only) is inflationary and monotone on the
//     information order of partial octads; resolve copies the winner and
//     never un-fills. Fixpoint within <= 8 fill steps (modality_cardinality)
//     plus one resolve pass (normalize_idempotent).
//
//   IdentityPreservation:
//     Every recovery operation preserves the entity identifier
//     (verisimdb Octad.v: project_preserves_id, merge_preserves_id,
//     enrich_preserves_id — a 3-line induction gives the iterated form),
//     and guarded fill never overwrites an observed value. "Inferred agrees
//     with observed" is NOT a theorem; disagreement is exactly what Phase 3
//     resolves and what the review queue records.
//
//   DeterministicResolution:
//     resolve is a pure function of its inputs and invariant under
//     permutation of the fragment list (no clock, no caller ordering).
//     This holds for ModalityPriority; LastWriterWins needs explicit
//     timestamps carried IN the input, not caller order.
//
//   TypeRatchetSoundness:
//     Certified level N implies certified at every level below N. Already
//     proven for VCL-UT statements and transitions (Levels.idr); squeakwell
//     only instantiates it, by grading the canonical DECLARE / NORMALISE
//     transition that would reintroduce each recovered entity. Only L0-L6
//     are expressible from VCL source today (vcl-ut #25), so the ratchet is
//     capped at L6 until that gap closes.
//
//   Termination (replaces "TerminationGuarantee"):
//     The lexicographic measure is well-founded on nat*nat, so the loop
//     reaches a fixpoint without needing max_iterations; max_iterations is
//     an operational budget, not the termination argument.
//
// Soundness of the output ("never export corrupt data") is stated
// separately, in echo-types vocabulary: every exported entity inhabits the
// fibre of the damage map over the observed fragments, and every
// non-singleton fibre is in the review queue.
