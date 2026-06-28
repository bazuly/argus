mod cli;
mod collectors;
mod models;
mod output;
mod tui;

use anyhow::{Ok, Result};
use clap::Parser;
use cli::{Cli, Command, OutputFormat};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Tui => {
            tui::run()?;
        }

        Command::Ports(args) => {
            let bindings = collectors::ports::collect(args.port)?;

            match args.format {
                OutputFormat::Table => output::table::print_ports(&bindings),
                OutputFormat::Json => output::json::print_ports(&bindings)?,
            }
        }

        Command::Ps(args) => {
            let processes = collectors::processes::collect(args.dev_only)?;

            match args.format {
                OutputFormat::Table => output::table::print_processes(&processes),
                OutputFormat::Json => output::json::print_processes(&processes)?,
            }
        }
        Command::Stats(args) => {
            let stats = collectors::system::collect()?;
            match args.format {
                OutputFormat::Table => output::table::print_stats(&stats),
                OutputFormat::Json => output::json::print_stats(&stats)?,
            }
        }
    }

    Ok(())
}
