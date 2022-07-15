use std::{fs, path::Path};

use crate::{utils, TomlEdit};
pub use clap::Parser;

use crate::Report;

/// Prunes the unused, but, enabled feature flags reported by `cargo unused-feature analyze`.
#[derive(Parser, Debug, Clone, Default)]
#[clap(author, version, about, long_about = None)]
pub struct PruneCommand {
    /// Absolute path to the json report generated by `cargo unused-features analyze`.
    #[clap(short = 'i', long = "input", value_parser)]
    pub input_json_path: String,
    /// The log level (debug, info, warn, error, off). Defaults to info.
    #[clap(short = 'l', long = "l")]
    pub log_level: Option<String>,
}

impl PruneCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        utils::initialize_logger(self.log_level);

        log::info!("Executing prune command.");

        let report = Report::from(Path::new(&self.input_json_path))?;

        log::info!("Loaded removal suggestions from {}.", self.input_json_path);

        for (crate_name, workspace_crate) in report.workspace_crates {
            log::info!("Start pruning features of crate {crate_name}.");

            let contents = fs::read_to_string(&Path::new(&workspace_crate.full_path))?;

            let mut toml = TomlEdit::new(contents)?;

            for (dep_name, dependency) in workspace_crate.dependencies {
                let diff = dependency
                    .original_features
                    .difference(&dependency.successfully_removed_features);

                log::info!("Start pruning features of dependency {dep_name}.");

                match toml.replace_dependency_features(
                    &dep_name,
                    diff.cloned().into_iter().collect::<Vec<String>>(),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "Failed to remove features from dependency {}: {}",
                            dep_name,
                            e
                        );
                    }
                }
            }

            let new_contents = toml.serialize()?;
            fs::write(&workspace_crate.full_path, new_contents)?;
            log::info!(
                "Updated {} with pruned unused, but, enabled feature flags.",
                workspace_crate.full_path
            );
        }
        Ok(())
    }
}
