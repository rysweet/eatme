use super::RecoveryReadinessInput;

const DOCS_BUILD_COMMAND: &str = "mkdocs build --strict";

pub(super) fn collect_diff_scope(blockers: &mut Vec<String>, input: &RecoveryReadinessInput) {
    if !input.diff_scope.focused {
        blockers.push(format!(
            "focused diff scope evidence is required for {}",
            input.validation_sha
        ));
    }

    for file in &input.diff_scope.changed_files {
        if !is_focused_recovery_path(file) {
            blockers.push(format!(
                "focused diff scope excludes unrelated path {file}; keep recovery changes in readiness CLI, uvx wrapper, docs, or tests"
            ));
        }
    }
}

fn is_focused_recovery_path(file: &str) -> bool {
    file == ".pre-commit-config.yaml"
        || file == "crates/eatme-cli/src/main.rs"
        || file == "crates/eatme-core/src/command.rs"
        || file == "pyproject.toml"
        || file == "mkdocs.yml"
        || file == "scripts/check-module-size.sh"
        || file.starts_with("crates/eatme-cli/src/pr_readiness")
        || file == "src/eatme_uvx/cli.py"
        || file == "docs/default-workflow-pr-readiness.md"
        || file == "docs/pr-readiness-recovery-evaluation.md"
        || file == "docs/cli-usage.md"
        || file == "docs/index.md"
        || is_yaml_under(file, "assets/scenarios/eatme/")
        || is_yaml_under(file, "assets/scenarios/gadugi/")
}

fn is_yaml_under(file: &str, directory: &str) -> bool {
    file.starts_with(directory) && file.ends_with(".yaml")
}

pub(super) fn collect_docs_impact(blockers: &mut Vec<String>, input: &RecoveryReadinessInput) {
    if input.docs_impact.docs_changed && !input.docs_impact.strict_build_required {
        blockers.push(format!(
            "docs impact requires strict documentation build evidence with `{DOCS_BUILD_COMMAND}`"
        ));
    }

    if (input.docs_impact.docs_changed || input.docs_impact.strict_build_required)
        && (!input.documentation_build.passed || input.documentation_build.exit_status != 0)
    {
        blockers.push(format!(
            "docs impact is not satisfied for {}; `{DOCS_BUILD_COMMAND}` must pass when docs changed",
            input.validation_sha
        ));
    }
}
