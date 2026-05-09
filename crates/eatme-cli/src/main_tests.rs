use super::*;
use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn baseline_launch_smoke_keeps_compatibility_without_gate() {
    let _gate = EnvOverride::remove("EATME_REAL_ALICE");

    let result = ensure_real_alice_gate("real-alice-launch-smoke");

    assert!(result.is_ok());
}

#[test]
fn lesson_launch_smoke_requires_real_alice_gate() {
    let _gate = EnvOverride::remove("EATME_REAL_ALICE");

    let result = ensure_real_alice_gate("building-a-scene-first-world");

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("EATME_REAL_ALICE=1")
    );
}

#[test]
fn next_lesson_launch_smoke_requires_real_alice_gate() {
    let _gate = EnvOverride::remove("EATME_REAL_ALICE");

    let result = ensure_real_alice_gate("code-editor-first-run");

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("EATME_REAL_ALICE=1")
    );
}

#[test]
fn hour_of_code_launch_smoke_requires_real_alice_gate() {
    let _gate = EnvOverride::remove("EATME_REAL_ALICE");

    let result = ensure_real_alice_gate("hour-of-code-studio-kickoff");

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("EATME_REAL_ALICE=1")
    );
}

#[test]
fn lesson_launch_smoke_accepts_explicit_real_alice_gate() {
    let _gate = EnvOverride::set("EATME_REAL_ALICE", "1");

    let result = ensure_real_alice_gate("building-a-scene-first-world");

    assert!(result.is_ok());
}

struct EnvOverride<'a> {
    _guard: MutexGuard<'a, ()>,
    key: &'static str,
    old_value: Option<OsString>,
}

impl<'a> EnvOverride<'a> {
    fn set(key: &'static str, value: &str) -> Self {
        let guard = ENV_LOCK.lock().unwrap();
        let old_value = env::var_os(key);
        unsafe {
            // SAFETY: environment mutation is process-global. ENV_LOCK keeps
            // these tests serial until Drop restores the original value.
            env::set_var(key, value);
        }
        Self {
            _guard: guard,
            key,
            old_value,
        }
    }

    fn remove(key: &'static str) -> Self {
        let guard = ENV_LOCK.lock().unwrap();
        let old_value = env::var_os(key);
        unsafe {
            // SAFETY: environment mutation is process-global. ENV_LOCK keeps
            // these tests serial until Drop restores the original value.
            env::remove_var(key);
        }
        Self {
            _guard: guard,
            key,
            old_value,
        }
    }
}

impl Drop for EnvOverride<'_> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: ENV_LOCK is still held until this drop completes.
            if let Some(value) = &self.old_value {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }
}
