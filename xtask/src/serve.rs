//! Module to serve result of challange.
use std::{path::PathBuf, process::Command};

use clap::Parser;

/// Options to serve command.
#[derive(Parser, Debug)]
pub struct ServeOptions {
    /// Maelstrom binary location
    #[arg(short, long, default_value = "maelstrom")]
    pub maelstrom_bin: PathBuf,
}

/// Serve maelstrom results.
pub fn serve(opts: ServeOptions) {
    let status = Command::new(opts.maelstrom_bin)
        .args(["serve"])
        .status()
        .expect("failed to serve!");
    assert!(status.success());
}
