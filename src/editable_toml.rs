use toml_edit::{Array, Document, Formatted, InlineTable, Item, Value};

/// An in memory toml file that can be used to edit the toml file without altering formatting, spaces, comments, etc..
pub struct TomlEdit {
    pub toml_document: Document,
    original_dependencies: Item,
}

impl TomlEdit {
    /// Crates a in-memory toml definition from the given toml contents.
    pub fn new(toml_contents: String) -> anyhow::Result<Self> {
        let toml_document = toml_contents.parse::<Document>()?;

        let original_dependencies = toml_document
            .get("dependencies")
            .ok_or_else(|| anyhow::anyhow!("Dependencies tag not found in toml document"))?
            .clone();

        Ok(Self {
            toml_document,
            original_dependencies,
        })
    }

    /// Replaces the dependency features with the given features.
    pub fn replace_dependency_features(
        &mut self,
        dependency: &String,
        features: Vec<String>,
    ) -> anyhow::Result<()> {
        // Get dependencies section.
        let dependencies = self
            .toml_document
            .get_mut("dependencies")
            .ok_or_else(|| anyhow::anyhow!("Dependencies tag not found in toml document"))?;

        // Find dependency.
        let dependency = dependencies
            .get_mut(dependency)
            .ok_or_else(|| anyhow::anyhow!("Dependency not found in toml document"))?;

        let features_to_add = Array::from_iter(features.into_iter());

        // Short dependency notation `x = "1.0"`
        if let Some(version) = dependency.as_str() {
            // We will have to transform the short notation to a detailed notation.

            let mut new_detailed_dependency = InlineTable::new();

            if !features_to_add.is_empty() {
                // Add the features explicitly.
                new_detailed_dependency.insert("features", Value::Array(features_to_add));
            }

            // Disable all default features, we provide them explicitly.
            new_detailed_dependency
                .insert("default-features", Value::Boolean(Formatted::new(false)));

            // Add the version.
            new_detailed_dependency.insert(
                "version",
                Value::String(Formatted::new(version.to_string())),
            );

            // Overwrite the dependency with the new one.
            *dependency = Item::Value(Value::InlineTable(new_detailed_dependency));

            return Ok(());
        }

        // Long dependency notation `x = { version = "1.0"}`
        if let Some(dependency_table) = dependency.as_inline_table_mut() {
            // Check if the 'default-features' tag should be inserted.
            if let Some(default_features) = dependency_table.get_mut("default-features") {
                // Overwrite the current 'default-features' tag with the new one.
                *default_features = Value::Boolean(Formatted::new(false));
            } else {
                // Insert the new 'default-features' tag.
                dependency_table.insert("default-features", Value::Boolean(Formatted::new(false)));
            };

            let current_features_array = dependency_table
                .get_mut("features")
                .and_then(|features_array| features_array.as_array_mut());

            // Insert array if non existent crate it.
            if current_features_array.is_none() {
                // Only define the array it there are features to add.
                if !features_to_add.is_empty() {
                    dependency_table.insert("features", Value::Array(features_to_add));
                }

                return Ok(());
            }

            let current_features_array_mut = current_features_array.unwrap();

            // Overwrite feature array.
            *current_features_array_mut = features_to_add; // Safe.

            // Remove feature array if empty.
            if current_features_array_mut.is_empty() {
                dependency_table.remove("features");
            }
        } else {
            return Err(anyhow::anyhow!("The toml document is wrongly formatted."));
        };

        Ok(())
    }

    /// Resets the in-memory toml dependencies.
    pub fn reset(&mut self) -> anyhow::Result<()> {
        let current_deps = self
            .toml_document
            .get_mut("dependencies")
            .ok_or_else(|| anyhow::anyhow!("Dependencies tag not found in toml document"))?;

        *current_deps = self.original_dependencies.clone();

        Ok(())
    }

    /// Returns a toml-formatted string of the current loaded in-memory toml definition.
    pub fn serialize(&self) -> anyhow::Result<String> {
        Ok(self.toml_document.to_string())
    }
}
