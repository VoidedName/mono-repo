use anyhow::Result;
use clap::Parser;

mod cli;
mod generator;
mod templates;
mod utils;

use cli::{CargoCli, ProjectConfig};

/// The main entry point for the `cargo-vn` tool.
///
/// It handles CLI argument parsing, interactive configuration,
/// and delegates project generation to the `generator` module.
fn main() -> Result<()> {
    let CargoCli::Vn(args) = CargoCli::parse();

    let config = ProjectConfig::from_args(args)?;

    generator::generate_project(&config)?;

    Ok(())
}
