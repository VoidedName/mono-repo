use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

/// Updates the root workspace Cargo.toml to include the newly generated crates.
pub fn update_workspace(name: &str, terminal: bool, native: bool, web: bool) -> Result<()> {
    let workspace_toml_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(workspace_toml_path).context("Failed to read Cargo.toml")?;
    let mut doc = content.parse::<DocumentMut>()?;

    let members = doc["workspace"]["members"]
        .as_array_mut()
        .context("Missing workspace.members array")?;

    let mut new_members = vec![
        format!("{}/{}-core", name, name),
    ];

    if web {
        new_members.push(format!("{}/{}-web", name, name));
    }
    if terminal {
        new_members.push(format!("{}/{}-terminal", name, name));
    }
    if native {
        new_members.push(format!("{}/{}-native", name, name));
    }

    for member in new_members {
        if !members.iter().any(|m| m.as_str() == Some(&member)) {
            members.push(member);
        }
    }

    fs::write(workspace_toml_path, doc.to_string())?;
    Ok(())
}

/// Creates a reasonable .gitignore file for the new project.
pub fn create_gitignore(root: &Path, has_web: bool) -> Result<()> {
    let mut content = "target/\n".to_string();
    if has_web {
        content.push_str("pkg/\n");
        content.push_str("node_modules/\n");
        content.push_str("**/dist/\n");
        content.push_str("wasm-pack.log\n");
    }
    fs::write(root.join(".gitignore"), content)?;
    Ok(())
}
