use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Light Stripe — ports, processes, and Docker for local dev", name = "light-stripe")]
pub struct Cli {
    /// Path to config.toml (default: platform config dir)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Interactive dashboard (TUI)
    #[command(alias = "ui")]
    Tui,

    /// Listening ports
    #[command(alias = "p", visible_alias = "port")]
    Ports(PortsArgs),

    /// Running processes
    #[command(alias = "proc")]
    Ps(PsArgs),

    /// System stats
    #[command(aliases = ["st", "sys"])]
    Stats(StatsArgs),

    /// Show config path and effective settings
    Config,
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
    pub dev_only: bool,

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
