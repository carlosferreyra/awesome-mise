#!/usr/bin/env -S cargo +nightly -Zscript
---
[package]
name = "checks"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
url = "2"

[dev-dependencies]
tempfile = "3"
---
//! Validate catalog data and recipe TOML without installing or executing resources.
mod catalog;

fn run() -> catalog::Result<()> {
    for arg in std::env::args().skip(1).filter(|a| a != "--") {
        if arg == "--help" || arg == "-h" {
            println!("usage: checks");
            return Ok(());
        }
        return Err(format!("unknown argument: {arg}").into());
    }
    catalog::load(&catalog::repo_root()?)?;
    println!("Catalog and recipe TOML are valid (no runtime or compatibility claims).");
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
