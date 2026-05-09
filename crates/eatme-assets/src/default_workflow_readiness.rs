const REQUIRED_COMMANDS: [&str; 4] = [
    "cargo run -q -p eatme-cli -- assets validate --json",
    "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "mkdocs build --strict",
    "TMPDIR=/tmp ./scripts/quality-gates.sh",
];

const OVERCLAIMS: [&str; 9] = [
    "full ui automation",
    "visible rendering correctness",
    "rendering correctness",
    "grading",
    "creative assessment",
    "full lesson completion",
    "lesson completion",
    "full tweedle/player decode",
    "full save completion",
];

mod model;
pub use model::*;

mod github;
pub use github::*;

mod validators;
use validators::{validate_check_run, validate_recorded_commands, validate_required_commands};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessArtifact {
    marker: &'static str,
    text: String,
    blocker: Option<String>,
}

impl ReadinessArtifact {
    pub fn marker(&self) -> &str {
        self.marker
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn blocker(&self) -> &str {
        self.blocker.as_deref().unwrap_or("")
    }

    fn ready(input: &ReadinessInput) -> Self {
        let required_checks = check_names(input, |check| check.required);
        let optional_skipped_checks = check_names(input, |check| {
            !check.required && check.conclusion == CheckConclusion::Skipped
        });
        let text = format!(
            "MERGE_READY_EVIDENCE\n\
             PR: #{} ({})\n\
             Exact head: {}\n\
             Local head: {}\n\n\
             Command evidence for this head:\n\
             - cargo run -q -p eatme-cli -- assets validate --json: passed\n\
             - cargo run -q -p eatme-cli -- assets generate-gadugi --check --json: passed\n\
             - mkdocs build --strict: passed\n\
              - TMPDIR=/tmp ./scripts/quality-gates.sh: passed\n\n\
              GitHub evidence:\n\
              - required checks: {}\n\
              - optional skipped jobs: {}\n\
              - mergeStateStatus: {}\n\
              - mergeable: {}\n\n\
             Review evidence:\n\
             - diff scope: focused on the default-workflow PR readiness evidence lane\n\
             - docs impact: strict MkDocs passed; docs claim only bounded readiness evidence\n\
             - PR evidence: {} records this exact head and command evidence\n\
             - quality audit: three SEEK/VALIDATE/FIX cycles completed; final cycle clean\n\
             - no manual merge: no manual merge operation was run\n\n\
             Boundary: readiness covers only the documented exact-head evidence lane.",
            input.pr_number,
            input.head_ref_name,
            input.head_ref_oid,
            input.local_head_sha,
            required_checks,
            optional_skipped_checks,
            input.merge_state_status,
            input.mergeable,
            input.pr_evidence.location
        );
        Self {
            marker: "MERGE_READY_EVIDENCE",
            text,
            blocker: None,
        }
    }

    fn blocked(
        input: &ReadinessInput,
        blocker: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        let blocker = blocker.into();
        let text = format!(
            "NOT_MERGE_READY\n\
             PR: #{} ({})\n\
             Observed head: {}\n\
             Blocker: {}\n\
             Required next action: {}",
            input.pr_number,
            display_or_unknown(&input.head_ref_name),
            display_or_unknown(&input.head_ref_oid),
            blocker,
            action.into()
        );
        Self {
            marker: "NOT_MERGE_READY",
            text,
            blocker: Some(blocker),
        }
    }
}

pub struct HeadVerification;

impl HeadVerification {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        if input.head_ref_oid.trim().is_empty() {
            return Err(ReadinessArtifact::blocked(
                input,
                "PR headRefOid is empty",
                "fetch live PR metadata again before recording readiness evidence",
            ));
        }
        if input.local_branch != input.head_ref_name {
            return Err(ReadinessArtifact::blocked(
                input,
                format!(
                    "local branch '{}' does not match PR head branch '{}'",
                    input.local_branch, input.head_ref_name
                ),
                "check out the current PR head branch before recording evidence",
            ));
        }
        if input.local_head_sha != input.head_ref_oid {
            return Err(ReadinessArtifact::blocked(
                input,
                format!(
                    "local head '{}' does not match PR headRefOid '{}'",
                    input.local_head_sha, input.head_ref_oid
                ),
                "fetch the PR head and rerun evidence for the matching SHA",
            ));
        }
        Ok(())
    }
}

pub struct EvidenceCommands;

impl EvidenceCommands {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        validate_required_commands(input)
    }
}

pub struct QualityAuditCycles;

impl QualityAuditCycles {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        if input.quality_audit_cycles.len() < 3 {
            return Err(ReadinessArtifact::blocked(
                input,
                "quality audit requires at least three SEEK/VALIDATE/FIX cycles",
                "complete three quality-audit cycles and make the final cycle clean",
            ));
        }

        for (index, cycle) in input.quality_audit_cycles.iter().enumerate() {
            let number = index + 1;
            if cycle.seek.trim().is_empty() {
                return Err(missing_cycle_part(input, number, "SEEK"));
            }
            if cycle.validate.trim().is_empty() {
                return Err(missing_cycle_part(input, number, "VALIDATE"));
            }
            if cycle.fix.trim().is_empty() {
                return Err(missing_cycle_part(input, number, "FIX"));
            }
        }

        if !input
            .quality_audit_cycles
            .last()
            .map(|cycle| cycle.clean)
            .unwrap_or(false)
        {
            return Err(ReadinessArtifact::blocked(
                input,
                "quality audit final cycle is not clean",
                "resolve the final-cycle findings or record a current-head no-op rationale",
            ));
        }
        Ok(())
    }
}

pub struct DiffScopeReview;

impl DiffScopeReview {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        if input.changed_files.is_empty() {
            return Err(ReadinessArtifact::blocked(
                input,
                "diff scope evidence is missing changed files",
                "review the PR diff and record the focused changed-file scope",
            ));
        }

        if let Some(path) = input
            .changed_files
            .iter()
            .find(|path| !is_focused_readiness_path(path))
        {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("unrelated file in readiness diff scope: {path}"),
                "remove unrelated changes or split them from the readiness PR",
            ));
        }
        Ok(())
    }
}

impl DocsImpactReview {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        if !input.docs_impact.mkdocs_strict_passed {
            return Err(ReadinessArtifact::blocked(
                input,
                "mkdocs build --strict has not passed for the docs impact",
                "run and record strict MkDocs evidence for the current head",
            ));
        }

        if let Some(claim) = input
            .docs_impact
            .bounded_claims
            .iter()
            .find(|claim| contains_overclaim(claim))
        {
            return Err(ReadinessArtifact::blocked(
                input,
                format!("unsupported docs overclaim: {claim}"),
                "replace the claim with bounded evidence proven by current-head validation",
            ));
        }
        Ok(())
    }
}

pub struct GitHubActionsReview;

impl GitHubActionsReview {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        if input.check_runs.is_empty() {
            return Err(ReadinessArtifact::blocked(
                input,
                "GitHub Actions check evidence is missing",
                "wait for current-head check runs and record their conclusions",
            ));
        }

        for check in &input.check_runs {
            validate_check_run(input, check)?;
        }
        Ok(())
    }
}

impl PREvidenceReview {
    pub fn validate(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
        let evidence = &input.pr_evidence;
        if !evidence.trusted_provenance {
            return Err(ReadinessArtifact::blocked(
                input,
                "PR evidence source is not trusted",
                "record current-head evidence in the same-repository PR body or a trusted PR comment",
            ));
        }

        if evidence.head_sha != input.head_ref_oid {
            return Err(ReadinessArtifact::blocked(
                input,
                "stale PR evidence does not name the current head",
                "update the PR body or comment with current-head evidence",
            ));
        }

        validate_recorded_commands(input)?;

        for (recorded, item) in [
            (evidence.records_github_checks, "GitHub checks"),
            (evidence.records_diff_scope, "diff scope"),
            (evidence.records_docs_impact, "docs impact"),
            (evidence.records_quality_audit, "quality audit"),
            (evidence.records_no_manual_merge, "no manual merge"),
        ] {
            if !recorded {
                return Err(missing_pr_evidence(input, item));
            }
        }

        if evidence.updated_during_review
            && evidence.reconfirmed_head_sha.as_deref() != Some(input.head_ref_oid.as_str())
        {
            return Err(ReadinessArtifact::blocked(
                input,
                "PR evidence update was not followed by a head reconfirmation",
                "reconfirm the PR head after the evidence update",
            ));
        }
        Ok(())
    }
}

pub struct MergeReadyGate;

impl MergeReadyGate {
    pub fn evaluate(input: ReadinessInput) -> ReadinessArtifact {
        for validate in [
            HeadVerification::validate,
            EvidenceCommands::validate,
            QualityAuditCycles::validate,
            DiffScopeReview::validate,
            DocsImpactReview::validate,
            GitHubActionsReview::validate,
            PREvidenceReview::validate,
            validate_merge_state,
            validate_no_manual_merge,
        ] {
            if let Err(artifact) = validate(&input) {
                return artifact;
            }
        }
        ReadinessArtifact::ready(&input)
    }
}

fn validate_merge_state(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
    if input.merge_state_status != "CLEAN" {
        return Err(ReadinessArtifact::blocked(
            input,
            format!("mergeStateStatus is '{}'", input.merge_state_status),
            "wait for or resolve the PR merge state before readiness",
        ));
    }
    if input.mergeable != "MERGEABLE" {
        return Err(ReadinessArtifact::blocked(
            input,
            format!("mergeable is '{}'", input.mergeable),
            "resolve mergeability before readiness",
        ));
    }
    Ok(())
}

fn validate_no_manual_merge(input: &ReadinessInput) -> Result<(), ReadinessArtifact> {
    if input.manual_merge_attempted {
        return Err(ReadinessArtifact::blocked(
            input,
            "manual merge operation was attempted",
            "discard the manual merge path and use PR review/merge automation only",
        ));
    }
    Ok(())
}

fn missing_cycle_part(input: &ReadinessInput, number: usize, part: &str) -> ReadinessArtifact {
    ReadinessArtifact::blocked(
        input,
        format!("quality audit cycle {number} is missing {part}"),
        "complete every SEEK/VALIDATE/FIX part before readiness",
    )
}

fn missing_pr_evidence(input: &ReadinessInput, item: &str) -> ReadinessArtifact {
    ReadinessArtifact::blocked(
        input,
        format!("PR evidence is missing {item}"),
        "update the PR body or comment with the missing current-head evidence",
    )
}

fn is_focused_readiness_path(path: &str) -> bool {
    matches!(
        path,
        ".pre-commit-config.yaml"
            | "Cargo.lock"
            | "crates/eatme-assets/Cargo.toml"
            | "docs/default-workflow-pr-readiness.md"
            | "docs/lesson-session-readiness.md"
            | "pyproject.toml"
            | "crates/eatme-assets/src/lib.rs"
            | "crates/eatme-assets/src/default_workflow_readiness.rs"
            | "crates/eatme-assets/src/lesson_session_readiness_doc_tests.rs"
    ) || path.starts_with("crates/eatme-assets/src/default_workflow_readiness/")
        || path.starts_with("crates/eatme-assets/tests/default_workflow_readiness")
}

fn contains_overclaim(claim: &str) -> bool {
    let claim = claim.to_lowercase();
    OVERCLAIMS.iter().any(|overclaim| claim.contains(overclaim))
}

fn display_or_unknown(value: &str) -> &str {
    if value.trim().is_empty() {
        "unknown"
    } else {
        value
    }
}

fn check_names(input: &ReadinessInput, include: impl Fn(&CheckRunEvidence) -> bool) -> String {
    let names = input
        .check_runs
        .iter()
        .filter(|check| include(check))
        .map(|check| check.name.as_str())
        .collect::<Vec<_>>();
    if names.is_empty() {
        "none".into()
    } else {
        names.join(", ")
    }
}
