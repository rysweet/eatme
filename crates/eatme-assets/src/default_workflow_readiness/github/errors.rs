use super::{ExternalServiceError, ExternalServiceErrorKind};

pub(super) fn classify_gh_failure(stderr: &str) -> ExternalServiceError {
    let lower = stderr.to_lowercase();
    let kind = if lower.contains("rate limit") || lower.contains("secondary rate limit") {
        ExternalServiceErrorKind::RateLimited
    } else if lower.contains("timeout") || lower.contains("timed out") {
        ExternalServiceErrorKind::Timeout
    } else if lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("temporarily unavailable")
    {
        ExternalServiceErrorKind::TemporarilyUnavailable
    } else {
        ExternalServiceErrorKind::CommandFailed
    };
    ExternalServiceError::new(kind, stderr)
}
