mod cli;
mod clipboard;
mod commands;
mod config;
mod generator;
mod printing;
mod prompts;
mod selectors;
mod vault;
mod write_flow;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    commands::run(Cli::parse())
}
