//! Binary entry point for hyp2mat-rs.

use hyp2mat_rs::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}