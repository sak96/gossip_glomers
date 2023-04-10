use std::{path::PathBuf, process::Command};

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Options {
    /// Package binary to build
    #[arg(value_enum)]
    pub program: Program,

    /// Maelstrom binary location
    #[arg(short, long, default_value = "maelstrom")]
    pub maelstrom_bin: PathBuf,

    /// Build and run the release target
    #[clap(long)]
    pub release: bool,
}

#[derive(Clone, ValueEnum, Parser, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Program {
    /// Build and run echo challenge
    Echo,
    /// Build and run echo challenge
    UniqueId,
}

impl Program {
    pub fn get_name(&self) -> String {
        match self {
            Program::Echo => "echo",
            Program::UniqueId => "unique_id",
        }
        .to_string()
    }
}

/// Builds the project
fn build(release: bool, bin_name: String) {
    let mut args = vec!["build", "--bin", &bin_name];
    if release {
        args.push("--release")
    }
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to build!");
    assert!(status.success());
}

/// Get maelstorm Arguments based on program
fn get_maelstrom_args(program: &Program, bin_path: String) -> Vec<&str> {
    let bin_path: &'static str = Box::leak(Box::new(bin_path));
    match program {
        Program::Echo => {
            vec![
                "test",
                "-w",
                "echo",
                "--bin",
                bin_path,
                "--node-count",
                "1",
                "--time-limit",
                "10",
            ]
        }
        Program::UniqueId => {
            vec![
                "test",
                "-w",
                "unique-ids",
                "--bin",
                bin_path,
                "--time-limit",
                "30",
                "--rate",
                "1000",
                "--node-count",
                "3",
                "--availability",
                "total",
                "--nemesis",
                "partition",
            ]
        }
    }
}

pub fn run(opts: Options) {
    let profile = if opts.release { "release" } else { "debug" };
    let bin_name = opts.program.get_name();
    let bin_path = format!("{}/{}/{}", env!("CARGO_TARGET_DIR"), profile, bin_name);
    build(opts.release, bin_name);
    let status = Command::new(opts.maelstrom_bin)
        .args(&get_maelstrom_args(&opts.program, bin_path))
        .status()
        .expect("failed to run!");
    assert!(status.success());
}
