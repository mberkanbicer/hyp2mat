//! PDF output writer — port of `hyp2mat/lib/pdf.cc` (libhpdf).
//! Stubbed out since libhpdf is a C dependency.

use crate::model::PCB;

/// Write a PCB as a PDF.
pub fn write_pdf(_pcb: &PCB) -> Result<String, String> {
    // TODO: implement PDF writer (or stub)
    Err("pdf writer not yet implemented".to_string())
}
