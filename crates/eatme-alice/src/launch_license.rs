use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;

const ACCEPT_LICENSES_ENV: &str = "EATME_ACCEPT_ALICE_LICENSES_FOR_TESTS";
const PREFERENCES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE map SYSTEM "http://java.sun.com/dtd/preferences.dtd">
<map MAP_XML_VERSION="1.0">
  <entry key="isLicenseAccepted" value="true"/>
</map>
"#;
const EMPTY_PREFERENCES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE map SYSTEM "http://java.sun.com/dtd/preferences.dtd">
<map MAP_XML_VERSION="1.0"/>
"#;

pub(crate) fn seed_license_preferences_if_requested(run_dir: &Path) -> Result<Option<String>> {
    if env::var(ACCEPT_LICENSES_ENV).as_deref() != Ok("1") {
        return Ok(None);
    }

    let user_prefs_root = run_dir.join("prefs/.java/.userPrefs");
    write_preference_branch(&user_prefs_root, &["org", "lgna", "project"])?;
    write_preference_branch(
        &user_prefs_root,
        &["edu", "cmu", "cs", "dennisc", "nebulous"],
    )?;

    Ok(Some(format!(
        "seeded Alice license preferences because {ACCEPT_LICENSES_ENV}=1"
    )))
}

fn write_preference_branch(root: &Path, branch: &[&str]) -> Result<()> {
    let mut current = root.to_path_buf();
    for (index, segment) in branch.iter().enumerate() {
        current.push(segment);
        fs::create_dir_all(&current)
            .with_context(|| format!("creating preference directory {}", current.display()))?;
        let xml = if index + 1 == branch.len() {
            PREFERENCES_XML
        } else {
            EMPTY_PREFERENCES_XML
        };
        fs::write(current.join("prefs.xml"), xml)
            .with_context(|| format!("writing preference file {}", current.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn does_not_seed_license_preferences_without_explicit_opt_in() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = EnvOverride::remove(ACCEPT_LICENSES_ENV);
        let root = unique_test_dir("license-no-seed");

        let result = seed_license_preferences_if_requested(&root).unwrap();

        assert!(result.is_none());
        assert!(!root.join("prefs/.java/.userPrefs").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seeds_alice_and_sims_license_preferences_with_explicit_opt_in() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = EnvOverride::set(ACCEPT_LICENSES_ENV, "1");
        let root = unique_test_dir("license-seed");

        let result = seed_license_preferences_if_requested(&root).unwrap();

        assert!(result.unwrap().contains(ACCEPT_LICENSES_ENV));
        assert!(
            fs::read_to_string(root.join("prefs/.java/.userPrefs/org/lgna/project/prefs.xml"))
                .unwrap()
                .contains(r#"key="isLicenseAccepted" value="true""#)
        );
        assert!(
            fs::read_to_string(
                root.join("prefs/.java/.userPrefs/edu/cmu/cs/dennisc/nebulous/prefs.xml")
            )
            .unwrap()
            .contains(r#"key="isLicenseAccepted" value="true""#)
        );
        let _ = fs::remove_dir_all(root);
    }

    struct EnvOverride {
        key: &'static str,
        old_value: Option<std::ffi::OsString>,
    }

    impl EnvOverride {
        fn set(key: &'static str, value: &str) -> Self {
            let old_value = env::var_os(key);
            unsafe {
                env::set_var(key, value);
            }
            Self { key, old_value }
        }

        fn remove(key: &'static str) -> Self {
            let old_value = env::var_os(key);
            unsafe {
                env::remove_var(key);
            }
            Self { key, old_value }
        }
    }

    impl Drop for EnvOverride {
        fn drop(&mut self) {
            unsafe {
                match &self.old_value {
                    Some(value) => env::set_var(self.key, value),
                    None => env::remove_var(self.key),
                }
            }
        }
    }

    fn unique_test_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{prefix}-{nonce}"))
    }
}
