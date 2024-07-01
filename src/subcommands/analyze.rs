use std::path::Path;

use crate::{
    cargo_project::CargoProject, create_dependencies::CrateDependencies,
    feature_buffer::DependencyFeaturePermutator, report::WorkspaceCrate, utils, Report,
};
use clap::Args;

/// Analyzes the workspace for unused, but, enabled feature flags.
#[derive(Args, Debug, Clone, Default)]
#[clap(author, version)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
pub struct AnalyzeCommand {
    /// The absolute root 'directory' of the toml project or workspace.
    /// If not specified it will take the current executable directory.
    #[clap(short = 'w', long = "workspace", value_parser)]
    pub workspace: Option<String>,

    /// The absolute report 'directory' path to which the report will be written.
    /// If not specified it will be written to the current executable directory.
    #[clap(short = 'r', long = "report-dir", value_parser)]
    pub report_dir: Option<String>,

    /// Number of parallel jobs to run. Defaults to the number of CPUs.
    #[clap(short = 'j', long = "jobs", value_parser)]
    pub parallel_build_jobs: Option<i32>,

    /// Skip certain dependencies in the analysis of unused feature flags.
    #[clap(short = 's', long = "skip")]
    pub skip_dependencies: Vec<String>,
    /// The log level (debug, info, warn, error, off). Defaults to info.
    #[clap(short = 'l', long = "log-level", value_parser)]
    pub log_level: Option<String>,

    /// Build the package's library. Enabled by default.
    #[clap(long = "lib", action, default_value_t = true)]
    pub build_lib: bool,

    /// Build all binary targets. Enabled by default.
    #[clap(long = "bins", action, default_value_t = true)]
    pub build_bins: bool,

    /// What target should be used for the cargo build process.
    /// Defaults to the current target.
    #[clap(short = 't', long = "target", value_parser)]
    pub build_target: Vec<String>,
    /// Build all targets in test mode that have the test = true manifest flag set.
    #[clap(long = "tests", action)]
    pub build_tests: bool,
    /// Build all targets in benchmark mode that have the bench = true manifest flag set.
    #[clap(long = "benches", action)]
    pub build_benches: bool,
    /// Build all example targets.
    #[clap(long = "examples", action)]
    pub build_examples: bool,
}

impl AnalyzeCommand {
    pub fn execute(mut self) -> anyhow::Result<()> {
        utils::initialize_logger(self.log_level.clone());

        let current_exe = std::env::current_dir()?;
        let workspace_path = self
            .workspace
            .take()
            .unwrap_or_else(|| current_exe.display().to_string());

        log::info!("{}", workspace_path);

        let crate_path = Path::new(&workspace_path);

        match CargoProject::new(crate_path, self.clone()) {
            Ok(root_toml) => {
                let workspace_members = root_toml.workspace_members();
                if !workspace_members.is_empty() {
                    log::debug!("Workspace detected, iterating over workspace crates...");

                    let mut report = Report::new("Workspace");

                    for member_path in workspace_members {
                        log::debug!("Processing '{}' crate ...", member_path.display());

                        match CargoProject::new(&member_path, self.clone()) {
                            Ok(workspace_member) => {
                                find_unused_crate_features(workspace_member, &mut report)
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to load '{}' crate. {e}",
                                    member_path.display()
                                );
                            }
                        }
                    }
                } else {
                    let mut report = Report::new(&root_toml.crate_name());
                    find_unused_crate_features(root_toml, &mut report);
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to load '{}/Cargo.toml' crate. {e}",
                    crate_path.display()
                );
            }
        }

        Ok(())
    }
}

pub fn find_unused_crate_features(toml_crate: CargoProject, report: &mut Report) {
    if let Err(e) = find_unused_features(toml_crate, report) {
        log::error!("Error while looking for unused features. {e}");
    }
}

pub fn find_unused_features(mut toml: CargoProject, report: &mut Report) -> anyhow::Result<()> {
    let crate_dependency = toml.gather_meta_data();

    permutate_features(crate_dependency, &mut toml, report)?;

    report.flush(&toml.report_path())
}

fn permutate_features(
    crate_deps: CrateDependencies,
    toml: &mut CargoProject,
    final_report: &mut Report,
) -> anyhow::Result<()> {
    let total_features: f32 = crate_deps
        .dependency_features
        .iter()
        .map(|f| f.1.len() as f32)
        .sum();
    let total_deps = crate_deps.dependency_features.len() as f32;

    let mut workspace_report = WorkspaceCrate::new(&toml.toml_path());

    log::info!("{}", format!("|===== Crate '{}' =====|", toml.crate_name()));

    log::info!("Start pruning feature flags. The process will recompile the project {total_features} times.");

    for (i, (dependency_name, config)) in crate_deps
        .dependency_features
        .iter()
        .filter(|f| !f.1.is_empty())
        .enumerate()
    {
        let mut dependency_progress = 100.0 / total_deps * i as f32;
        let next_dependency_progress = 100.0 / total_deps * (i as f32 + 1.0);
        let dependency_progress_str = format!("[{:.1}%]", dependency_progress);

        log::info!(
            "{}",
            format!(
                "{}: ==== Dependency '{}', removing {} flags =====",
                dependency_progress_str,
                dependency_name,
                config.len()
            )
        );

        let mut feature_buffer =
            DependencyFeaturePermutator::new(Vec::from_iter(config.clone().into_iter()));

        let progress_step =
            (next_dependency_progress - dependency_progress) / feature_buffer.left_count() as f32;

        while !feature_buffer.features_left() {
            let feature_progress_str = format!("[{:.1}%]", dependency_progress);

            let (permutated_features, removed_feature) = feature_buffer.permutated_features();

            log::info!(
                "{}",
                format!(
                    "{}: Prune '{}' feature flag from '{}'",
                    feature_progress_str, removed_feature, dependency_name,
                )
            );

            if let Err(e) = toml.replace_dependency_features(dependency_name, permutated_features) {
                log::error!("Error while pruning feature flag. error: {e}");
                continue; // skip this permutation
            }

            if let Err(e) = toml.flush() {
                log::error!("Error while saving modified toml file. error: {e}");
                continue; // skip this permutation
            }

            log::debug!(
                "{}: {}",
                feature_progress_str,
                "Try compiling without feature flag."
            );

            match toml.try_compile() {
                Ok(_) => {
                    feature_buffer
                        .successfully_removed_features
                        .insert(removed_feature.clone());

                    log::debug!(
                        "{}: {}",
                        feature_progress_str,
                        "Successfully compiled without feature.flag."
                    );
                }
                Err(e) => {
                    feature_buffer
                        .unsuccessfully_removed_features
                        .insert(removed_feature.clone());

                    log::debug!(
                        "{}",
                        format!(
                            "{}: Failed to compile without feature flag. error: {}",
                            feature_progress_str, e
                        )
                    );
                }
            }

            toml.reset_dependencies()?;

            dependency_progress += progress_step;
        }

        log::debug!(
            "{}: Finished stripping feature flags from dependency {}.",
            dependency_progress_str,
            toml.crate_name()
        );

        if !feature_buffer.successfully_removed_features.is_empty() {
            workspace_report.add_permutated_dependency(
                dependency_name.clone(),
                feature_buffer.original_features,
                feature_buffer.successfully_removed_features,
                feature_buffer.unsuccessfully_removed_features,
            );
        }
    }

    final_report.add_workspace_crate(toml.crate_name(), workspace_report);

    Ok(())
}
