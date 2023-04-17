//! Utility to run Gossip Glomers challenge.
use clap::Parser;
use xtask::{challange, serve, Xtask, XtaskCommand};

/// Parse and run the CLI.
fn main() {
    let opts = Xtask::parse();
    match opts.command {
        XtaskCommand::Run(options) => challange::run(options),
        XtaskCommand::Serve(options) => serve::serve(options),
        XtaskCommand::List => challange::list(),
    }
}
