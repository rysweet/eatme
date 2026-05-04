use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn prepare_run_dir(run_dir: &Path) -> Result<()> {
    if run_dir.exists() {
        archive_existing_run_dir(run_dir)?;
    }
    fs::create_dir_all(run_dir.join("screenshots"))
        .with_context(|| format!("creating {}", run_dir.display()))?;
    fs::create_dir_all(run_dir.join("home"))?;
    fs::create_dir_all(run_dir.join("prefs"))?;
    fs::create_dir_all(run_dir.join("tmp"))?;
    Ok(())
}

fn archive_existing_run_dir(run_dir: &Path) -> Result<()> {
    let parent = run_dir
        .parent()
        .with_context(|| format!("{} has no parent directory", run_dir.display()))?;
    let name = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("{} has no valid directory name", run_dir.display()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_nanos();

    for attempt in 0..1000 {
        let archive_name = format!("{name}.previous-{stamp}-{attempt}");
        let archive_path = parent.join(archive_name);
        if archive_path.exists() {
            continue;
        }
        fs::rename(run_dir, &archive_path).with_context(|| {
            format!(
                "archiving existing launch evidence {} to {}",
                run_dir.display(),
                archive_path.display()
            )
        })?;
        return Ok(());
    }

    bail!(
        "could not find a unique archive path for existing launch evidence {}",
        run_dir.display()
    );
}
