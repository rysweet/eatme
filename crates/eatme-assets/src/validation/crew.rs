use super::{PersonaReferenceIndex, require_list, require_nonempty, validate_id};
use crate::report::AssetValidationReport;
use crate::schema::{ConstituencyCoverage, CrewAsset, Persona, PromptCard, Scenario};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn validate_persona_crew(path: &Path) -> Result<AssetValidationReport> {
    validate_persona_crew_against_scenario_assets(path, None)
}

pub(crate) fn validate_persona_crew_against_scenario_assets(
    path: &Path,
    scenario_asset_ids: Option<&BTreeSet<String>>,
) -> Result<AssetValidationReport> {
    let crew = parse_crew(path)?;
    Ok(validate_crew(path, &crew, scenario_asset_ids))
}

pub(crate) fn persona_reference_index(path: &Path) -> Result<PersonaReferenceIndex> {
    let crew = parse_crew(path)?;
    let instructors = crew
        .personas
        .instructors
        .iter()
        .map(|persona| persona.id.clone())
        .collect::<BTreeSet<_>>();
    let students = crew
        .personas
        .students
        .iter()
        .map(|persona| persona.id.clone())
        .collect::<BTreeSet<_>>();
    let all = instructors
        .iter()
        .chain(students.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    Ok(PersonaReferenceIndex {
        instructors,
        students,
        all,
    })
}

fn parse_crew(path: &Path) -> Result<CrewAsset> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading persona crew asset {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("parsing persona crew YAML {}", path.display()))
}

fn validate_crew(
    path: &Path,
    crew: &CrewAsset,
    scenario_asset_ids: Option<&BTreeSet<String>>,
) -> AssetValidationReport {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    require_nonempty(&crew.workstream, "workstream", &mut errors);
    require_nonempty(&crew.title, "title", &mut errors);
    require_nonempty(&crew.purpose, "purpose", &mut errors);

    let mut instructor_ids = BTreeSet::new();
    let mut student_ids = BTreeSet::new();
    validate_personas(
        &crew.personas.instructors,
        "instructor",
        &mut instructor_ids,
        &mut errors,
    );
    validate_personas(
        &crew.personas.students,
        "student",
        &mut student_ids,
        &mut errors,
    );
    let all_ids = instructor_ids
        .iter()
        .chain(student_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>();

    if crew.core_scenarios_from_existing_alice_resources.len() < 5 {
        errors
            .push("core scenarios must contain at least five Alice.org-grounded scenarios".into());
    }
    if crew.creative_new_teaching_learning_scenarios.len() < 10 {
        errors.push("creative scenarios must contain at least ten scenarios".into());
    }

    let mut scenario_ids = BTreeSet::new();
    validate_scenarios(
        &crew.core_scenarios_from_existing_alice_resources,
        "existing-alice-resource",
        &instructor_ids,
        &student_ids,
        &all_ids,
        &mut scenario_ids,
        &mut errors,
    );
    validate_scenarios(
        &crew.creative_new_teaching_learning_scenarios,
        "creative-new",
        &instructor_ids,
        &student_ids,
        &all_ids,
        &mut scenario_ids,
        &mut errors,
    );
    validate_constituency_coverage(
        &crew.constituency_coverage,
        &all_ids,
        &scenario_ids,
        &mut errors,
    );
    validate_student_outside_in_flow_assets(
        &crew.student_outside_in_flow_assets,
        &scenario_ids,
        scenario_asset_ids,
        &mut errors,
    );
    AssetValidationReport {
        schema_version: "eatme.assets/persona-crew-validation/v1".into(),
        asset_path: path.display().to_string(),
        passed: errors.is_empty(),
        instructor_count: crew.personas.instructors.len(),
        student_count: crew.personas.students.len(),
        core_scenario_count: crew.core_scenarios_from_existing_alice_resources.len(),
        creative_scenario_count: crew.creative_new_teaching_learning_scenarios.len(),
        scenario_asset_count: 0,
        errors,
        warnings,
    }
}

fn validate_constituency_coverage(
    constituency_coverage: &[ConstituencyCoverage],
    all_persona_ids: &BTreeSet<String>,
    all_scenario_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let required_constituencies = [
        "curriculum-designers",
        "it-setup-support",
        "workshop-facilitators",
        "vr-player-users",
        "media-audio-creators",
        "model-texture-import-users",
        "alice-2-migration-users",
        "teacher-community-sharing",
    ];
    let mut seen_ids = BTreeSet::new();

    for coverage in constituency_coverage {
        validate_id(&coverage.id, "constituency", errors);
        if !seen_ids.insert(coverage.id.clone()) {
            errors.push(format!("duplicate constituency id {}", coverage.id));
        }
        require_nonempty(&coverage.label, &format!("{}.label", coverage.id), errors);
        require_nonempty(
            &coverage.editable_by,
            &format!("{}.editable_by", coverage.id),
            errors,
        );
        require_list(
            &coverage.persona_ids,
            &format!("{}.persona_ids", coverage.id),
            errors,
        );
        require_list(
            &coverage.scenario_ids,
            &format!("{}.scenario_ids", coverage.id),
            errors,
        );
        require_list(
            &coverage.evidence,
            &format!("{}.evidence", coverage.id),
            errors,
        );

        for persona_id in &coverage.persona_ids {
            if !all_persona_ids.contains(persona_id) {
                errors.push(format!(
                    "constituency {} references missing persona {}",
                    coverage.id, persona_id
                ));
            }
        }
        for scenario_id in &coverage.scenario_ids {
            if !all_scenario_ids.contains(scenario_id) {
                errors.push(format!(
                    "constituency {} references missing scenario {}",
                    coverage.id, scenario_id
                ));
            }
        }
    }

    for required_id in required_constituencies {
        if !seen_ids.contains(required_id) {
            errors.push(format!("missing constituency coverage {required_id}"));
        }
    }
}

fn validate_student_outside_in_flow_assets(
    flow_assets: &crate::schema::StudentOutsideInFlowAssets,
    crew_scenario_ids: &BTreeSet<String>,
    scenario_asset_ids: Option<&BTreeSet<String>>,
    errors: &mut Vec<String>,
) {
    validate_prompt_cards(
        &flow_assets.prompt_cards,
        crew_scenario_ids,
        scenario_asset_ids,
        errors,
    );
    for (coverage_id, persona_ids) in &flow_assets.coverage_map {
        require_nonempty(
            coverage_id,
            "student_outside_in_flow_assets.coverage_map key",
            errors,
        );
        require_list(
            persona_ids,
            &format!("student_outside_in_flow_assets.coverage_map.{coverage_id}"),
            errors,
        );
    }
}

fn validate_prompt_cards(
    prompt_cards: &[PromptCard],
    crew_scenario_ids: &BTreeSet<String>,
    scenario_asset_ids: Option<&BTreeSet<String>>,
    errors: &mut Vec<String>,
) {
    for prompt_card in prompt_cards {
        validate_id(&prompt_card.id, "prompt card", errors);
        require_nonempty(
            &prompt_card.editable_by,
            &format!("prompt card {}.editable_by", prompt_card.id),
            errors,
        );
        require_nonempty(
            &prompt_card.purpose,
            &format!("prompt card {}.purpose", prompt_card.id),
            errors,
        );
        require_nonempty(
            &prompt_card.prompt_frame,
            &format!("prompt card {}.prompt_frame", prompt_card.id),
            errors,
        );
        require_list(
            &prompt_card.scenario_ids,
            &format!("prompt card {}.scenario_ids", prompt_card.id),
            errors,
        );
        require_list(
            &prompt_card.evidence,
            &format!("prompt card {}.evidence", prompt_card.id),
            errors,
        );
        for scenario_id in &prompt_card.scenario_ids {
            if !crew_scenario_ids.contains(scenario_id) {
                errors.push(format!(
                    "prompt card {} references missing crew scenario {}",
                    prompt_card.id, scenario_id
                ));
            }
            if let Some(asset_ids) = scenario_asset_ids
                && !asset_ids.contains(scenario_id)
            {
                errors.push(format!(
                    "prompt card {} references missing scenario asset {}",
                    prompt_card.id, scenario_id
                ));
            }
        }
    }
}

fn validate_personas(
    personas: &[Persona],
    expected_role: &str,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for persona in personas {
        validate_id(&persona.id, "persona", errors);
        if !ids.insert(persona.id.clone()) {
            errors.push(format!("duplicate persona id {}", persona.id));
        }
        if persona.role != expected_role {
            errors.push(format!(
                "persona {} has role {}, expected {expected_role}",
                persona.id, persona.role
            ));
        }
        require_nonempty(
            &persona.archetype,
            &format!("{}.archetype", persona.id),
            errors,
        );
        require_list(&persona.goals, &format!("{}.goals", persona.id), errors);
        require_list(
            &persona.constraints,
            &format!("{}.constraints", persona.id),
            errors,
        );
        require_list(
            &persona.educational_intent,
            &format!("{}.educational_intent", persona.id),
            errors,
        );
        require_list(
            &persona.observable_behaviors,
            &format!("{}.observable_behaviors", persona.id),
            errors,
        );
        require_list(
            &persona.anti_behaviors,
            &format!("{}.anti_behaviors", persona.id),
            errors,
        );
        require_list(
            &persona.evidence,
            &format!("{}.evidence", persona.id),
            errors,
        );
    }
}

fn validate_scenarios(
    scenarios: &[Scenario],
    expected_origin: &str,
    instructor_ids: &BTreeSet<String>,
    student_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    scenario_ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for scenario in scenarios {
        validate_id(&scenario.id, "scenario", errors);
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("duplicate scenario id {}", scenario.id));
        }
        if scenario.origin != expected_origin {
            errors.push(format!(
                "scenario {} has origin {}, expected {expected_origin}",
                scenario.id, scenario.origin
            ));
        }
        require_list(
            &scenario.coverage,
            &format!("{}.coverage", scenario.id),
            errors,
        );
        require_nonempty(
            &scenario.user_story,
            &format!("{}.user_story", scenario.id),
            errors,
        );
        if !scenario.user_story.starts_with("As a ") && !scenario.user_story.starts_with("As an ") {
            errors.push(format!(
                "scenario {} user_story must start with 'As a ' or 'As an '",
                scenario.id
            ));
        }
        require_list(
            &scenario.educational_intent.concepts,
            &format!("{}.educational_intent.concepts", scenario.id),
            errors,
        );
        require_list(
            &scenario.educational_intent.habits,
            &format!("{}.educational_intent.habits", scenario.id),
            errors,
        );
        require_list(
            &scenario.constraints,
            &format!("{}.constraints", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.instructor,
            &format!("{}.observable_behaviors.instructor", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.student,
            &format!("{}.observable_behaviors.student", scenario.id),
            errors,
        );
        require_list(
            &scenario.observable_behaviors.system_or_artifact,
            &format!("{}.observable_behaviors.system_or_artifact", scenario.id),
            errors,
        );
        require_nonempty(
            &scenario.agentic_test_prompt,
            &format!("{}.agentic_test_prompt", scenario.id),
            errors,
        );
        require_list(
            &scenario.acceptance_probes,
            &format!("{}.acceptance_probes", scenario.id),
            errors,
        );
        require_list(&scenario.avoid, &format!("{}.avoid", scenario.id), errors);
        validate_references(
            &scenario.id,
            &scenario.personas.instructors,
            instructor_ids,
            all_ids,
            "instructor",
            errors,
        );
        validate_references(
            &scenario.id,
            &scenario.personas.students,
            student_ids,
            all_ids,
            "student",
            errors,
        );
    }
}

fn validate_references(
    scenario_id: &str,
    refs: &[String],
    expected_ids: &BTreeSet<String>,
    all_ids: &BTreeSet<String>,
    role: &str,
    errors: &mut Vec<String>,
) {
    require_list(refs, &format!("{scenario_id}.personas.{role}s"), errors);
    for id in refs {
        if !expected_ids.contains(id) {
            let suffix = if all_ids.contains(id) {
                " with wrong role"
            } else {
                ""
            };
            errors.push(format!(
                "scenario {scenario_id} references missing {role} persona {id}{suffix}"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_card_refs_must_have_scenario_assets() {
        let prompt_cards = vec![PromptCard {
            id: "data-state-card".into(),
            scenario_ids: vec!["neighborhood-data-story".into()],
            ..Default::default()
        }];
        let crew_scenario_ids = BTreeSet::from(["neighborhood-data-story".into()]);
        let scenario_asset_ids = BTreeSet::from(["variables-scorekeeper-timekeeper".into()]);
        let mut errors = Vec::new();

        validate_prompt_cards(
            &prompt_cards,
            &crew_scenario_ids,
            Some(&scenario_asset_ids),
            &mut errors,
        );

        assert!(
            errors
                .iter()
                .any(|error| error.contains(
                    "prompt card data-state-card references missing scenario asset neighborhood-data-story"
                )),
            "expected prompt-card scenario asset reference error, got {errors:?}"
        );
    }
}
