//! Events-and-collision grading — to be extracted from grading_report.rs.
//!
//! Contains `EventsGradingInput`, `grade_events_and_collision`, and event-specific
//! AST helpers. Shared helpers are imported from `crate::grading_report`.

#![allow(dead_code, unused_imports)]

use eatme_core::ast::Program;

// Re-export shared types so `use super::*` works in the test file
pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

/// Input struct for events-and-collision grading.
pub struct EventsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

/// Grade a student's events-and-collision lesson attempt.
pub fn grade_events_and_collision(_input: EventsGradingInput) -> GradingReport {
    todo!("extract implementation from grading_report.rs lines 357–512")
}
