//! hyp2mat-rs: HyperLynx PCB parsing and CSXCAD/PDF/QucsStudio conversion workflows.
//!
//! Port of the hyp2mat C++ autotools project to Rust.
//! See `hyp2mat/lib/hyp2mat.h` for the original PCB model.

pub mod model;
pub mod parser;
pub mod polygon;
pub mod csxcad;
pub mod qucsstudio;
pub mod pdf;
pub mod pcb_writer;
pub mod cli;
