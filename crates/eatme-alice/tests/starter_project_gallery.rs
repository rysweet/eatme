#![allow(unexpected_cfgs)]
//! Starter-project gallery integration tests.
//!
//! Discovers **all** `.a3p` starter projects in the Alice installation and
//! validates each one:
//!
//! 1. Parse succeeds (`parse_a3p_program` returns `Some`)
//! 2. `programType.xml` inside the ZIP is present and non-empty
//! 3. At least one `Procedure` with a non-empty body (entity presence proxy)
//! 4. Grading pipelines run without panicking
//!
//! **Gated behind `EATME_REAL_ALICE=1`** — requires an actual Alice installation
//! with starter projects on disk.

use std::io::Read;
use std::path::PathBuf;

use eatme_assets::grading_report::{LoopsGradingInput, grade_loops_and_conditionals};
use eatme_assets::{EventsGradingInput, grade_events_and_collision};
use eatme_core::ast::Program;

#[allow(dead_code)]
mod launch_smoke_support;
use launch_smoke_support::{alice_home, real_alice_enabled};

#[allow(dead_code)]
mod a3p_parser_support;
use a3p_parser_support::parse_a3p_program;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Directory containing Alice starter projects.
fn starter_projects_dir() -> PathBuf {
    alice_home().join("starter-projects")
}

/// Discover all `.a3p` files in the starter-projects directory.
fn discover_a3p_files() -> Vec<PathBuf> {
    let dir = starter_projects_dir();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read starter-projects dir {}: {e}", dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("a3p") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

/// Read the `programType.xml` entry from an `.a3p` ZIP archive.
fn extract_program_type_xml(path: &std::path::Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    // Try common entry names — Alice uses both root-level and nested paths.
    for name in ["programType.xml", "type.xml"] {
        if let Ok(mut entry) = archive.by_name(name) {
            let mut content = String::new();
            if entry.read_to_string(&mut content).is_ok() && !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    // Fallback: scan all entries for one whose name ends with "programType.xml"
    for i in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(i) {
            let entry_name = entry.name().to_string();
            if entry_name.ends_with("programType.xml") || entry_name.ends_with("type.xml") {
                let mut content = String::new();
                if entry.read_to_string(&mut content).is_ok() && !content.trim().is_empty() {
                    return Some(content);
                }
            }
        }
    }

    None
}

fn loops_input(program: Program) -> LoopsGradingInput {
    LoopsGradingInput {
        assets_valid: true,
        asset_reason: "Gallery test — real .a3p parsed".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

fn events_input(program: Program) -> EventsGradingInput {
    EventsGradingInput {
        assets_valid: true,
        asset_reason: "Gallery test — real .a3p parsed".into(),
        deps_available: true,
        deps_reason: "Alice installation verified".into(),
        student_program: Some(program),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn gallery_discovers_starter_projects() {
    if !real_alice_enabled() {
        eprintln!("skipping starter-project gallery tests (set EATME_REAL_ALICE=1 to enable)");
        return;
    }

    let files = discover_a3p_files();
    eprintln!(
        "gallery: discovered {} .a3p starter projects in {}",
        files.len(),
        starter_projects_dir().display()
    );
    for f in &files {
        eprintln!(
            "  • {}",
            f.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    assert!(
        !files.is_empty(),
        "expected >0 .a3p files in {}",
        starter_projects_dir().display()
    );
}

#[test]
fn gallery_all_projects_parse_successfully() {
    if !real_alice_enabled() {
        eprintln!("skipping (set EATME_REAL_ALICE=1)");
        return;
    }

    let files = discover_a3p_files();
    assert!(!files.is_empty(), "no .a3p files discovered");

    for path in &files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        eprintln!("parsing: {name}");

        let program = parse_a3p_program(path);
        assert!(
            program.is_some(),
            "parse_a3p_program returned None for {name} ({})",
            path.display()
        );
    }
    eprintln!("gallery: all {} projects parsed successfully", files.len());
}

#[test]
fn gallery_all_projects_have_program_type_xml() {
    if !real_alice_enabled() {
        eprintln!("skipping (set EATME_REAL_ALICE=1)");
        return;
    }

    let files = discover_a3p_files();
    assert!(!files.is_empty(), "no .a3p files discovered");

    for path in &files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        eprintln!("checking programType.xml: {name}");

        let xml = extract_program_type_xml(path);
        assert!(
            xml.is_some(),
            "programType.xml missing or empty in {name} ({})",
            path.display()
        );
        let content = xml.unwrap();
        assert!(
            !content.trim().is_empty(),
            "programType.xml is empty in {name}"
        );
        eprintln!("  programType.xml: {} bytes", content.len());
    }
}

#[test]
fn gallery_all_projects_have_entities() {
    if !real_alice_enabled() {
        eprintln!("skipping (set EATME_REAL_ALICE=1)");
        return;
    }

    let files = discover_a3p_files();
    assert!(!files.is_empty(), "no .a3p files discovered");

    for path in &files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let program = parse_a3p_program(path).unwrap_or_else(|| panic!("failed to parse {name}"));

        assert!(
            !program.procedures.is_empty(),
            "{name}: expected at least one Procedure (entity presence proxy)"
        );

        let has_body = program.procedures.iter().any(|p| !p.body.is_empty());
        assert!(
            has_body,
            "{name}: expected at least one Procedure with a non-empty body \
             (entities should produce statements)"
        );

        let total_stmts: usize = program.procedures.iter().map(|p| p.body.len()).sum();
        eprintln!(
            "  {name}: {} procedures, {total_stmts} statements",
            program.procedures.len()
        );
    }
}

#[test]
fn gallery_grading_pipelines_no_crash() {
    if !real_alice_enabled() {
        eprintln!("skipping (set EATME_REAL_ALICE=1)");
        return;
    }

    let files = discover_a3p_files();
    assert!(!files.is_empty(), "no .a3p files discovered");

    for path in &files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let program = parse_a3p_program(path).unwrap_or_else(|| panic!("failed to parse {name}"));

        // Run loops & conditionals pipeline
        eprintln!("grading (loops): {name}");
        let loops_report = grade_loops_and_conditionals(loops_input(program.clone()));
        eprintln!(
            "  loops result: passed={}, steps={}",
            loops_report.passed,
            loops_report.steps.len()
        );

        // Run events & collision pipeline
        eprintln!("grading (events): {name}");
        let events_report = grade_events_and_collision(events_input(program));
        eprintln!(
            "  events result: passed={}, steps={}",
            events_report.passed,
            events_report.steps.len()
        );
    }
    eprintln!(
        "gallery: all {} projects passed grading pipelines without crash",
        files.len()
    );
}
