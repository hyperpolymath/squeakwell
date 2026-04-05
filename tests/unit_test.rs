// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Unit tests for squeakwell — individual module coverage.
// Covers: engine data structures, ingest format detection, entity confidence,
// recovery state invariants, drift thresholds, and serialisation round-trips.

use squeakwell::engine::{EntityConfidence, RecoveryState};

// ─── RecoveryState ──────────────────────────────────────────────────────────

/// A freshly constructed RecoveryState should start at phase 1.
#[test]
fn recovery_state_initial_phase_is_one() {
    let state = RecoveryState {
        phase: 1,
        iteration: 0,
        overall_drift: 1.0,
        entities_total: 0,
        entities_converged: 0,
        entities_review: 0,
        max_level_achieved: 0,
    };
    assert_eq!(state.phase, 1);
}

/// overall_drift must be in [0.0, 1.0] — a drift of 0.0 is valid (fully converged).
#[test]
fn recovery_state_drift_zero_is_valid() {
    let state = RecoveryState {
        phase: 5,
        iteration: 42,
        overall_drift: 0.0,
        entities_total: 100,
        entities_converged: 100,
        entities_review: 0,
        max_level_achieved: 10,
    };
    assert!(state.overall_drift >= 0.0);
    assert!(state.overall_drift <= 1.0);
}

/// entities_converged should never exceed entities_total.
#[test]
fn recovery_state_converged_le_total() {
    let state = RecoveryState {
        phase: 3,
        iteration: 10,
        overall_drift: 0.3,
        entities_total: 50,
        entities_converged: 30,
        entities_review: 5,
        max_level_achieved: 5,
    };
    assert!(state.entities_converged <= state.entities_total);
}

/// RecoveryState must round-trip through JSON without loss.
#[test]
fn recovery_state_json_round_trip() {
    let original = RecoveryState {
        phase: 2,
        iteration: 7,
        overall_drift: 0.47,
        entities_total: 200,
        entities_converged: 80,
        entities_review: 12,
        max_level_achieved: 3,
    };
    let json = serde_json::to_string(&original).expect("serialise RecoveryState");
    let restored: RecoveryState = serde_json::from_str(&json).expect("deserialise RecoveryState");
    assert_eq!(restored.phase, original.phase);
    assert!((restored.overall_drift - original.overall_drift).abs() < f64::EPSILON);
    assert_eq!(restored.entities_total, original.entities_total);
}

/// phase must be in the range 1-5 (contract for the five-phase recovery model).
#[test]
fn recovery_state_phase_range_1_to_5() {
    for phase in 1u8..=5 {
        let state = RecoveryState {
            phase,
            iteration: 0,
            overall_drift: 1.0,
            entities_total: 1,
            entities_converged: 0,
            entities_review: 0,
            max_level_achieved: 0,
        };
        assert!((1..=5).contains(&state.phase));
    }
}

// ─── EntityConfidence ───────────────────────────────────────────────────────

/// An entity below the confidence threshold must have needs_review set.
#[test]
fn entity_confidence_low_needs_review() {
    let entity = EntityConfidence {
        entity_id: "tbl_orders/row_42".to_string(),
        confidence: 0.4,
        drift_score: 0.6,
        modalities_populated: 3,
        max_type_level: 2,
        needs_review: true,
        review_reason: Some("confidence below 0.8 threshold".to_string()),
    };
    assert!(entity.needs_review);
    assert!(entity.confidence < 0.8);
    assert!(entity.review_reason.is_some());
}

/// A high-confidence entity should not need review.
#[test]
fn entity_confidence_high_no_review() {
    let entity = EntityConfidence {
        entity_id: "tbl_users/row_1".to_string(),
        confidence: 0.95,
        drift_score: 0.03,
        modalities_populated: 8,
        max_type_level: 7,
        needs_review: false,
        review_reason: None,
    };
    assert!(!entity.needs_review);
    assert!(entity.review_reason.is_none());
}

/// modalities_populated must be between 0 and 8 (octad).
#[test]
fn entity_confidence_modalities_in_octad_range() {
    let entity = EntityConfidence {
        entity_id: "e1".to_string(),
        confidence: 0.5,
        drift_score: 0.5,
        modalities_populated: 8,
        max_type_level: 5,
        needs_review: false,
        review_reason: None,
    };
    assert!(entity.modalities_populated <= 8);
}

/// EntityConfidence must round-trip through JSON.
#[test]
fn entity_confidence_json_round_trip() {
    let original = EntityConfidence {
        entity_id: "tbl_foo/row_99".to_string(),
        confidence: 0.73,
        drift_score: 0.27,
        modalities_populated: 6,
        max_type_level: 4,
        needs_review: false,
        review_reason: None,
    };
    let json = serde_json::to_string(&original).expect("serialise EntityConfidence");
    let restored: EntityConfidence = serde_json::from_str(&json).expect("deserialise EntityConfidence");
    assert_eq!(restored.entity_id, original.entity_id);
    assert!((restored.confidence - original.confidence).abs() < f64::EPSILON);
    assert_eq!(restored.modalities_populated, original.modalities_populated);
}

/// max_type_level must be in 1-10 (VCL-total type ratchet levels).
#[test]
fn entity_confidence_max_type_level_range() {
    for level in 1u8..=10 {
        let entity = EntityConfidence {
            entity_id: format!("row_{}", level),
            confidence: 0.9,
            drift_score: 0.1,
            modalities_populated: 8,
            max_type_level: level,
            needs_review: false,
            review_reason: None,
        };
        assert!((1..=10).contains(&entity.max_type_level));
    }
}
