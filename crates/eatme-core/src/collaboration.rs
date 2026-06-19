use crate::ast::{Procedure, Statement};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeComment {
    pub author: String,
    pub block_id: String,
    pub text: String,
    pub revision: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditSession {
    pub user_id: String,
    pub procedure_name: String,
    pub base_revision: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationTarget {
    pub procedure_name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ProjectSnapshot {
    revision: usize,
    procedures: BTreeMap<String, Procedure>,
    comments: BTreeMap<String, Vec<CodeComment>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollaborativeProject {
    owner: String,
    collaborators: BTreeSet<String>,
    current: ProjectSnapshot,
    history: Vec<ProjectSnapshot>,
}

impl CollaborativeProject {
    pub fn new(owner: &str, procedures: Vec<Procedure>) -> Self {
        let snapshot = ProjectSnapshot {
            revision: 0,
            procedures: procedures
                .into_iter()
                .map(|procedure| (procedure.name.clone(), procedure))
                .collect(),
            comments: BTreeMap::new(),
        };
        Self {
            owner: owner.into(),
            collaborators: BTreeSet::new(),
            current: snapshot.clone(),
            history: vec![snapshot],
        }
    }

    pub fn current_revision(&self) -> usize {
        self.current.revision
    }

    pub fn share_with(&mut self, grantor: &str, user_id: &str) -> bool {
        if grantor != self.owner {
            return false;
        }
        self.collaborators.insert(user_id.into())
    }

    pub fn can_access(&self, user_id: &str) -> bool {
        user_id == self.owner || self.collaborators.contains(user_id)
    }

    pub fn begin_edit(&self, user_id: &str, procedure_name: &str) -> Option<EditSession> {
        if !self.can_access(user_id) || !self.current.procedures.contains_key(procedure_name) {
            return None;
        }
        Some(EditSession {
            user_id: user_id.into(),
            procedure_name: procedure_name.into(),
            base_revision: self.current.revision,
        })
    }

    pub fn apply_edit(
        &mut self,
        session: &EditSession,
        body: Vec<Statement>,
    ) -> Result<usize, String> {
        if !self.can_access(&session.user_id) {
            return Err(format!("{} does not have project access", session.user_id));
        }
        let current_procedure = self
            .current
            .procedures
            .get(&session.procedure_name)
            .cloned()
            .ok_or_else(|| format!("unknown procedure {}", session.procedure_name))?;
        let merged_body = if session.base_revision < self.current.revision {
            merge_statement_sequences(&current_procedure.body, &body)
        } else {
            body
        };
        self.current.procedures.insert(
            session.procedure_name.clone(),
            Procedure {
                name: current_procedure.name,
                parameters: current_procedure.parameters,
                body: merged_body,
            },
        );
        Ok(self.commit())
    }

    pub fn add_comment(
        &mut self,
        user_id: &str,
        block_id: &str,
        text: &str,
    ) -> Result<usize, String> {
        if !self.can_access(user_id) {
            return Err(format!("{} does not have project access", user_id));
        }
        let revision = self.current.revision;
        self.current
            .comments
            .entry(block_id.into())
            .or_default()
            .push(CodeComment {
                author: user_id.into(),
                block_id: block_id.into(),
                text: text.into(),
                revision,
            });
        Ok(self.commit())
    }

    pub fn comments_for(&self, block_id: &str) -> Vec<CodeComment> {
        self.current
            .comments
            .get(block_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn procedure_body(&self, procedure_name: &str) -> Option<Vec<Statement>> {
        self.current
            .procedures
            .get(procedure_name)
            .map(|procedure| procedure.body.clone())
    }

    pub fn revision_body(&self, revision: usize, procedure_name: &str) -> Option<Vec<Statement>> {
        self.history
            .iter()
            .find(|snapshot| snapshot.revision == revision)
            .and_then(|snapshot| snapshot.procedures.get(procedure_name))
            .map(|procedure| procedure.body.clone())
    }

    pub fn restore_revision(&mut self, revision: usize) -> bool {
        let Some(snapshot) = self
            .history
            .iter()
            .find(|entry| entry.revision == revision)
            .cloned()
        else {
            return false;
        };
        self.current = snapshot;
        self.commit();
        true
    }

    pub fn revision_count(&self) -> usize {
        self.history.len()
    }

    pub fn navigate_method_call(
        &self,
        procedure_name: &str,
        statement_index: usize,
    ) -> Option<NavigationTarget> {
        let procedure = self.current.procedures.get(procedure_name)?;
        let statement = procedure.body.get(statement_index)?;
        match statement {
            Statement::MethodCall { method, .. } => self
                .current
                .procedures
                .contains_key(method)
                .then(|| NavigationTarget {
                    procedure_name: method.clone(),
                }),
            _ => None,
        }
    }

    fn commit(&mut self) -> usize {
        let next_revision = self
            .history
            .last()
            .map(|snapshot| snapshot.revision + 1)
            .unwrap_or(0);
        self.current.revision = next_revision;
        self.history.push(self.current.clone());
        next_revision
    }
}

fn merge_statement_sequences(current: &[Statement], incoming: &[Statement]) -> Vec<Statement> {
    let mut merged = current.to_vec();
    for statement in incoming {
        if !merged.contains(statement) {
            merged.push(statement.clone());
        }
    }
    merged
}

#[cfg(test)]
#[path = "collaboration_tests.rs"]
mod tests;
