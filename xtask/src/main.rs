use clap::Parser;

mod run;
mod serve;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Run(run::Options),
    Serve(serve::Options),
}

fn main() {
    let opts = Cli::parse();
    match opts.command {
        Command::Run(options) => run::run(options),
        Command::Serve(options) => serve::serve(options),
    }
}
