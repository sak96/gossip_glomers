use std::{env::var, path::PathBuf, process::Command};

use clap::{Parser, ValueEnum};
use convert_case::{Case, Casing};

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
    /// Echo challenge
    Echo,
    /// Unique id challenge
    UniqueIds,
    /// Single broadcast challenge
    SingleBroadcast,
    /// Multi broadcast challenge
    MultiBroadcast,
    /// Faulty broadcast challenge
    FaultyBroadcast,
    /// Efficient broadcast challenge
    EfficientBroadcast,
    /// Efficient broadcast two challenge
    EfficientBroadcast2,
    /// Grow Only Counter
    GrowOnlyCounter,
}

impl Challange {
    pub fn get_name(&self) -> String {
        match self {
            Challange::Echo => "echo",
            Challange::UniqueIds => "unique_ids",
            Challange::SingleBroadcast
            | Challange::MultiBroadcast
            | Challange::FaultyBroadcast
            | Challange::EfficientBroadcast
            | Challange::EfficientBroadcast2 => "broadcast",
            Challange::GrowOnlyCounter => "g_counter",
        }
        .to_string()
    }
}

/// Builds the challenge binary using cargo.
fn build(release: bool, bin_name: &str) -> String {
    let mut args = vec!["build", "--bin", bin_name];
    let profile = if release {
        args.push("--release");
        "release"
    } else {
        "debug"
    };
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to build!");
    assert!(status.success());
    format!(
        "{}/{}/{}",
        var("CARGO_TARGET_DIR").unwrap_or("target".to_string()),
        profile,
        bin_name
    )
}

struct MaelStormCommand(Command);

impl MaelStormCommand {
    /// create command to execute maelstorm.
    pub fn new(
        maelstorm_bin: &PathBuf,
        bin_path: &str,
        bin_name: &str,
        node_count: usize,
        time_limit: usize,
    ) -> Self {
        let mut command = Command::new(maelstorm_bin);
        command
            .arg("test")
            .args(["-w", &bin_name.to_case(Case::Kebab)])
            .args(["--bin", bin_path])
            .args(["--node-count", &node_count.to_string()])
            .args(["--time-limit", &time_limit.to_string()]);
        Self(command)
    }

    /// set any environment variable required by maelstorm or binary.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.0.env(key, value);
        self
    }

    /// Add partitioning.
    pub fn partition(mut self) -> Self {
        self.0.args(["--nemesis", "partition"]);
        self
    }

    /// Set total availability.
    pub fn total_availability(mut self) -> Self {
        self.0.args(["--availability", "total"]);
        self
    }

    /// Changes rate.
    pub fn rate(mut self, rate: usize) -> Self {
        self.0.args(["--rate", &rate.to_string()]);
        self
    }

    /// Changes latency.
    pub fn latency(mut self, latency: usize) -> Self {
        self.0.args(["--latency", &latency.to_string()]);
        self
    }

    /// Changes topology.
    pub fn topology(mut self, topology: &str) -> Self {
        self.0.args(["--topology", topology]);
        self
    }

    /// Executes command and makes sure it was a success.
    pub fn execute(self) {
        let mut command = self.0;
        let status = command
            .status()
            .unwrap_or_else(|_| panic!("command invocation failed {command:?}!"));
        assert!(status.success());
    }
}

/// build and run the challenge
pub fn run(opts: Options) {
    let bin_name = opts.challange.get_name();
    let bin_path = build(opts.release, &bin_name);
    match opts.challange {
        Challange::Echo => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 1, 10).execute();
        }
        Challange::UniqueIds => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 3, 30)
                .partition()
                .rate(1000)
                .total_availability()
                .execute();
        }
        Challange::SingleBroadcast => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 1, 20)
                .rate(10)
                .execute();
        }
        Challange::MultiBroadcast => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 5, 20)
                .rate(10)
                .execute();
        }
        Challange::FaultyBroadcast => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 5, 20)
                .rate(10)
                .partition()
                .execute();
        }
        Challange::EfficientBroadcast => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 25, 20)
                .rate(100)
                .latency(100)
                .topology("tree4")
                .execute();
        }
        Challange::EfficientBroadcast2 => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 25, 20)
                .env("TICK_TIME", "1000")
                .rate(100)
                .latency(100)
                .execute();
        }
        Challange::GrowOnlyCounter => {
            MaelStormCommand::new(&opts.maelstrom_bin, &bin_path, &bin_name, 3, 20)
                .rate(100)
                .partition()
                .execute();
        }
    }
}

/// list challenges
pub fn list() {
    print!(
        "{}",
        Challange::value_variants()
            .iter()
            .map(|var| format!("{}\n", var.to_possible_value().unwrap().get_name()))
            .collect::<String>()
    );
}
