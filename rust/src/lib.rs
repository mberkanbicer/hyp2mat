//! hyp2mat-rs: HyperLynx PCB parsing and CSXCAD/PDF/QucsStudio conversion
//! workflows.
//!
//! Port of the hyp2mat C++ autotools project to Rust.
//! See `hyp2mat/lib/hyp2mat.h` for the original PCB model and
//! `hyp2mat/lib/hypfile.h` for the parser model.

pub mod cli;
pub mod csxcad;
pub mod format;
pub mod hypfile;
pub mod model;
pub mod parser;
pub mod pcb;
pub mod pcb_writer;
pub mod pdf;
pub mod polygon;
pub mod qucsstudio;
