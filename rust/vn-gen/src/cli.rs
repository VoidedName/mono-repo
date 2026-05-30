use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};

/// The main entry point for the cargo-vn subcommand.
#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    Vn(Args),
}

/// Arguments for the 'vn' command.
#[derive(Parser, Debug)]
#[command(author, version, about = "Scaffold a new VN project", long_about = None)]
pub struct Args {
    /// The name of the project.
    #[arg(short, long)]
    pub name: Option<String>,

    /// Generate a terminal (Ratatui) target.
    #[arg(long)]
    pub terminal: bool,

    /// Generate a native (WGPU) target.
    #[arg(long)]
    pub native: bool,

    /// Generate a web (Wasm) target.
    #[arg(long)]
    pub web: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands for 'cargo vn'.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new project.
    New {
        /// The name of the project.
        name: Option<String>,
        /// Generate a terminal target.
        #[arg(long)]
        terminal: bool,
        /// Generate a native target.
        #[arg(long)]
        native: bool,
        /// Generate a web target.
        #[arg(long)]
        web: bool,
    },
}

/// Represents the different types of application targets that can be generated.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TargetType {
    Terminal,
    Native,
    Web,
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::Terminal => write!(f, "Terminal (Ratatui)"),
            TargetType::Native => write!(f, "Native (WGPU)"),
            TargetType::Web => write!(f, "Web (Wasm)"),
        }
    }
}

/// Configuration for project generation parsed from CLI or interactive prompts.
pub struct ProjectConfig {
    pub name: String,
    pub terminal: bool,
    pub native: bool,
    pub web: bool,
}

impl ProjectConfig {
    /// Parses CLI arguments and handles interactive prompts if necessary.
    pub fn from_args(args: Args) -> Result<Self> {
        let (name, mut terminal, mut native, mut web) = match args.command {
            Some(Commands::New { name, terminal, native, web }) => (name, terminal, native, web),
            None => (args.name, args.terminal, args.native, args.web),
        };

        let project_name = if let Some(n) = name {
            n
        } else {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Project name (e.g., vn-clock)")
                .interact_text()?
        };

        if !terminal && !native && !web {
            let options = vec![TargetType::Terminal, TargetType::Native, TargetType::Web];
            let defaults = vec![true, false, true];
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select targets to generate (Space to toggle, Enter to confirm)")
                .items(&options)
                .defaults(&defaults)
                .interact()?;
            
            for selection in selections {
                match options[selection] {
                    TargetType::Terminal => terminal = true,
                    TargetType::Native => native = true,
                    TargetType::Web => web = true,
                }
            }
        }

        Ok(ProjectConfig {
            name: project_name,
            terminal,
            native,
            web,
        })
    }
}
