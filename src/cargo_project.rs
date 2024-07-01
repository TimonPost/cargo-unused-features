use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use crate::TomlInMemory;
use cargo::{
    core::{
        compiler::{BuildConfig, CompileMode},
        Shell, Verbosity, Workspace,
    },
    ops::{CompileFilter, CompileOptions},
    Config,
};
use cargo_metadata::Metadata;

use crate::{create_dependencies::CrateDependencies, subcommands::analyze::AnalyzeCommand};

/// In-memory toml file.
pub struct CargoProject {
    /// The original toml file contents.
    original: String,
    /// The directory of the toml file.
    directory: Box<Path>,
    /// The absolute path of the toml file.
    toml_path: Box<Path>,
    /// The in memory loaded toml definition that can be edited and serialized.
    in_memory_toml: TomlInMemory,
    /// Configurations.
    config: AnalyzeCommand,
}

impl CargoProject {
    pub fn new(directory: &Path, config: AnalyzeCommand) -> anyhow::Result<Self> {
        let toml_path = directory.join("Cargo.toml");

        log::debug!("Loading '{}' ...", toml_path.display());

        let mut file = File::options().read(true).write(false).open(&toml_path)?;
        let mut toml_contents = String::new();
        file.read_to_string(&mut toml_contents)?;

        log::debug!("Successfully read the toml file.");

        log::debug!("Parsing toml definition ...");

        let in_memory_toml = TomlInMemory::new(toml_contents.clone())?;

        log::debug!("Successfully parsed the toml file.");

        Ok(CargoProject {
            original: toml_contents,
            directory: Box::from(directory),
            toml_path: toml_path.into_boxed_path(),
            in_memory_toml,
            config,
        })
    }

    /// Returns the configuration.
    pub fn config(&self) -> &AnalyzeCommand {
        &self.config
    }

    /// Returns a list of absolute path names to the workspace members of this toml file.
    /// No paths are returned if this is not a workspace toml file.
    pub fn workspace_members(&self) -> Vec<Box<Path>> {
        if let Some(workspace) = &self.in_memory_toml.workspace {
            let mut members = Vec::new();
            for member in &workspace.members {
                members.push(Box::from(self.directory.join(&member)));
            }
            members
        } else {
            vec![]
        }
    }

    /// Returns the crate name in the toml file.
    pub fn crate_name(&self) -> String {
        self.in_memory_toml
            .package
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    }

    /// Returns the absolute path of the toml file.
    pub fn report_path(&self) -> Box<Path> {
        let report_dir = self
            .config
            .report_dir
            .clone()
            .unwrap_or_else(|| self.workspace_path().display().to_string());

        Box::from(Path::new(&report_dir).join("report.json"))
    }

    /// Returns the absolute path of the toml file.
    pub fn toml_path(&self) -> Box<Path> {
        self.toml_path.clone()
    }

    /// Returns the absolute path to the workspace directory of this toml file.
    pub fn workspace_path(&self) -> Box<Path> {
        self.directory.clone()
    }

    /// Flushes the changes made to the in memory toml to the toml file on disk.
    pub fn flush(&self) -> anyhow::Result<()> {
        let toml_contents = self.in_memory_toml.serialize()?;

        fs::write(&self.toml_path(), toml_contents.as_bytes())?;

        Ok(())
    }

    /// Replaces the dependency features with the given features.
    pub fn replace_dependency_features(
        &mut self,
        dependency_name: &String,
        new_features: Vec<String>,
    ) -> anyhow::Result<()> {
        self.in_memory_toml
            .replace_dependency_feature(dependency_name, new_features)
    }

    /// Resets the dependencies of this in memory toml definition.
    /// This does not update any files.
    pub fn reset_dependencies(&mut self) -> anyhow::Result<()> {
        self.in_memory_toml.reset_dependencies()
    }

    /// Tries to compile the project of the this toml file.    
    pub fn try_compile(&self) -> anyhow::Result<()> {
        let config = Config::default()?;

        let buffer = Box::new(Vec::new());
        *config.shell() = Shell::from_write(buffer);
        config.shell().set_verbosity(Verbosity::Quiet);

        let mut compile_options = CompileOptions::new(&config, CompileMode::Build)?;

        // Pass custom target if configured.
        compile_options.build_config = BuildConfig::new(
            &config,
            self.config.parallel_build_jobs,
            false,
            self.config.build_target.as_slice(),
            CompileMode::Build,
        )?;

        compile_options.filter = CompileFilter::Only {
            all_targets: self.config.build_target.is_empty(), // if no targets specified, build all targets.
            lib: cargo::ops::LibRule::False,
            bins: cargo::ops::FilterRule::Just(vec![]),
            examples: cargo::ops::FilterRule::Just(vec![]),
            tests: cargo::ops::FilterRule::Just(vec![]),
            benches: cargo::ops::FilterRule::Just(vec![]),
        };

        if let CompileFilter::Only {
            lib,
            bins,
            examples,
            tests,
            benches,
            ..
        } = &mut compile_options.filter
        {
            if self.config.build_bins {
                *bins = cargo::ops::FilterRule::All;
            }

            if self.config.build_lib {
                *lib = cargo::ops::LibRule::True;
            }

            if self.config.build_examples {
                *examples = cargo::ops::FilterRule::All;
            }

            if self.config.build_tests {
                *tests = cargo::ops::FilterRule::All;
            }

            if self.config.build_benches {
                *benches = cargo::ops::FilterRule::All;
            }
        }

        let workspace = Workspace::new(&self.toml_path(), &config)?;

        cargo::ops::compile(&workspace, &compile_options)
            .map_err(|e| anyhow::anyhow!("Failed to compile toml document: {}", e))?;

        Ok(())
    }

    /// Gathers metadata of the toml file and returns the crate dependencies with their features.
    pub fn gather_meta_data(&self) -> CrateDependencies {
        log::debug!("Fetching crate metadata...");

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(self.toml_path())
            .current_dir(self.workspace_path())
            .exec()
            .expect("failed to fetch metadata");

        log::debug!("Fetched crate metadata.");

        log::debug!("Analyzing metadata...");

        let mut crate_dependencies = self.crate_dependencies();

        self.get_permutable_features(&metadata, &mut crate_dependencies);

        log::debug!("{:?}", crate_dependencies);

        crate_dependencies
    }

    /// Returns the crate dependencies of this toml file.
    fn crate_dependencies(&self) -> CrateDependencies {
        let mut crate_dependencies = CrateDependencies::new();

        for (name, dep) in self.in_memory_toml.dependencies.clone() {
            let dependency = match dep {
                cargo_toml::Dependency::Simple(version) => {
                    // We replace simple notation by detailed notation to make it easier to work with.
                    cargo_toml::DependencyDetail {
                        version: Some(version),
                        default_features: true,
                        ..Default::default()
                    }
                }
                cargo_toml::Dependency::Detailed(detailed) => detailed,
                cargo_toml::Dependency::Inherited(_inherited) => cargo_toml::DependencyDetail {
                    ..Default::default()
                },
            };

            crate_dependencies
                .dependencies
                .insert(name.clone(), dependency);
        }

        crate_dependencies
    }

    /// Features can be enabled in different ways.
    ///
    /// 1) Manually,
    /// 2) Disabling default features
    /// 3) Enabling default features
    /// 4) Default features are always enabled unless disabled.
    ///
    /// Also, features can enable 0-n other features.
    /// We need to gather all enabled features being it implicitly or explicitly.
    /// Those features will be permutated later on.
    fn get_permutable_features(
        &self,
        metadata: &Metadata,
        crate_dependencies: &mut CrateDependencies,
    ) {
        log::debug!("Gathering dependencies their features...");

        for package in &metadata.packages {
            let package_name = package.name.clone();

            if self.config.skip_dependencies.contains(&package_name) {
                continue;
            }

            if let Some(crate_dependency) = crate_dependencies.dependencies.get(&package_name) {
                // The manually entered features in toml file.
                let manual_selected_features: HashSet<String> =
                    HashSet::from_iter(crate_dependency.features.clone().into_iter());

                // All features of each dependency.
                let dependency_features = package.features.clone();

                // The features that will be applicable to removal.
                let mut permutation_features = HashSet::new();

                let has_manual_selected_features = !manual_selected_features.is_empty();
                let has_default_features = crate_dependency.default_features;

                // Gather the features that will be applicable to removal.
                // Feature flags might contain a collection of other features.

                fn gather_manual_selected_features(
                    permutation_features: &mut HashSet<String>,
                    manual_selected_features: &HashSet<String>,
                    dependency_features: &HashMap<String, Vec<String>>,
                ) {
                    for manual_selected_feature in manual_selected_features {
                        if let Some(custom_feature_list) =
                            dependency_features.get(manual_selected_feature)
                        {
                            // Features can have 0-n other features as dependencies e.g. 'default=[x,y]'.

                            for custom_feature_list_feature in custom_feature_list {
                                // Must be a public facing feature.
                                if dependency_features.contains_key(custom_feature_list_feature) {
                                    permutation_features
                                        .insert(custom_feature_list_feature.clone());
                                }
                            }
                        }

                        // Also insert the custom feature itself.
                        permutation_features.insert(manual_selected_feature.clone());
                    }
                }

                fn gather_default_enabled_features(
                    permutation_features: &mut HashSet<String>,
                    dependency_features: &HashMap<String, Vec<String>>,
                ) {
                    if let Some(default_features) = dependency_features.get("default") {
                        for default_feature in default_features {
                            // Must be a public facing feature.
                            if dependency_features.contains_key(default_feature) {
                                permutation_features.insert(default_feature.clone());
                            }
                        }
                    }
                }

                // Test the various ways features can be enabled/disabled and gather the explicitly or implicitly enabled features.

                if !has_default_features && !has_manual_selected_features { /* do nothing as there are no features specified */
                } else if !has_default_features && has_manual_selected_features {
                    /* permutate features */
                    gather_manual_selected_features(
                        &mut permutation_features,
                        &manual_selected_features,
                        &dependency_features,
                    );
                } else if has_default_features && !has_manual_selected_features {
                    /* permutate default features */
                    gather_default_enabled_features(
                        &mut permutation_features,
                        &dependency_features,
                    );
                } else if has_default_features && has_manual_selected_features {
                    /* permutate default features and custom features */

                    gather_manual_selected_features(
                        &mut permutation_features,
                        &manual_selected_features,
                        &dependency_features,
                    );
                    gather_default_enabled_features(
                        &mut permutation_features,
                        &dependency_features,
                    );
                }

                // If no features were found then we dont have to record this dependency.
                if !permutation_features.is_empty() {
                    crate_dependencies
                        .dependency_features
                        .insert(package_name.to_string(), permutation_features);
                }
            }
        }
    }
}

impl Drop for CargoProject {
    fn drop(&mut self) {
        // By default we reset the toml always after we mutated it for analyzing purposes.
        // Could be made optional later.
        let mut permutated_toml_file = File::create(&self.toml_path()).unwrap();
        permutated_toml_file
            .write_all(self.original.as_bytes())
            .unwrap();
        log::debug!("Resetting toml file to original.");
    }
}
