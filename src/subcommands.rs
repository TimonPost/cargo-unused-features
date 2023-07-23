pub mod analyze;
pub mod prune;
pub mod report_builder;

use clap::Parser;

use self::{analyze::AnalyzeCommand, prune::PruneCommand, report_builder::ReportBuildingCommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "cargo-unused-features")]
pub enum Cargo {
    Analyze(AnalyzeCommand),
    BuildReport(ReportBuildingCommand),
    Prune(PruneCommand),
}

impl Cargo {
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            Cargo::Analyze(args) => args.execute(),
            Cargo::BuildReport(args) => args.execute(),
            Cargo::Prune(args) => args.execute(),
        }
    }
}
