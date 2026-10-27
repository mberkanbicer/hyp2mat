//! PCB output writer dispatcher — routes to the appropriate format writer.

use std::io::Write;

use crate::csxcad::{write_csxcad, write_csxcad_xml, CsxcadOptions};
use crate::model::PCB;
use crate::pdf::{write_pdf, PdfOptions};
use crate::qucsstudio::write_qucsstudio;

/// Options that influence the rendered output.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OutputOptions {
    pub csxcad: CsxcadOptions,
    /// Emit CSXCAD XML instead of the native MATLAB script.
    pub csxcad_xml: bool,
    pub pdf: PdfOptions,
}

/// Rendered output plus the messages the native writer prints to stderr.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    pub data: Vec<u8>,
    pub warnings: Vec<String>,
}

/// Render a PCB in the requested format.
///
/// Formats are the native `--output-format` values: `csxcad`, `pdf`, `qucs`
/// (with `qucsstudio` accepted as an alias).
pub fn render(pcb: &PCB, format: &str, options: &OutputOptions) -> Result<Rendered, String> {
    match format {
        "csxcad" => {
            if options.csxcad_xml {
                Ok(Rendered {
                    data: write_csxcad_xml(pcb, &options.csxcad)?.into_bytes(),
                    warnings: Vec::new(),
                })
            } else {
                Ok(Rendered {
                    data: write_csxcad(pcb, &options.csxcad)?.into_bytes(),
                    warnings: Vec::new(),
                })
            }
        }
        "pdf" => Ok(Rendered {
            data: write_pdf(pcb, &options.pdf)?,
            warnings: Vec::new(),
        }),
        "qucs" | "qucsstudio" => {
            let (text, warnings) = write_qucsstudio(pcb)?;
            Ok(Rendered {
                data: text.into_bytes(),
                warnings,
            })
        }
        other => Err(format!("unknown output format {other}")),
    }
}

/// Render a PCB and write it to `output` (`-` means standard output).
///
/// Returns the writer's stderr messages.
pub fn write_pcb(
    pcb: &PCB,
    format: &str,
    output: &str,
    options: &OutputOptions,
) -> Result<Vec<String>, String> {
    let rendered = render(pcb, format, options)?;
    if output == "-" {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(&rendered.data)
            .and_then(|()| handle.flush())
            .map_err(|e| format!("cannot write to standard output: {e}"))?;
    } else {
        std::fs::write(output, &rendered.data)
            .map_err(|e| format!("cannot open '{output}' for writing: {e}"))?;
    }
    Ok(rendered.warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_format_is_rejected() {
        let error = render(&PCB::default(), "gbr", &OutputOptions::default()).unwrap_err();
        assert_eq!(error, "unknown output format gbr");
    }

    #[test]
    fn csxcad_xml_switch_selects_xml() {
        let pcb = PCB::default();
        let script = render(&pcb, "csxcad", &OutputOptions::default()).unwrap();
        assert!(script.data.starts_with(b"function CSX = pcb(CSX)"));
        let xml = render(
            &pcb,
            "csxcad",
            &OutputOptions {
                csxcad_xml: true,
                ..OutputOptions::default()
            },
        )
        .unwrap();
        assert!(xml.data.starts_with(b"<?xml"));
    }

    #[test]
    fn qucsstudio_alias_matches_qucs() {
        let pcb = PCB::default();
        let a = render(&pcb, "qucs", &OutputOptions::default()).unwrap();
        let b = render(&pcb, "qucsstudio", &OutputOptions::default()).unwrap();
        assert_eq!(a, b);
    }
}
