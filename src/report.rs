use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

pub const REPORT_VERSION: u16 = 0;

/// Serializable Report.
#[derive(Serialize, Deserialize, Default)]
pub struct Report {
    pub(crate) version: u16,
    /// The name of the root workspace or crate.
    pub(crate) root_name: String,
    /// The crates in the workspace or just a single crate.
    pub(crate) workspace_crates: HashMap<String, WorkspaceCrate>,
}

impl Report {
    pub fn new(root_name: &str) -> Self {
        Report {
            root_name: root_name.to_string(),
            workspace_crates: Default::default(),
            version: REPORT_VERSION,
        }
    }

    /// Adds a new workspace crate to the report.
    pub fn add_workspace_crate(&mut self, crate_name: String, workspace_crate: WorkspaceCrate) {
        if workspace_crate.dependencies.is_empty() {
            return;
        }

        self.workspace_crates.insert(crate_name, workspace_crate);
    }

    /// Deserializes a report from the the given json-file.
    pub fn from(path: &Path) -> anyhow::Result<Report> {
        let mut contents = String::new();

        log::info!("Loading report from {}.", path.display());

        let mut permutated_toml_file = File::open(path).unwrap();
        permutated_toml_file.read_to_string(&mut contents).unwrap();

        if let Ok(report) = serde_json::from_str(&contents) {
            log::info!("Successfully loaded report from {}.", path.display());
            Ok(report)
        } else {
            let error = format!("Failed to deserialize report from {}, maybe an old report? Current version is {}, make sure this is the same one in the report.", path.display(), REPORT_VERSION);
            Err(anyhow::anyhow!(error))
        }
    }

    /// Serializes and flushes the report to the given path.
    pub fn flush(&self, path: &Path) -> anyhow::Result<()> {
        log::debug!("Write report to {}.", path.display());
        let report = serde_json::to_string_pretty(&self)?;
        let mut permutated_toml_file = File::create(path)?;
        permutated_toml_file.write_all(report.as_bytes())?;
        log::info!("Written json report to {}.", path.display());
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct WorkspaceCrate {
    /// Full path to the crate.
    pub(crate) full_path: String,
    /// The dependencies of the crate and a report per dependency.
    pub(crate) dependencies: HashMap<String, ReportDependencyEntry>,
}

impl WorkspaceCrate {
    pub fn new(full_path: &Path) -> Self {
        WorkspaceCrate {
            full_path: full_path.to_string_lossy().to_string(),
            dependencies: Default::default(),
        }
    }

    pub fn add_permutated_dependency(
        &mut self,
        dependency_name: String,
        all_features: HashSet<String>,
        successfully_removed_features: HashSet<String>,
        unsuccessfully_removed_features: HashSet<String>,
    ) {
        self.dependencies.insert(
            dependency_name,
            ReportDependencyEntry {
                original_features: all_features,
                successfully_removed_features,
                unsuccessfully_removed_features,
            },
        );
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ReportDependencyEntry {
    /// The original explicitly or implicitly enabled features of the dependency.
    pub(crate) original_features: HashSet<String>,
    /// The features that were successfully removed.
    pub(crate) successfully_removed_features: HashSet<String>,
    /// The features that were unsuccessfully removed.
    pub(crate) unsuccessfully_removed_features: HashSet<String>,
}
