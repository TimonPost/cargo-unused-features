pub mod analyze;
pub mod remove;
pub mod report_builder;

use clap::Parser;

use self::{analyze::AnalyzeCommand, remove::PurgeCommand, report_builder::ReportBuildingCommand};

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
pub enum UnusedFeatures {
    Analyze(AnalyzeCommand),
    BuildReport(ReportBuildingCommand),
    Prune(PurgeCommand),
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
