use super::{PersonaReferenceIndex, persona_reference_index};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub(crate) struct PersonaDiscovery {
    pub(crate) index: Option<PersonaReferenceIndex>,
    pub(crate) diagnostics: Vec<String>,
}

pub(crate) fn discover_scenario_personas(path: &Path) -> Result<PersonaDiscovery> {
    let mut accumulator = PersonaAccumulator::default();

    for persona_dir in persona_dirs_for(path) {
        let mut crew_paths = yaml_paths_under(&persona_dir)?;
        crew_paths.sort();
        for crew_path in crew_paths {
            let index = persona_reference_index(&crew_path)?;
            accumulator.merge(&crew_path, index);
        }
    }

    Ok(accumulator.finish())
}

fn persona_dirs_for(path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = path.parent();

    while let Some(parent) = current {
        let persona_dir = parent.join("personas");
        if seen.insert(persona_dir.clone()) {
            dirs.push(persona_dir);
        }
        current = parent.parent();
    }

    dirs
}

fn yaml_paths_under(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_yaml_paths(root, &mut paths)?;
    Ok(paths)
}

fn collect_yaml_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_yaml_paths(&path, paths)?;
        } else if is_yaml_path(&path) {
            paths.push(path);
        }
    }

    Ok(())
}

fn is_yaml_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| matches!(extension, "yaml" | "yml"))
        .unwrap_or(false)
}

#[derive(Default)]
struct PersonaAccumulator {
    index: PersonaReferenceIndex,
    instructor_sources: BTreeMap<String, PathBuf>,
    student_sources: BTreeMap<String, PathBuf>,
    diagnostics: Vec<String>,
}

impl PersonaAccumulator {
    fn merge(&mut self, path: &Path, index: PersonaReferenceIndex) {
        for id in index.instructors {
            self.add_id(path, id, "instructor");
        }
        for id in index.students {
            self.add_id(path, id, "student");
        }
    }

    fn add_id(&mut self, path: &Path, id: String, role: &str) {
        match role {
            "instructor" => {
                if let Some(student_path) = self.student_sources.get(&id) {
                    self.diagnostics
                        .push(role_conflict(&id, student_path, path));
                }
                self.instructor_sources
                    .entry(id.clone())
                    .or_insert(path.into());
                self.index.instructors.insert(id.clone());
                self.index.all.insert(id);
            }
            "student" => {
                if let Some(instructor_path) = self.instructor_sources.get(&id) {
                    self.diagnostics
                        .push(role_conflict(&id, instructor_path, path));
                }
                self.student_sources
                    .entry(id.clone())
                    .or_insert(path.into());
                self.index.students.insert(id.clone());
                self.index.all.insert(id);
            }
            _ => unreachable!("persona discovery only merges instructor and student roles"),
        }
    }

    fn finish(self) -> PersonaDiscovery {
        let has_personas = !self.index.all.is_empty();
        PersonaDiscovery {
            index: has_personas.then_some(self.index),
            diagnostics: self.diagnostics,
        }
    }
}

fn role_conflict(id: &str, first_path: &Path, second_path: &Path) -> String {
    format!(
        "persona id {id} appears as both instructor and student across persona crew files ({} and {})",
        first_path.display(),
        second_path.display()
    )
}
