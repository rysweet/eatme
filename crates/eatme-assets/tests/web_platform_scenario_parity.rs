use serde_yaml::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_yaml_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("yaml"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn read_yaml(path: &Path) -> Value {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    value_at(value, path)?.as_str().map(str::to_owned)
}

fn string_list_at(value: &Value, path: &[&str]) -> Vec<String> {
    value_at(value, path)
        .and_then(Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn map_keys_at(value: &Value, path: &[&str]) -> BTreeSet<String> {
    value_at(value, path)
        .and_then(Value::as_mapping)
        .map(|mapping| {
            mapping
                .keys()
                .filter_map(|key| key.as_str().map(str::to_owned))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn scenario_id(path: &Path, value: &Value) -> String {
    string_at(value, &["id"]).unwrap_or_else(|| {
        path.file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned()
    })
}

fn desktop_scenarios(root: &Path) -> Vec<(String, Value)> {
    scenario_yaml_paths(&root.join("assets/scenarios/eatme"))
        .into_iter()
        .map(|path| {
            let yaml = read_yaml(&path);
            (scenario_id(&path, &yaml), yaml)
        })
        .collect()
}

fn source_lookingglass_targeted_desktop_ids(root: &Path) -> BTreeSet<String> {
    desktop_scenarios(root)
        .into_iter()
        .filter(|(_, yaml)| {
            string_list_at(yaml, &["adapter", "targets"])
                .iter()
                .any(|target| target == "lookingglass")
        })
        .map(|(id, _)| id)
        .collect()
}

fn generated_gadugi_adapter_source_ids(root: &Path) -> BTreeSet<String> {
    scenario_yaml_paths(&root.join("assets/scenarios/gadugi"))
        .into_iter()
        .filter_map(|path| {
            let yaml = read_yaml(&path);
            let source = string_at(&yaml, &["metadata", "source_eatme_asset"])?;
            Path::new(&source)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_owned)
        })
        .collect()
}

fn matrix_lookingglass_supported_ids(root: &Path) -> BTreeSet<String> {
    let yaml = read_yaml(&root.join("assets/parity/rabbithole-lookingglass-journey-matrix.yaml"));
    value_at(&yaml, &["rows"])
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let status = string_at(row, &["looking_glass", "status"])?;
            if status == "covered" || status == "partial" {
                string_at(row, &["scenario"])
            } else {
                None
            }
        })
        .collect()
}

fn matrix_lookingglass_not_supported_ids(root: &Path) -> BTreeSet<String> {
    let yaml = read_yaml(&root.join("assets/parity/rabbithole-lookingglass-journey-matrix.yaml"));
    value_at(&yaml, &["rows"])
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let status = string_at(row, &["looking_glass", "status"])?;
            if status == "not_supported" {
                string_at(row, &["scenario"])
            } else {
                None
            }
        })
        .collect()
}

fn lookingglass_evidenced_desktop_ids(root: &Path) -> BTreeSet<String> {
    source_lookingglass_targeted_desktop_ids(root)
        .union(&matrix_lookingglass_supported_ids(root))
        .cloned()
        .collect()
}

const CORE_CURRICULUM_SCENARIOS: &[&str] = &[
    "hour-of-code-studio-kickoff",
    "building-a-scene-first-world",
    "code-editor-first-run",
    "events-collision-proximity-game",
    "functions-as-questions-about-the-world",
    "loops-and-conditionals-mini-challenge",
    "reusable-methods-and-parameters",
];

const ROUND_86_TARGETED_WEB_SCENARIOS: &[&str] = &[
    "alien-linguist-parameter-dialogue",
    "lost-robot-debug-museum",
    "audio-camera-and-export-sharecase",
    "time-travel-recipe-sequencing",
    "modified-class-portability",
];

#[test]
fn desktop_scenarios_report_web_parity_and_core_curriculum_has_equivalents() {
    let root = repository_root();
    let desktop_ids = desktop_scenarios(&root)
        .into_iter()
        .map(|(id, _)| id)
        .collect::<BTreeSet<_>>();
    let lookingglass_evidenced_ids = lookingglass_evidenced_desktop_ids(&root);
    let generated_ids = generated_gadugi_adapter_source_ids(&root);
    let web_capable_ids = desktop_ids
        .intersection(&lookingglass_evidenced_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let non_web_capable_ids = desktop_ids
        .difference(&lookingglass_evidenced_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let without_generated_adapters = desktop_ids
        .difference(&generated_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let extra_generated = generated_ids
        .difference(&desktop_ids)
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        53,
        web_capable_ids.len(),
        "expected 53 web-capable desktop scenarios, found {:?}",
        web_capable_ids
    );
    assert!(
        without_generated_adapters.is_empty() && extra_generated.is_empty(),
        "desktop scenarios with generated Gadugi adapters: {:?}\ndesktop scenarios without generated Gadugi adapters: {:?}\nextra generated Gadugi adapters: {:?}",
        generated_ids,
        without_generated_adapters,
        extra_generated
    );

    let missing_core = CORE_CURRICULUM_SCENARIOS
        .iter()
        .filter(|id| !web_capable_ids.contains(**id))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing_core.is_empty(),
        "core curriculum scenarios must have LookingGlass evidence; missing {:?}\nweb-capable scenarios: {:?}\nnon-web-capable scenarios: {:?}",
        missing_core,
        web_capable_ids,
        non_web_capable_ids
    );
}

#[test]
fn lookingglass_targets_do_not_include_unsupported_matrix_rows() {
    let root = repository_root();
    let lookingglass_targeted_ids = source_lookingglass_targeted_desktop_ids(&root);
    let not_supported_ids = matrix_lookingglass_not_supported_ids(&root);
    let targeted_but_not_supported = lookingglass_targeted_ids
        .intersection(&not_supported_ids)
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        targeted_but_not_supported.is_empty(),
        "adapter.targets must not include lookingglass for unsupported matrix rows: {:?}",
        targeted_but_not_supported
    );
}

#[test]
fn round_86_targeted_curriculum_topics_are_web_capable() {
    let root = repository_root();
    let lookingglass_evidenced_ids = lookingglass_evidenced_desktop_ids(&root);
    let generated_ids = generated_gadugi_adapter_source_ids(&root);
    let missing_lookingglass_target = ROUND_86_TARGETED_WEB_SCENARIOS
        .iter()
        .filter(|id| !lookingglass_evidenced_ids.contains(**id))
        .copied()
        .collect::<Vec<_>>();
    let missing_generated = ROUND_86_TARGETED_WEB_SCENARIOS
        .iter()
        .filter(|id| !generated_ids.contains(**id))
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing_lookingglass_target.is_empty() && missing_generated.is_empty(),
        "round 86 targeted topics must be both LookingGlass-evidenced and generated; missing LookingGlass-evidenced={:?}, missing generated={:?}",
        missing_lookingglass_target,
        missing_generated
    );
}

#[test]
fn every_web_scenario_has_health_launch_action_and_verification_structure() {
    let root = repository_root();
    let desktop = desktop_scenarios(&root);
    let web_ids = lookingglass_evidenced_desktop_ids(&root);
    let mut failures = Vec::new();

    for (id, yaml) in desktop.into_iter().filter(|(id, _)| web_ids.contains(id)) {
        let steps = value_at(&yaml, &["steps"])
            .and_then(Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        let acceptance = value_at(&yaml, &["acceptance_criteria"])
            .and_then(Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        let artifacts = map_keys_at(&yaml, &["artifacts"]);
        let launcher = string_at(&yaml, &["launcher", "command"]).unwrap_or_default();
        let kind = string_at(&yaml, &["kind"]).unwrap_or_default();

        let has_validate = steps.iter().any(|step| {
            string_at(step, &["id"]).as_deref() == Some("validate-assets")
                || string_at(step, &["command"])
                    .as_deref()
                    .is_some_and(|command| command.contains("assets validate"))
        });
        let has_dependency_check = steps.iter().any(|step| {
            string_at(step, &["id"]).as_deref() == Some("check-dependencies")
                || string_at(step, &["command"])
                    .as_deref()
                    .is_some_and(|command| command.contains("deps check"))
        });
        let has_discovery = steps.iter().any(|step| {
            string_at(step, &["id"]).as_deref().is_some_and(|step_id| {
                step_id.starts_with("discover") || step_id.contains("preflight")
            }) || string_at(step, &["command"])
                .as_deref()
                .is_some_and(|command| command.contains("alice discover"))
        });
        let has_launch = is_launch_entrypoint(&launcher)
            || steps.iter().any(|step| {
                string_at(step, &["command"])
                    .as_deref()
                    .is_some_and(is_launch_entrypoint)
            })
            || (kind == "instructor_agentic_flow"
                && steps.iter().any(|step| {
                    string_at(step, &["command"])
                        .as_deref()
                        .is_some_and(|command| {
                            command.contains("agentic instructor acceptance review")
                        })
                }));
        let has_action = steps.iter().any(|step| {
            let step_id = string_at(step, &["id"]).unwrap_or_default();
            let command = string_at(step, &["command"]).unwrap_or_default();
            !command.trim().is_empty()
                && step_id != "validate-assets"
                && step_id != "check-dependencies"
                && !step_id.starts_with("discover")
        });
        let has_verification = (artifacts.contains("manifest")
            || artifacts.contains("screenshot")
            || artifacts.contains("log")
            || !acceptance.is_empty()
            || value_at(&yaml, &["rubric"])
                .and_then(Value::as_sequence)
                .is_some_and(|rubric| !rubric.is_empty()))
            && steps.iter().any(|step| {
                value_at(step, &["evidence"])
                    .and_then(Value::as_sequence)
                    .is_some_and(|evidence| !evidence.is_empty())
            });
        let has_health = has_validate || has_dependency_check || has_discovery;

        if !(has_health && has_launch && has_action && has_verification) {
            failures.push(format!(
                "{id}: health={has_health} launch={has_launch} action={has_action} verification={has_verification}"
            ));
        }

        fn is_launch_entrypoint(command: &str) -> bool {
            command.contains("alice launch-smoke")
                || command.contains("alice run-howto")
                || command.contains("alice objects-first-full-path")
                || command.contains("alice run-objects-first-world")
        }
    }

    assert!(
        failures.is_empty(),
        "every web scenario must cover health -> launch -> action -> verification:\n{}",
        failures.join("\n")
    );
}
