mod cli;
mod collectors;
mod model;
mod output;
use anyhow::{Ok, Result};
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Ports(args) => {
            let bindings = collectors::ports::collect(args.port)?;

            for binding in bindings {
                println!(
                    "port={} proto={:?} addr={} pid={:?} name={:?}",
                    binding.port,
                    binding.protocol,
                    binding.address,
                    binding.pid,
                    binding.process_name,
                );
            }
        }

        Command::Ps(_) => {
            println!("In progress");
        }

        Command::Stats(_) => {
            println!("In progress")
        }
    }

    Ok(())
}
