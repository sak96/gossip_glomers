//! Utility to run Gossip Glomers challenge.
use clap::Parser;

pub mod challange;
pub mod serve;

/// CLI to run Gossip Glomers challenge.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Xtask {
    /// Subcommand for CLI.
    #[clap(subcommand)]
    pub command: XtaskCommand,
}

/// Subcommand for CLI.
#[derive(Debug, Parser)]
pub enum XtaskCommand {
    /// Run some challenge.
    Run(challange::RunOptions),
    /// Serve results of previous run challenges.
    Serve(serve::ServeOptions),
    /// List all challenges.
    List,
}
