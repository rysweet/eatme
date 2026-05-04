use super::*;

#[test]
fn chooses_non_default_display_format() {
    assert!(choose_display().starts_with(':'));
}

#[test]
fn rejects_non_kebab_case_scenario_names() {
    assert!(validate_scenario_name("../bad").is_err());
    assert!(validate_scenario_name("building-a-scene-first-world").is_ok());
}
