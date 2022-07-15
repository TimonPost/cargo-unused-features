use std::collections::HashSet;

/// The feature permutator permutates features and keeps track of successful and unsuccessful removed features.
#[derive(Clone)]
pub struct DependencyFeaturePermutator {
    pub(crate) original_features: HashSet<String>,
    pub successfully_removed_features: HashSet<String>,
    pub unsuccessfully_removed_features: HashSet<String>,
    tmp_features: Vec<String>,
}

impl DependencyFeaturePermutator {
    pub fn new(features: Vec<String>) -> Self {
        DependencyFeaturePermutator {
            original_features: features.clone().into_iter().collect(),
            successfully_removed_features: HashSet::new(),
            unsuccessfully_removed_features: HashSet::new(),
            tmp_features: features,
        }
    }

    /// Removes a feature from the dependency.
    /// Returns the list of current enabled features along with the removed one.
    pub fn permutated_features(&mut self) -> (Vec<String>, String) {
        let mut features = Vec::new();

        let removed = self.remove_feature();

        // Also iterate over unsuccessfully removed features as those could not be removed.
        for feature in self
            .tmp_features
            .iter()
            .chain(self.unsuccessfully_removed_features.iter())
        {
            if feature != &removed {
                features.push(feature.clone())
            }
        }

        (features, removed)
    }

    /// Returns if the features are dependency features left to remove.
    pub fn features_left(&self) -> bool {
        self.tmp_features.is_empty()
    }

    pub fn left_count(&self) -> usize {
        self.tmp_features.len() + self.unsuccessfully_removed_features.len()
    }

    /// Removes a feature from the dependency.
    pub fn remove_feature(&mut self) -> String {
        assert!(!self.tmp_features.is_empty());
        self.tmp_features.remove(self.tmp_features.len() - 1)
    }
}
