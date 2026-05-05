use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct RunFirstLessonReadinessArgs {
    #[arg(long, default_value = "assets/alice-comparison-targets.yaml")]
    pub registry: PathBuf,
    #[arg(long, default_value = "baseline")]
    pub baseline_target: String,
    #[arg(long, default_value = "modernized")]
    pub modernized_target: String,
    #[arg(long)]
    pub baseline_home: Option<PathBuf>,
    #[arg(long)]
    pub modernized_home: Option<PathBuf>,
    #[arg(long)]
    pub run_id: String,
    #[arg(long, default_value = "runs")]
    pub runs_dir: PathBuf,
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub no_memory: bool,
    #[arg(long)]
    pub offline_package: bool,
    #[arg(long)]
    pub starter_project: Option<PathBuf>,
    #[arg(long)]
    pub execute: bool,
}
