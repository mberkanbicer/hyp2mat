//! CLI entry point — port of `src/hyp2mat.cc`.

use std::fs;

use clap::Parser;

use crate::parser::parse_hyp;
use crate::pcb_writer::write_pcb;

/// HyperLynx .hyp → CSXCAD/PDF/QucsStudio converter.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input .hyp file
    #[arg(short, long)]
    input: String,

    /// Output file
    #[arg(short, long)]
    output: String,

    /// Output format: pdf, csxcad, qucs
    #[arg(short, long, default_value = "csxcad")]
    format: String,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    if cli.verbose > 0 {
        eprintln!("Input: {}", cli.input);
        eprintln!("Output: {}", cli.output);
        eprintln!("Format: {}", cli.format);
    }

    // Read input file
    let input = fs::read_to_string(&cli.input)
        .map_err(|e| format!("Cannot read input file '{}': {}", cli.input, e))?;

    // Parse the .hyp file
    let pcb = parse_hyp(&input).map_err(|e| format!("Parse error: {}", e))?;

    if cli.verbose > 0 {
        eprintln!(
            "Parsed PCB: {} layers, {} vias, {} devices, {} pins",
            pcb.stackup.len(),
            pcb.via.len(),
            pcb.device.len(),
            pcb.pin.len()
        );
    }

    // Write output
    write_pcb(&pcb, &cli.format, &cli.output).map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
