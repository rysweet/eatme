use super::{PersonaReferenceIndex, require_list, require_nonempty, validate_id};
use crate::report::AssetValidationReport;
use crate::schema::{CrewAsset, Persona, Scenario};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn validate_persona_crew(path: &Path) -> Result<AssetValidationReport> {
    let crew = parse_crew(path)?;
    Ok(validate_crew(path, &crew))
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

fn validate_crew(path: &Path, crew: &CrewAsset) -> AssetValidationReport {
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
