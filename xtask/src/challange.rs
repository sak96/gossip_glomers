//! Module to handle challenge running and list.
use std::{env::var, path::PathBuf, process::Command};

use clap::{Parser, ValueEnum};
use convert_case::{Case, Casing};

/// Options to run command.
#[derive(Parser, Debug)]
pub struct RunOptions {
    /// Package binary to build
    #[arg(value_enum)]
    pub challange: Challange,

    /// Maelstrom binary location
    #[arg(short, long, env, default_value = "maelstrom")]
    pub maelstrom_bin: PathBuf,

    /// Build and run the release target
    #[clap(long)]
    pub release: bool,

    /// Extra arguments to be passed to maelstrom.
    ///
    /// Example: `--log-stderr`, `--log-net-send`, `--log-net-recv`
    #[clap(last = true)]
    pub extra_args: Vec<String>,
}

/// Challenges from Gossip Glomers.
#[derive(Clone, ValueEnum, Parser, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Challange {
    /// Echo
    Echo,
    /// Unique id
    UniqueIds,
    /// Single node broadcast
    SingleBroadcast,
    /// Multi node broadcast
    MultiBroadcast,
    /// Faulty node broadcast
    FaultyBroadcast,
    /// Efficient broadcast
    EfficientBroadcast,
    /// Efficient broadcast two
    EfficientBroadcast2,
    /// Grow only counter
    GrowOnlyCounter,
}

impl Challange {
    /// Get name of the challenge program.
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

/// Helper for running maelstrom commands.
///
/// [Docs](https://github.com/jepsen-io/maelstrom/blob/main/README.md#cli-options).
struct MaelStromCommand(Command);

struct MaelStromResult(edn_format::Value);

impl MaelStromResult {
    pub fn get_value_at<'a>(&'a self, path: &[edn_format::Value]) -> Option<&'a edn_format::Value> {
        Self::get_value_at_inner(&self.0, path)
    }

    pub fn get_value_at_inner<'a>(
        value: &'a edn_format::Value,
        path: &[edn_format::Value],
    ) -> Option<&'a edn_format::Value> {
        if path.is_empty() {
            return Some(value);
        }
        match value {
            edn_format::Value::Map(map) => {
                let value = map.get(&path[0])?;
                Self::get_value_at_inner(value, &path[1..])
            }
            _ => None,
        }
    }
}

impl MaelStromCommand {
    /// create command to execute maelstrom.
    pub fn new(
        maelstrom_bin: &PathBuf,
        bin_path: &str,
        bin_name: &str,
        node_count: usize,
        time_limit: usize,
        extra_args: &[String],
    ) -> Self {
        let mut command = Command::new(maelstrom_bin);
        command
            .arg("test")
            .args(["-w", &bin_name.to_case(Case::Kebab)])
            .args(["--bin", bin_path])
            .args(["--node-count", &node_count.to_string()])
            .args(["--time-limit", &time_limit.to_string()])
            .args(extra_args);
        Self(command)
    }

    /// set any environment variable required by maelstrom or binary.
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
            .unwrap_or_else(|e| panic!("command invocation failed {command:?} with error {e:?}!"));
        assert!(status.success());
    }

    pub fn get_results() -> MaelStromResult {
        const FILE: &str = "store/current/results.edn";
        MaelStromResult(
            edn_format::parse_str(&std::fs::read_to_string(FILE).expect("could not open file"))
                .expect("failed to parse result"),
        )
    }
}

/// build and run the challenge
pub fn run(opts: RunOptions) {
    let bin_name = opts.challange.get_name();
    let bin_path = build(opts.release, &bin_name);
    match opts.challange {
        Challange::Echo => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                1,
                10,
                &opts.extra_args,
            )
            .execute();
        }
        Challange::UniqueIds => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                3,
                30,
                &opts.extra_args,
            )
            .partition()
            .rate(1000)
            .total_availability()
            .execute();
        }
        Challange::SingleBroadcast => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                1,
                20,
                &opts.extra_args,
            )
            .rate(10)
            .execute();
        }
        Challange::MultiBroadcast => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                5,
                20,
                &opts.extra_args,
            )
            .rate(10)
            .execute();
        }
        Challange::FaultyBroadcast => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                5,
                20,
                &opts.extra_args,
            )
            .rate(10)
            .partition()
            .execute();
        }
        Challange::EfficientBroadcast => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                25,
                20,
                &opts.extra_args,
            )
            .rate(100)
            .latency(100)
            .topology("tree4")
            .execute();
            let result = MaelStromCommand::get_results();
            let message_per_op = result.get_value_at(&[
                edn_format::Keyword::from_name("net").into(),
                edn_format::Keyword::from_name("servers").into(),
                edn_format::Keyword::from_name("msgs-per-op").into(),
            ]);
            assert!(
                message_per_op.expect("failed to get message per ops")
                    < &edn_format::Value::Float(30.0.into())
            );
            let median_latency = result.get_value_at(&[
                edn_format::Keyword::from_name("workload").into(),
                edn_format::Keyword::from_name("stable-latencies").into(),
                0.5.into(),
            ]);
            assert!(
                median_latency.expect("failed to get median latency")
                    < &edn_format::Value::Integer(400)
            );
            let maximum_latency = result.get_value_at(&[
                edn_format::Keyword::from_name("workload").into(),
                edn_format::Keyword::from_name("stable-latencies").into(),
                1.into(),
            ]);
            assert!(
                maximum_latency.expect("failed to get maximum latency")
                    < &edn_format::Value::Integer(600)
            );
        }
        Challange::EfficientBroadcast2 => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                25,
                20,
                &opts.extra_args,
            )
            .env("FORCE_TICK", "false")
            .rate(100)
            .latency(100)
            .execute();
            let result = MaelStromCommand::get_results();
            let message_per_op = result.get_value_at(&[
                edn_format::Keyword::from_name("net").into(),
                edn_format::Keyword::from_name("servers").into(),
                edn_format::Keyword::from_name("msgs-per-op").into(),
            ]);
            assert!(
                message_per_op.expect("failed to get message per ops")
                    < &edn_format::Value::Float(20.0.into())
            );
            let median_latency = result.get_value_at(&[
                edn_format::Keyword::from_name("workload").into(),
                edn_format::Keyword::from_name("stable-latencies").into(),
                0.5.into(),
            ]);
            assert!(
                median_latency.expect("failed to get median latency")
                    < &edn_format::Value::Integer(1000)
            );
            let maximum_latency = result.get_value_at(&[
                edn_format::Keyword::from_name("workload").into(),
                edn_format::Keyword::from_name("stable-latencies").into(),
                1.into(),
            ]);
            assert!(
                maximum_latency.expect("failed to get maximum latency")
                    < &edn_format::Value::Integer(2000)
            );
        }
        Challange::GrowOnlyCounter => {
            MaelStromCommand::new(
                &opts.maelstrom_bin,
                &bin_path,
                &bin_name,
                3,
                20,
                &opts.extra_args,
            )
            .rate(100)
            .partition()
            .execute();
        }
    }
}

/// list challenges.
pub fn list() {
    print!(
        "{}",
        Challange::value_variants()
            .iter()
            .map(|var| format!("{}\n", var.to_possible_value().unwrap().get_name()))
            .collect::<String>()
    );
}
