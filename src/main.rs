use clap::Parser;
use cli::Cli;

mod cli;
mod common;
mod exporter;
mod parser;
mod plotter;
mod printer;
mod runner;
mod solvers;

fn main() {
    let args = Cli::parse();

    if let Err(e) = runner::run(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
