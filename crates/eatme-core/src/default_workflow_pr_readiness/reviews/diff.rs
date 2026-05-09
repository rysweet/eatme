use super::super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
enum FocusedFileKind {
    CanonicalAsset,
    GeneratedAsset { fresh: bool },
    Test,
    Documentation,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusedFile {
    path: String,
    kind: FocusedFileKind,
}

impl FocusedFile {
    pub fn canonical_asset(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FocusedFileKind::CanonicalAsset,
        }
    }

    pub fn generated_asset(path: impl Into<String>, fresh: bool) -> Self {
        Self {
            path: path.into(),
            kind: FocusedFileKind::GeneratedAsset { fresh },
        }
    }

    pub fn test(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FocusedFileKind::Test,
        }
    }

    pub fn documentation(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FocusedFileKind::Documentation,
        }
    }

    pub fn unknown(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: FocusedFileKind::Unknown,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffScopeReview {
    focused: bool,
}

impl DiffScopeReview {
    pub fn focused(&self) -> bool {
        self.focused
    }
}

pub struct DiffScopeReviewer;

impl DiffScopeReviewer {
    pub fn review(files: &[FocusedFile]) -> Result<DiffScopeReview, ReadinessError> {
        for file in files {
            match file.kind {
                FocusedFileKind::GeneratedAsset { fresh: false } => {
                    return Err(ReadinessError::new(
                        ReadinessErrorKind::StaleGeneratedAsset,
                        format!("generated asset '{}' is stale", file.path),
                    ));
                }
                FocusedFileKind::Unknown => {
                    return Err(ReadinessError::new(
                        ReadinessErrorKind::UnfocusedDiff,
                        format!("file '{}' is outside the focused PR scope", file.path),
                    ));
                }
                _ => {}
            }
        }

        Ok(DiffScopeReview { focused: true })
    }
}
