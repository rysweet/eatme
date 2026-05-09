use super::GitHubPullRequest;
use crate::default_workflow_readiness::PREvidenceReview;

pub(super) fn pr_evidence_from_texts(pull_request: &GitHubPullRequest) -> PREvidenceReview {
    if pull_request.head_ref_oid.trim().is_empty() {
        return empty_pr_evidence();
    }

    let mut first_trusted = None;
    let mut head_evidence = None;

    for text in &pull_request.evidence_texts {
        if !text.trusted {
            continue;
        }
        first_trusted.get_or_insert(text);
        if text.body.contains(&pull_request.head_ref_oid) {
            head_evidence = Some(text);
            break;
        }
    }

    let Some((evidence_text, names_head_sha)) = head_evidence
        .map(|text| (text, true))
        .or_else(|| first_trusted.map(|text| (text, false)))
    else {
        return empty_pr_evidence();
    };

    PREvidenceReview {
        location: evidence_text.location.clone(),
        trusted_provenance: true,
        head_sha: if names_head_sha {
            pull_request.head_ref_oid.clone()
        } else {
            String::new()
        },
        recorded_commands: super::super::REQUIRED_COMMANDS
            .iter()
            .filter(|command| evidence_text.body.contains(**command))
            .map(|command| (*command).into())
            .collect(),
        records_github_checks: contains_ascii_case_insensitive(&evidence_text.body, "github")
            && contains_ascii_case_insensitive(&evidence_text.body, "check"),
        records_diff_scope: contains_ascii_case_insensitive(&evidence_text.body, "diff")
            && contains_ascii_case_insensitive(&evidence_text.body, "scope"),
        records_docs_impact: contains_ascii_case_insensitive(&evidence_text.body, "docs")
            && contains_ascii_case_insensitive(&evidence_text.body, "impact"),
        records_quality_audit: contains_ascii_case_insensitive(
            &evidence_text.body,
            "quality audit",
        ),
        records_no_manual_merge: contains_ascii_case_insensitive(
            &evidence_text.body,
            "no manual merge",
        ),
        updated_during_review: false,
        reconfirmed_head_sha: None,
    }
}

pub(super) fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn empty_pr_evidence() -> PREvidenceReview {
    PREvidenceReview {
        location: "missing".into(),
        trusted_provenance: false,
        head_sha: String::new(),
        recorded_commands: Vec::new(),
        records_github_checks: false,
        records_diff_scope: false,
        records_docs_impact: false,
        records_quality_audit: false,
        records_no_manual_merge: false,
        updated_during_review: false,
        reconfirmed_head_sha: None,
    }
}
