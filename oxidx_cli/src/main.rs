//! # OxidX CLI
//!
//! Command-line interface for the OxidX UI framework toolchain.
//!
//! ## Commands
//!
//! - `oxidx generate -i input.json -o output.rs` - Generate Rust code from JSON schema
//! - `oxidx schema` - Print JSON Schema to stdout for IDE IntelliSense
//! - `oxidx watch -i input.json` - Watch mode with auto-regeneration

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use oxidx_codegen::{
    generate_json_schema, CodeGenerator, ComponentNode, RustGenerator, WindowSchema,
};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(
    name = "oxidx",
    author,
    version,
    about = "OxidX UI Framework CLI",
    long_about = "A modern CLI for the OxidX declarative UI framework.\nGenerate Rust code from JSON layouts with hot-reload support."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate Rust code from a JSON schema file
    Generate {
        /// Input JSON schema file
        #[arg(short, long)]
        input: PathBuf,

        /// Output Rust file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Print JSON Schema for OxidX layout files (for IDE IntelliSense)
    Schema,

    /// Watch a JSON file and regenerate on changes
    Watch {
        /// Input JSON schema file to watch
        #[arg(short, long)]
        input: PathBuf,

        /// Output Rust file (optional, defaults to input with .rs extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { input, output } => cmd_generate(&input, &output),
        Commands::Schema => cmd_schema(),
        Commands::Watch { input, output } => cmd_watch(&input, output),
    }
}

/// Generate Rust code from JSON schema.
fn cmd_generate(input: &PathBuf, output: &PathBuf) -> Result<()> {
    println!("🚀 OxidX Code Generator");
    println!("   Input: {:?}", input);
    println!("   Output: {:?}", output);

    let code = generate_code(input)?;

    fs::write(output, code)
        .with_context(|| format!("Failed to write output file: {:?}", output))?;

    println!("✅ Generated successfully!");
    Ok(())
}

/// Print JSON Schema to stdout.
fn cmd_schema() -> Result<()> {
    let schema = generate_json_schema();
    println!("{}", schema);
    Ok(())
}

/// Watch mode: monitor file changes and regenerate.
fn cmd_watch(input: &PathBuf, output: Option<PathBuf>) -> Result<()> {
    let output = output.unwrap_or_else(|| input.with_extension("rs"));

    println!("👀 OxidX Watch Mode");
    println!("   Watching: {:?}", input);
    println!("   Output: {:?}", output);
    println!("   Press Ctrl+C to exit\n");

    // Initial build
    do_build(input, &output);

    // Setup file watcher with 500ms debounce
    let (tx, rx) = channel();
    let mut debouncer =
        new_debouncer(Duration::from_millis(500), tx).context("Failed to create file watcher")?;

    debouncer
        .watcher()
        .watch(input, RecursiveMode::NonRecursive)
        .with_context(|| format!("Failed to watch file: {:?}", input))?;

    // Event loop - robust error handling, never crash
    loop {
        match rx.recv() {
            Ok(Ok(_events)) => {
                do_build(input, &output);
            }
            Ok(Err(errors)) => {
                eprintln!("❌ Watch error: {:?}", errors);
            }
            Err(e) => {
                eprintln!("❌ Channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Perform a build with error recovery.
fn do_build(input: &PathBuf, output: &PathBuf) {
    clear_terminal();

    let now = Local::now().format("%H:%M:%S");

    match generate_code(input) {
        Ok(code) => {
            if let Err(e) = fs::write(output, code) {
                println!("❌ Write Error at [{}]", now);
                println!("   {}", e);
            } else {
                println!("✅ Build Successful at [{}]", now);
                println!("   Output: {:?}", output);
            }
        }
        Err(e) => {
            println!("❌ Build Failed at [{}]", now);
            println!("   {}", e);
            // Don't crash - wait for next save
        }
    }
}

/// Generate code from a JSON file.
fn generate_code(input: &PathBuf) -> Result<String> {
    let json_content =
        fs::read_to_string(input).with_context(|| format!("Failed to read: {:?}", input))?;

    let root: ComponentNode =
        serde_json::from_str(&json_content).context("Invalid JSON component")?;

    let name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("App")
        .to_string();

    let schema = WindowSchema { name, root };

    let generator = RustGenerator;
    // generator.generate returns Result<String>
    generator.generate(&schema)
}

/// Clear terminal screen.
fn clear_terminal() {
    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::Write::flush(&mut io::stdout());
}
