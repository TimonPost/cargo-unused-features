use clap::Parser;
use subcommands::Cargo;

mod cargo_project;
mod create_dependencies;
mod editable_toml;
mod feature_buffer;
mod in_memory_toml;
mod subcommands;

pub(crate) mod report;
pub(crate) mod utils;

pub use cargo_project::CargoProject;
pub use editable_toml::TomlEdit;
pub use in_memory_toml::TomlInMemory;
pub use report::{Report, ReportDependencyEntry, WorkspaceCrate};

fn main() {
    let subcommand = Cargo::parse();
    match subcommand.execute() {
        Ok(_) => log::info!("Finished the process"),
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(1);
        }
    }
}
