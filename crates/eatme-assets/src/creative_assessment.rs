use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CreativeAssessmentReport {
    pub lesson: String,
    pub machine_assessable: Vec<AssessmentAspect>,
    pub human_review_needed: Vec<AssessmentAspect>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssessmentAspect {
    pub name: String,
    pub category: AssessmentCategory,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum AssessmentCategory {
    #[serde(rename = "file-structure")]
    FileStructure,
    #[serde(rename = "runtime-behavior")]
    RuntimeBehavior,
    #[serde(rename = "creative-expression")]
    CreativeExpression,
    #[serde(rename = "learning-evidence")]
    LearningEvidence,
}

pub fn for_building_a_scene() -> CreativeAssessmentReport {
    CreativeAssessmentReport {
        lesson: "building-a-scene-first-world".into(),
        machine_assessable: vec![
            AssessmentAspect {
                name: "scene-file-exists".into(),
                category: AssessmentCategory::FileStructure,
                rationale: "Verify the saved scene file is present on disk".into(),
            },
            AssessmentAspect {
                name: "scene-file-valid-format".into(),
                category: AssessmentCategory::FileStructure,
                rationale: "Parse the scene file to confirm it matches the expected schema".into(),
            },
            AssessmentAspect {
                name: "object-count-nonzero".into(),
                category: AssessmentCategory::FileStructure,
                rationale: "Check that at least one 3D object was placed in the scene".into(),
            },
            AssessmentAspect {
                name: "code-file-modified".into(),
                category: AssessmentCategory::RuntimeBehavior,
                rationale: "Detect whether the student edited code files via diff or timestamp"
                    .into(),
            },
            AssessmentAspect {
                name: "world-ran-without-errors".into(),
                category: AssessmentCategory::RuntimeBehavior,
                rationale: "Check process exit code and stderr for runtime errors".into(),
            },
            AssessmentAspect {
                name: "world-ran-minimum-duration".into(),
                category: AssessmentCategory::RuntimeBehavior,
                rationale: "Verify the world ran for at least a few seconds, indicating engagement"
                    .into(),
            },
        ],
        human_review_needed: vec![
            AssessmentAspect {
                name: "object-placement-intentional".into(),
                category: AssessmentCategory::CreativeExpression,
                rationale: "Assess whether object placement shows deliberate spatial reasoning"
                    .into(),
            },
            AssessmentAspect {
                name: "scene-aesthetic-coherence".into(),
                category: AssessmentCategory::CreativeExpression,
                rationale: "Evaluate whether the scene has a coherent visual theme or narrative"
                    .into(),
            },
            AssessmentAspect {
                name: "code-change-purposeful".into(),
                category: AssessmentCategory::CreativeExpression,
                rationale: "Judge whether code edits reflect intentional creative goals".into(),
            },
            AssessmentAspect {
                name: "student-can-explain-choices".into(),
                category: AssessmentCategory::LearningEvidence,
                rationale: "Ask the student to articulate why they made specific decisions".into(),
            },
            AssessmentAspect {
                name: "student-iterated-on-design".into(),
                category: AssessmentCategory::LearningEvidence,
                rationale: "Look for evidence of revision cycles indicating reflective practice"
                    .into(),
            },
            AssessmentAspect {
                name: "student-connected-to-prior-knowledge".into(),
                category: AssessmentCategory::LearningEvidence,
                rationale: "Assess whether the student related the activity to prior learning"
                    .into(),
            },
        ],
    }
}
