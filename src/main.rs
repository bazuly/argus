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

        Command::Ps(args) => {
            let processes = collectors::processes::collect(args.dev_only)?;
            for process in processes {
                println!(
                    "pid={} dev={} cpu={:.1}% mem={} name={:?} cmd={:?}",
                    process.pid,
                    process.is_dev,
                    process.cpu_usage,
                    process.memory_bytes,
                    process.name,
                    process.cmdline,
                );
            }
        }

        Command::Stats(_args) => {
            let stats = collectors::system::collect()?;
            println!(
                "ram_used_bytes={} ram_total_bytes={} cpu_global={:.1}%",
                stats.used_memory, stats.total_memory, stats.global_cpu_usage,
            );
        }
    }

    Ok(())
}
