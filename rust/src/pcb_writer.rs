//! PCB output writer dispatcher — routes to the appropriate format writer.

use crate::model::PCB;
use crate::csxcad::write_csxcad;
use crate::qucsstudio::write_qucsstudio;
use crate::pdf::write_pdf;

/// Write a PCB in the specified format to the given output path.
pub fn write_pcb(pcb: &PCB, format: &str, output: &str) -> Result<(), String> {
    let content = match format {
        "csxcad" => write_csxcad(pcb)?,
        "qucs" | "qucsstudio" => write_qucsstudio(pcb)?,
        "pdf" => write_pdf(pcb)?,
        _ => return Err(format!("unknown output format: {}", format)),
    };

    std::fs::write(output, &content)
        .map_err(|e| format!("Cannot write output file '{}': {}", output, e))?;

    Ok(())
}