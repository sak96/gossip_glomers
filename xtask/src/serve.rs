use std::{path::PathBuf, process::Command};

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Options {
    /// Maelstrom binary location
    #[arg(short, long, default_value = "maelstrom")]
    pub maelstrom_bin: PathBuf,
}

pub fn serve(opts: Options) {
    let status = Command::new(opts.maelstrom_bin)
        .args(["serve"])
        .status()
        .expect("failed to serve!");
    assert!(status.success());
}
