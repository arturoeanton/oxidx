//! # OxidX Viewer
//!
//! Runs OxidX UI layouts directly from JSON files.
//!
//! ## Usage
//!
//! ```bash
//! # From file
//! oxidx-viewer path/to/layout.json
//!
//! # From stdin
//! cat layout.json | oxidx-viewer
//! ```

use anyhow::{Context, Result};
use oxidx_core::schema::ComponentNode;
use oxidx_std::dynamic::DynamicRoot;
use oxidx_std::run;
use std::io::{self, Read};
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let json_content = if args.len() > 1 {
        // Read from file
        let path = &args[1];
        eprintln!("[oxidx-viewer] Loading: {}", path);
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?
    } else {
        // Read from stdin
        eprintln!("[oxidx-viewer] Reading from stdin...");
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read from stdin")?;
        content
    };

    // Deserialize JSON to ComponentNode
    let schema: ComponentNode =
        serde_json::from_str(&json_content).context("Failed to parse JSON as ComponentNode")?;

    eprintln!("[oxidx-viewer] Building UI tree...");
    eprintln!("[oxidx-viewer] Root component: {}", schema.type_name);

    // Build the component tree dynamically using DynamicRoot wrapper
    let root = DynamicRoot::from_schema(&schema);

    eprintln!("[oxidx-viewer] Starting application...");

    // Run the application
    run(root);

    Ok(())
}
