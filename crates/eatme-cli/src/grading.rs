use anyhow::{Result, bail};
use clap::Args;
use eatme_alice::check_dependencies;
use eatme_core::CommandRunner;
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
use eatme_core::{CommandOutput, CommandSpec};

#[derive(Args)]
pub struct AssetsGradingReportArgs {
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
}

pub fn run_grading_report(
    args: &AssetsGradingReportArgs,
    runner: &impl CommandRunner,
) -> Result<()> {
    let report = build_grading_report(args, runner)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.passed {
        bail!("first-lesson grading report: not all steps ready");
    }
    Ok(())
}

fn build_grading_report(
    args: &AssetsGradingReportArgs,
    runner: &impl CommandRunner,
) -> Result<eatme_assets::GradingReport> {
    let ar = eatme_assets::validate_assets(Path::new(&args.path))?;
    let dr = check_dependencies(runner)?;
    let asset_reason = if ar.passed {
        format!(
            "All {} scenario assets passed validation",
            ar.scenario_asset_count
        )
    } else {
        format!("{} errors found", ar.errors.len())
    };
    let missing: Vec<_> = dr
        .tools
        .iter()
        .filter(|(_, v)| !*v)
        .map(|(k, _)| k.as_str())
        .collect();
    let deps_reason: String = if dr.all_required_available {
        "All required tools available".into()
    } else {
        format!("Missing required tools: {}", missing.join(", "))
    };
    Ok(eatme_assets::grade_first_lesson_readiness(
        eatme_assets::GradingInput {
            assets_valid: ar.passed,
            asset_reason,
            deps_available: dr.all_required_available,
            deps_reason,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeRunner {
        missing: &'static str,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            let command = spec.shell_display();
            let exit_status = if command.contains(self.missing) { 1 } else { 0 };
            Ok(CommandOutput {
                command,
                exit_status: Some(exit_status),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn build_grading_report_marks_preconditions_ready_without_blocking_interactive_steps() {
        let args = AssetsGradingReportArgs {
            path: repo_root(),
            json: true,
        };

        let report = build_grading_report(
            &args,
            &FakeRunner {
                missing: "__none__",
            },
        )
        .unwrap();

        assert!(!report.passed);
        assert_eq!(report.lesson, "building-a-scene-first-world");
        assert_eq!(report.steps[0].name, "validate-assets");
        assert_eq!(report.steps[0].status, eatme_assets::StepStatus::Ready);
        assert_eq!(report.steps[1].name, "check-dependencies");
        assert_eq!(report.steps[1].status, eatme_assets::StepStatus::Ready);
        assert_eq!(report.steps[2].name, "launch-smoke");
        assert_eq!(report.steps[2].status, eatme_assets::StepStatus::Ready);
        assert!(
            report
                .steps
                .iter()
                .skip(3)
                .all(|step| step.status == eatme_assets::StepStatus::NotYetTested)
        );
    }

    #[test]
    fn run_grading_report_fails_when_required_dependency_is_missing() {
        let args = AssetsGradingReportArgs {
            path: repo_root(),
            json: true,
        };

        let error = run_grading_report(
            &args,
            &FakeRunner {
                missing: "command -v Xvfb",
            },
        )
        .expect_err("missing Xvfb should block the grading report");

        assert!(error.to_string().contains("not all steps ready"));
    }
}
