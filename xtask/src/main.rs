//! Utility to run Gossip Glomers challenge.
use clap::Parser;

mod challange;
mod serve;

/// CLI to run Gossip Glomers challenge.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Xtask {
    /// Subcommand for CLI.
    #[clap(subcommand)]
    command: XtaskCommand,
}

/// Subcommand for CLI.
#[derive(Debug, Parser)]
enum XtaskCommand {
    /// Run some challenge.
    Run(challange::RunOptions),
    /// Serve results of previous run challenges.
    Serve(serve::ServeOptions),
    /// List all challenges.
    List,
}

/// Parse and run the CLI.
fn main() {
    let opts = Xtask::parse();
    match opts.command {
        XtaskCommand::Run(options) => challange::run(options),
        XtaskCommand::Serve(options) => serve::serve(options),
        XtaskCommand::List => challange::list(),
    }
}
