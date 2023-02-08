use std::ops::Deref;

use cargo_toml::{DependencyDetail, DepsSet, Manifest};
use serde::Serialize;

/// In memory representation of a Cargo.toml file.
/// This can be edited and serialized but it will not keep original formatting.
pub struct TomlInMemory {
    pub manifest: Manifest,
    pub original_dependencies: DepsSet,
}

impl TomlInMemory {
    /// Crates a new in memory representation of a Cargo.toml file.
    pub fn new(toml_contents: String) -> anyhow::Result<Self> {
        let manifest = Manifest::from_str(&toml_contents)?;

        Ok(Self {
            original_dependencies: manifest.dependencies.clone(),
            manifest,
        })
    }

    /// Serializes the in memory toml to a toml formatted string.
    pub fn serialize(&self) -> anyhow::Result<String> {
        let mut toml_buffer = String::new();
        let serializer = toml::ser::Serializer::new(&mut toml_buffer);
        self.manifest.serialize(serializer)?;
        Ok(toml_buffer)
    }

    /// Replaces the dependency features with the given features.
    pub fn replace_dependency_feature(
        &mut self,
        dependency: &String,
        features: Vec<String>,
    ) -> anyhow::Result<()> {
        let dependency = self
            .manifest
            .dependencies
            .get_mut(dependency)
            .ok_or_else(|| anyhow::anyhow!("Dependency not found"))?;

        match dependency {
            // Short dependency notation `x = "1.0"`
            cargo_toml::Dependency::Simple(version) => {
                let new_dependency = DependencyDetail {
                    version: Some(version.clone()),
                    features,
                    default_features: false,
                    ..Default::default()
                };
                *dependency = cargo_toml::Dependency::Detailed(new_dependency);
            }
            // Detailed dependency notation `x = { version = "1.0", features = ["a", "b"] }`
            cargo_toml::Dependency::Detailed(detailed) => {
                detailed.features = features;
                detailed.default_features = false;
            }
            cargo_toml::Dependency::Inherited(_inherited) => {}
        };

        Ok(())
    }

    /// Resets the dependencies of this in memory toml definition.
    /// This does not update any files.
    pub fn reset_dependencies(&mut self) -> anyhow::Result<()> {
        self.manifest.dependencies = self.original_dependencies.clone();
        Ok(())
    }
}

impl Deref for TomlInMemory {
    type Target = Manifest;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}
