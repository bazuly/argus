use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, name = "argus")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Ports(PortsArgs), // listening port and owning processes
    Ps(PsArgs),       // Running Process (dev-only)
    Stats(StatsArgs), // System cpu/ram snapshot
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct PortsArgs {
    #[arg(long, short)]
    pub port: Option<u16>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct PsArgs {
    #[arg(long, short)]
    pub dev_only: bool, // show only dev processes

    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct StatsArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}
