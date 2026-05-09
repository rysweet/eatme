use crate::{CommandRunner, CommandSpec};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct Pr199Metadata {
    pub number: u32,
    pub supporting_context_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_ref_oid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_state_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_check_rollup: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
}

impl Pr199Metadata {
    pub fn from_optional_value(value: Option<Value>) -> Result<Self> {
        match value {
            Some(value) => Self::from_value(value),
            None => Ok(Self::supporting_context_only(199)),
        }
    }

    pub fn from_value(value: Value) -> Result<Self> {
        let number = value
            .get("number")
            .and_then(Value::as_u64)
            .map(|number| number as u32)
            .unwrap_or(199);

        Ok(Self {
            number,
            supporting_context_only: true,
            head_ref_oid: value
                .get("headRefOid")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            merge_state_status: value
                .get("mergeStateStatus")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            mergeable: value.get("mergeable").cloned(),
            status_check_rollup: value.get("statusCheckRollup").cloned(),
            raw: Some(value),
        })
    }

    fn supporting_context_only(number: u32) -> Self {
        Self {
            number,
            supporting_context_only: true,
            head_ref_oid: None,
            merge_state_status: None,
            mergeable: None,
            status_check_rollup: None,
            raw: None,
        }
    }
}

pub fn fetch_pr199_metadata(runner: &impl CommandRunner) -> Result<Pr199Metadata> {
    let output = runner
        .run(&CommandSpec::new("gh").args([
            "pr",
            "view",
            "199",
            "--json",
            "headRefOid,mergeStateStatus,mergeable,statusCheckRollup",
        ]))
        .context("fetching fixed PR #199 metadata with gh pr view")?;

    if output.exit_status != Some(0) {
        bail!(
            "gh pr view 199 failed with status {:?}: {}",
            output.exit_status,
            output.stderr.trim()
        );
    }

    let metadata = serde_json::from_str::<Value>(&output.stdout)
        .context("parsing gh pr view 199 metadata JSON")?;
    Pr199Metadata::from_value(metadata)
}
