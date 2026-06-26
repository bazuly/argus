mod cli;
mod collectors;
mod model;
mod output;
use anyhow::{Ok, Result};
use clap::Parser;
use cli::{Cli, Command, OutputFormat};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Ports(args) => {
            let bindings = collectors::ports::collect(args.port)?;

            match args.format {
                OutputFormat::Table => output::table::print_ports(&bindings),
                OutputFormat::Json => {
                    println!("JSON output is not implemented yet. Use --format table.");
                }
            }
        }

        Command::Ps(args) => {
            let processes = collectors::processes::collect(args.dev_only)?;

            match args.format {
                OutputFormat::Table => output::table::print_processes(&processes),
                OutputFormat::Json => {
                    println!("JSON output is not implemented yet. Use --format table.");
                }
            }
        }
        Command::Stats(args) => {
            let stats = collectors::system::collect()?;
            match args.format {
                OutputFormat::Table => output::table::print_stats(&stats),
                OutputFormat::Json => {
                    println!("JSON output is not implemented yet. Use --format table.");
                }
            }
        }
    }

    Ok(())
}
