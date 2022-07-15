use std::collections::{HashMap, HashSet};

use cargo_toml::DependencyDetail;

/// Crate dependencies and their features.
pub struct CrateDependencies {
    /// The dependencies of the crate.
    pub(crate) dependencies: HashMap<String, DependencyDetail>,
    /// The dependencies by name and their features.
    pub(crate) dependency_features: HashMap<String, HashSet<String>>,
}

impl CrateDependencies {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::default(),
            dependency_features: HashMap::default(),
        }
    }
}

impl std::fmt::Debug for CrateDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found the following dependencies:")?;

        for (name, features) in &self.dependency_features {
            let features = features.clone().into_iter().collect::<Vec<String>>();
            writeln!(f, "{}=[{}]", name, features.join(","))?;
        }
        Ok(())
    }
}

impl Default for CrateDependencies {
    fn default() -> Self {
        Self::new()
    }
}
