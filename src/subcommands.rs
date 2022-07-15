pub mod analyze;
pub mod prune;
pub mod report_builder;

use clap::Parser;

use self::{analyze::AnalyzeCommand, prune::PruneCommand, report_builder::ReportBuildingCommand};

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
pub enum UnusedFeatures {
    Analyze(AnalyzeCommand),
    BuildReport(ReportBuildingCommand),
    Prune(PruneCommand),
}

impl UnusedFeatures {
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            UnusedFeatures::Analyze(args) => args.execute(),
            UnusedFeatures::BuildReport(args) => args.execute(),
            UnusedFeatures::Prune(args) => args.execute(),
        }
    }
}
