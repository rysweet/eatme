use crate::pr199_recovery::qa::QaCommandProof;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Pr199RecoveryEvidence {
    pub pr: u32,
    pub workflow_proof: Option<String>,
    pub alice_actions: Vec<AliceActionEvidence>,
    pub qa_commands: Vec<QaCommandProof>,
    pub pr_metadata: Option<Value>,
}

impl Pr199RecoveryEvidence {
    pub fn for_pr199() -> Self {
        Self {
            pr: 199,
            workflow_proof: None,
            alice_actions: Vec::new(),
            qa_commands: Vec::new(),
            pr_metadata: None,
        }
    }

    pub fn with_pr(mut self, pr: u32) -> Self {
        self.pr = pr;
        self
    }

    pub fn with_workflow_proof(mut self, proof: impl Into<String>) -> Self {
        self.workflow_proof = Some(proof.into());
        self
    }

    pub fn without_workflow_proof(mut self) -> Self {
        self.workflow_proof = None;
        self
    }

    pub fn with_alice_action(mut self, action: AliceActionEvidence) -> Self {
        self.alice_actions.push(action);
        self
    }

    pub fn with_qa_command(mut self, command: QaCommandProof) -> Self {
        self.qa_commands.push(command);
        self
    }

    pub fn without_qa_command(mut self, command: &str) -> Self {
        self.qa_commands.retain(|proof| proof.command != command);
        self
    }

    pub fn with_pr_metadata(mut self, metadata: Value) -> Self {
        self.pr_metadata = Some(metadata);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliceActionEvidence {
    pub action: String,
    pub target: AliceEvidenceTarget,
    pub kind: AliceEvidenceKind,
    pub source: Option<String>,
}

impl AliceActionEvidence {
    pub fn real_original(action: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: AliceEvidenceTarget::Original,
            kind: AliceEvidenceKind::Real,
            source: Some(source.into()),
        }
    }

    pub fn missing_original(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: AliceEvidenceTarget::Original,
            kind: AliceEvidenceKind::Missing,
            source: None,
        }
    }

    pub fn synthetic_original(action: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: AliceEvidenceTarget::Original,
            kind: AliceEvidenceKind::Synthetic,
            source: Some(source.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliceEvidenceTarget {
    Original,
}

impl AliceEvidenceTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Original => "original",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliceEvidenceKind {
    Real,
    Missing,
    Synthetic,
}
