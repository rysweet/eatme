use super::super::error::{ReadinessError, ReadinessErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditFix {
    NoRepositoryChangeNeeded,
    RepositoryChangeRequired,
    RemainingBlocker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAuditCycle {
    pub seek: String,
    pub validate: String,
    pub fix: AuditFix,
    pub clean: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAuditReport {
    cycle_count: usize,
    final_cycle_clean: bool,
}

impl QualityAuditReport {
    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    pub fn final_cycle_clean(&self) -> bool {
        self.final_cycle_clean
    }
}

pub struct QualityAuditCycleRunner;

impl QualityAuditCycleRunner {
    pub fn review(cycles: &[QualityAuditCycle]) -> Result<QualityAuditReport, ReadinessError> {
        if cycles.len() < 3 || cycles.iter().any(missing_seek_or_validate) {
            return Err(ReadinessError::new(
                ReadinessErrorKind::MissingQualityAuditCycle,
                "at least three SEEK / VALIDATE / FIX quality-audit cycles are required",
            ));
        }

        let final_cycle = cycles.last().expect("cycles length checked above");
        if !final_cycle.clean || final_cycle.fix == AuditFix::RemainingBlocker {
            return Err(ReadinessError::new(
                ReadinessErrorKind::UncleanFinalAuditCycle,
                "the final quality-audit cycle is not clean",
            ));
        }

        Ok(QualityAuditReport {
            cycle_count: cycles.len(),
            final_cycle_clean: true,
        })
    }
}

fn missing_seek_or_validate(cycle: &QualityAuditCycle) -> bool {
    cycle.seek.trim().is_empty() || cycle.validate.trim().is_empty()
}
