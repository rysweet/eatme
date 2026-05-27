use super::*;
use crate::ast::Procedure;

fn method_call(method: &str, arguments: &[&str]) -> Statement {
    Statement::MethodCall {
        object: "this".into(),
        method: method.into(),
        arguments: arguments.iter().map(|arg| (*arg).into()).collect(),
    }
}

fn comment(text: &str) -> Statement {
    Statement::Comment { text: text.into() }
}

fn project() -> CollaborativeProject {
    CollaborativeProject::new(
        "alice",
        vec![
            Procedure {
                name: "main".into(),
                parameters: vec![],
                body: vec![comment("start")],
            },
            Procedure {
                name: "helper".into(),
                parameters: vec![],
                body: vec![comment("helper")],
            },
        ],
    )
}

#[test]
fn collaborative_editing_allows_two_users_to_update_same_project() {
    let mut project = project();
    assert!(project.share_with("alice", "bob"));

    let alice_session = project.begin_edit("alice", "main").unwrap();
    let bob_session = project.begin_edit("bob", "helper").unwrap();

    project
        .apply_edit(
            &alice_session,
            vec![comment("start"), method_call("hop", &["1"])],
        )
        .unwrap();
    project
        .apply_edit(
            &bob_session,
            vec![comment("helper"), method_call("wave", &[])],
        )
        .unwrap();

    assert_eq!(
        project.procedure_body("main").unwrap(),
        vec![comment("start"), method_call("hop", &["1"])],
    );
    assert_eq!(
        project.procedure_body("helper").unwrap(),
        vec![comment("helper"), method_call("wave", &[])],
    );
}

#[test]
fn conflict_resolution_merges_concurrent_procedure_edits() {
    let mut project = project();
    project.share_with("alice", "bob");

    let alice_session = project.begin_edit("alice", "main").unwrap();
    let bob_session = project.begin_edit("bob", "main").unwrap();

    project
        .apply_edit(
            &alice_session,
            vec![comment("start"), method_call("say", &["\"Alice\""])],
        )
        .unwrap();
    project
        .apply_edit(
            &bob_session,
            vec![comment("start"), method_call("think", &["\"Bob\""])],
        )
        .unwrap();

    assert_eq!(
        project.procedure_body("main").unwrap(),
        vec![
            comment("start"),
            method_call("say", &["\"Alice\""]),
            method_call("think", &["\"Bob\""]),
        ],
    );
}

#[test]
fn chat_comments_persist_with_code_blocks() {
    let mut project = project();
    project.share_with("alice", "bob");

    project
        .add_comment("bob", "main:0", "Needs a friendlier introduction")
        .unwrap();

    assert_eq!(
        project.comments_for("main:0"),
        vec![CodeComment {
            author: "bob".into(),
            block_id: "main:0".into(),
            text: "Needs a friendlier introduction".into(),
            revision: 0,
        }],
    );
}

#[test]
fn sharing_grants_access_to_other_users() {
    let mut project = project();

    assert!(!project.can_access("bob"));
    assert!(project.share_with("alice", "bob"));
    assert!(project.can_access("bob"));
    assert!(!project.share_with("bob", "charlie"));
    assert!(!project.can_access("charlie"));
}

#[test]
fn version_history_restores_each_edit_revision() {
    let mut project = project();
    let mut expected = Vec::new();
    for index in 1..=5 {
        let session = project.begin_edit("alice", "main").unwrap();
        let body = vec![
            comment("start"),
            method_call("say", &[&format!("\"step-{index}\"")]),
        ];
        let revision = project.apply_edit(&session, body.clone()).unwrap();
        expected.push((revision, body));
    }

    assert_eq!(project.revision_count(), 6);
    for (revision, body) in expected {
        let mut restored = project.clone();
        assert!(restored.restore_revision(revision));
        assert_eq!(restored.procedure_body("main").unwrap(), body);
    }
}

#[test]
fn method_navigation_finds_declarations_from_callsites() {
    let mut project = project();
    let session = project.begin_edit("alice", "main").unwrap();
    project
        .apply_edit(&session, vec![method_call("helper", &[])])
        .unwrap();

    let target = project.navigate_method_call("main", 0).unwrap();
    assert_eq!(target.procedure_name, "helper");
}
