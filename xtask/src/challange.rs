use std::{path::PathBuf, process::Command};

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Options {
    /// Package binary to build
    #[arg(value_enum)]
    pub challange: Challange,

    /// Maelstrom binary location
    #[arg(short, long, default_value = "maelstrom")]
    pub maelstrom_bin: PathBuf,

    /// Build and run the release target
    #[clap(long)]
    pub release: bool,
}

#[derive(Clone, ValueEnum, Parser, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Challange {
    /// Build and run echo challenge
    Echo,
    /// Build and run unique id challenge
    UniqueId,
    /// Build and run single broadcast challenge
    SingleBroadcast,
    /// Build and run multi broadcast challenge
    MultiBroadcast,
    /// Build and run faulty broadcast challenge
    FaultyBroadcast,
    /// Build and run efficient broadcast challenge
    EfficientBroadcast,
    /// Build and run efficient broadcast two challenge
    EfficientBroadcast2,
}

impl Challange {
    pub fn get_name(&self) -> String {
        use Challange::*;
        match self {
            Echo => "echo",
            UniqueId => "unique_id",
            SingleBroadcast | MultiBroadcast | FaultyBroadcast | EfficientBroadcast | EfficientBroadcast2 => "broadcast",
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

/// Get maelstorm Arguments based on challenge
fn get_maelstrom_args(challange: &Challange, bin_path: String) -> Vec<&str> {
    use Challange::*;
    let bin_path: &'static str = Box::leak(Box::new(bin_path));
    match challange {
        Echo => {
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
        UniqueId => {
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
        SingleBroadcast => {
            vec![
                "test",
                "-w",
                "broadcast",
                "--bin",
                bin_path,
                "--time-limit",
                "20",
                "--rate",
                "10",
                "--node-count",
                "1",
            ]
        }
        MultiBroadcast => {
            vec![
                "test",
                "-w",
                "broadcast",
                "--bin",
                bin_path,
                "--node-count",
                "5",
                "--time-limit",
                "20",
                "--rate",
                "10",
            ]
        }
        FaultyBroadcast => {
            vec![
                "test",
                "-w",
                "broadcast",
                "--bin",
                bin_path,
                "--node-count",
                "5",
                "--time-limit",
                "20",
                "--rate",
                "10",
                "--nemesis",
                "partition",
            ]
        }
        EfficientBroadcast => {
            std::env::set_var("TOPOLOGY", "ignore");
            std::env::set_var("TICK_TIME", "300");
            vec![
                "test",
                "-w",
                "broadcast",
                "--bin",
                bin_path,
                "--node-count",
                "25",
                "--time-limit",
                "20",
                "--rate",
                "100",
                "--latency",
                "100",
            ]
        }
        EfficientBroadcast2 => {
            vec![
                "test",
                "-w",
                "broadcast",
                "--bin",
                bin_path,
                "--node-count",
                "25",
                "--time-limit",
                "20",
                "--rate",
                "100",
                "--latency",
                "100",
            ]
        }
    }
}

/// build and run the challenge
pub fn run(opts: Options) {
    let profile = if opts.release { "release" } else { "debug" };
    let bin_name = opts.challange.get_name();
    let bin_path = format!("{}/{}/{}", env!("CARGO_TARGET_DIR"), profile, bin_name);
    build(opts.release, bin_name);
    let status = Command::new(opts.maelstrom_bin)
        .args(&get_maelstrom_args(&opts.challange, bin_path))
        .status()
        .expect("failed to run!");
    assert!(status.success());
}

/// list challenges
pub fn list() {
    print!(
        "{}",
        Challange::value_variants()
            .iter()
            .map(|var| format!("{}\n", var.to_possible_value().unwrap().get_name()))
            .collect::<String>()
    )
}
