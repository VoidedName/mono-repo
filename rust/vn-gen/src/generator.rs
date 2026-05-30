use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use crate::cli::ProjectConfig;
use crate::templates;
use crate::utils;

/// Orchestrates the project generation process.
pub fn generate_project(config: &ProjectConfig) -> Result<()> {
    let name = &config.name;
    let root = Path::new(name);
    
    if root.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&root).context(format!("Failed to create project root at {:?}", root))?;

    println!("Generating project '{}' in {:?}...", name, root);

    // 1. Core Crate
    templates::core::create(&root, name)?;

    // 2. Web Crate
    if config.web {
        templates::web::create(&root, name)?;
    }

    // 3. UI/Native Crates
    if config.terminal {
        templates::terminal::create(&root, name)?;
    }
    if config.native {
        templates::native::create(&root, name)?;
    }

    // 4. Gitignore
    utils::create_gitignore(&root, config.web)?;

    // 5. Update Workspace
    utils::update_workspace(name, config.terminal, config.native, config.web)?;

    println!("\nProject '{}' generated successfully!", name);
    println!("Next steps:");
    println!("  - Review rust/{}/Cargo.toml", name);
    println!("  - Start coding in rust/{}/{}-core", name, name);

    Ok(())
}
