use clap::Parser;
use subcommands::UnusedFeatures;

mod create_dependencies;
mod editable_toml;
mod feature_buffer;
mod in_memory_toml;
mod loaded_toml;
mod subcommands;

pub(crate) mod report;
pub(crate) mod utils;

pub use editable_toml::TomlEdit;
pub use in_memory_toml::TomlInMemory;
pub use loaded_toml::Toml;
pub use report::{Report, ReportDependencyEntry, WorkspaceCrate};

fn main() {
    let subcommand = UnusedFeatures::parse();
    match subcommand.execute() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn default_features_false_and_no_custom_features() {
        let toml = "
            test4 = { version=\"0.5.0\", default-features = false, features =[]}
        ";
    }

    #[test]
    fn default_features_false_and_custom_features() {
        let toml = "
            test1 = { version=\"0.5.0\", default-features = false, features = [\"f1\", \"f2\", \"f3\"]}
        ";
    }

    #[test]
    fn default_features_true_and_no_custom_features() {
        let toml = "
            test3 = { version=\"0.5.0\", default-features = true, features  = []}
        ";
    }

    #[test]
    fn default_features_true_and_custom_features() {
        let toml = "
            test2 = { version=\"0.5.0\", default-features = true, features  = [\"f1\", \"f2\", \"f3\"]}
        ";
    }
}
