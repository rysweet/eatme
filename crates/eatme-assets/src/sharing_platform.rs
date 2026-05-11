use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SharingPlatformReport {
    pub schema_version: String,
    pub lesson: String,
    pub passed: bool,
    pub entries: Vec<FeatureEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FeatureEntry {
    pub feature: String,
    pub status: FeatureReadiness,
    pub depends_on: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum FeatureReadiness {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "platform-blocked")]
    PlatformBlocked,
}

pub struct SharingPlatformInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
}

/// Evaluate sharing and deployment feature readiness.
///
/// Returns a report with 6 entries (2 preconditions + 4 features), evaluating
/// which sharing features work vs which are blocked. Only export-a3w and
/// file-sharing contribute to the `passed` flag; web-sharing and classroom-deploy
/// are always platform-blocked.
pub fn check_sharing_platform_readiness(input: SharingPlatformInput) -> SharingPlatformReport {
    let assets_status = if input.assets_valid {
        FeatureReadiness::Ready
    } else {
        FeatureReadiness::Blocked
    };
    let deps_status = if input.deps_available {
        FeatureReadiness::Ready
    } else {
        FeatureReadiness::Blocked
    };

    let (export_status, export_reason) = match (&assets_status, &deps_status) {
        (FeatureReadiness::Ready, FeatureReadiness::Ready) => (
            FeatureReadiness::Ready,
            "All preconditions met for .a3w export".into(),
        ),
        _ => {
            let mut blockers = Vec::new();
            if assets_status == FeatureReadiness::Blocked {
                blockers.push("validate-assets");
            }
            if deps_status == FeatureReadiness::Blocked {
                blockers.push("check-dependencies");
            }
            (
                FeatureReadiness::Blocked,
                format!("Blocked by {}", blockers.join(", ")),
            )
        }
    };

    let (file_sharing_status, file_sharing_reason) = if export_status == FeatureReadiness::Ready {
        (
            FeatureReadiness::Ready,
            "Ready: export-a3w available for file sharing".into(),
        )
    } else {
        (FeatureReadiness::Blocked, "Blocked by export-a3w".into())
    };

    let passed =
        export_status == FeatureReadiness::Ready && file_sharing_status == FeatureReadiness::Ready;

    SharingPlatformReport {
        schema_version: "eatme.assets/sharing-platform/v1".into(),
        lesson: "building-a-scene-first-world".into(),
        passed,
        entries: vec![
            FeatureEntry {
                feature: "validate-assets".into(),
                status: assets_status,
                depends_on: vec![],
                reason: input.asset_reason,
            },
            FeatureEntry {
                feature: "check-dependencies".into(),
                status: deps_status,
                depends_on: vec![],
                reason: input.deps_reason,
            },
            FeatureEntry {
                feature: "export-a3w".into(),
                status: export_status,
                depends_on: vec!["validate-assets".into(), "check-dependencies".into()],
                reason: export_reason,
            },
            FeatureEntry {
                feature: "file-sharing".into(),
                status: file_sharing_status,
                depends_on: vec!["export-a3w".into()],
                reason: file_sharing_reason,
            },
            FeatureEntry {
                feature: "web-sharing".into(),
                status: FeatureReadiness::PlatformBlocked,
                depends_on: vec![],
                reason: "Web sharing is not supported on this platform".into(),
            },
            FeatureEntry {
                feature: "classroom-deploy".into(),
                status: FeatureReadiness::PlatformBlocked,
                depends_on: vec![],
                reason: "Classroom deployment is not supported on this platform".into(),
            },
        ],
    }
}

#[cfg(test)]
#[path = "sharing_platform_tests.rs"]
mod tests;
