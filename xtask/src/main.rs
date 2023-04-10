use clap::Parser;

mod challange;
mod serve;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Run(challange::Options),
    Serve(serve::Options),
    List,
}

fn main() {
    let opts = Cli::parse();
    match opts.command {
        Command::Run(options) => challange::run(options),
        Command::Serve(options) => serve::serve(options),
        Command::List => challange::list(),
    }
}
