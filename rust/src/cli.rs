//! CLI entry point — port of `hyp2mat/src/hyp2mat.cc` and
//! `hyp2mat/src/cmdline.ggo`.
//!
//! Option names, short options, defaults and exit codes follow the native
//! gengetopt interface:
//!
//! * no input file reads standard input, `-o -` writes standard output,
//! * `-f pdf` requires an explicit `-o` (the native binary prints
//!   `pdf output file missing` otherwise),
//! * syntax errors are reported and conversion continues (exit status 0),
//! * a missing input file exits with status 1 and
//!   `Can't open input file <name>`.
//!
//! `--csxcad-xml` is an extension of this port: it writes CSXCAD XML instead
//! of the native MATLAB script, which remains the default for `-f csxcad`.

use std::io::Read;

use clap::Parser;

use crate::csxcad::CsxcadOptions;
use crate::format::format_cpp_float;
use crate::hypfile::GeomSettings;
use crate::model::Bounds;
use crate::parser::parse_hypfile;
use crate::pcb::{print_nets, print_summary, read_hyp, Options};
use crate::pcb_writer::{write_pcb, OutputOptions};
use crate::pdf::PdfOptions;

/// Version string printed by `--version` (native: `hyp2mat 0.0.18`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// HyperLynx .hyp → CSXCAD/PDF/QucsStudio converter.
#[derive(Parser, Debug)]
#[command(
    name = "hyp2mat",
    bin_name = "hyp2mat",
    version = VERSION,
    about = "Converts Hyperlynx Signal-Integrity Transfer Format files to Octave/matlab scripts.",
    after_help = "All lengths are in meters."
)]
pub struct Cli {
    /// Output file.  (default=`-')
    #[arg(short = 'o', long = "output", value_name = "filename")]
    pub output: Option<String>,

    /// Output file format.
    #[arg(
        short = 'f',
        long = "output-format",
        value_name = "ENUM",
        default_value = "pdf",
        value_parser = ["csxcad", "pdf", "qucs"]
    )]
    pub output_format: String,

    /// Import net. Repeat to import several nets. Default is importing all
    /// nets.
    #[arg(short = 'n', long = "net", value_name = "STRING")]
    pub net: Vec<String>,

    /// Import layer. Repeat to import several layers. Default is importing all
    /// layers.
    #[arg(short = 'l', long = "layer", value_name = "STRING")]
    pub layer: Vec<String>,

    /// Crop pcb. Set lower bound of x coordinate.
    #[arg(
        short = 'x',
        long = "xmin",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub xmin: Option<f64>,

    /// Crop pcb. Set upper bound of x coordinate.
    #[arg(
        short = 'X',
        long = "xmax",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub xmax: Option<f64>,

    /// Crop pcb. Set lower bound of y coordinate.
    #[arg(
        short = 'y',
        long = "ymin",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub ymin: Option<f64>,

    /// Crop pcb. Set upper bound of y coordinate.
    #[arg(
        short = 'Y',
        long = "ymax",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub ymax: Option<f64>,

    /// Crop pcb. Set lower bound of z coordinate.
    #[arg(
        short = 'z',
        long = "zmin",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub zmin: Option<f64>,

    /// Crop pcb. Set upper bound of z coordinate.
    #[arg(
        short = 'Z',
        long = "zmax",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub zmax: Option<f64>,

    /// Set output grid size.
    #[arg(
        long = "grid",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "10e-6"
    )]
    pub grid: f64,

    /// Set maximum difference between perfect arc and polygonal approximation.
    #[arg(
        long = "arc-precision",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "0"
    )]
    pub arc_precision: f64,

    /// Set default trace-to-plane clearance.
    #[arg(
        long = "clearance",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "0.0002"
    )]
    pub clearance: f64,

    /// Flood layer with copper. Repeat to flood several layers. The value
    /// "plane_layers" floods all plane layers.
    #[arg(long = "flood", value_name = "STRING")]
    pub flood: Vec<String>,

    /// Set epsilon r of dielectric.
    #[arg(long = "epsilonr", value_name = "DOUBLE", allow_hyphen_values = true)]
    pub epsilonr: Option<f64>,

    /// Set loss tangent of dielectric.
    #[arg(
        long = "loss-tangent",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub loss_tangent: Option<f64>,

    /// Set bulk resistivity of copper (in Ohm meter).
    #[arg(
        long = "bulk-resistivity",
        value_name = "DOUBLE",
        allow_hyphen_values = true
    )]
    pub bulk_resistivity: Option<f64>,

    /// Detailed board outline.
    #[arg(long = "pcb-outline")]
    pub pcb_outline: bool,

    /// Model copper as lossy. Default is modeling copper as perfect conductor.
    #[arg(long = "lossy-copper")]
    pub lossy_copper: bool,

    /// Model copper layers as 3D. Default is modeling metal as 2D sheet.
    #[arg(long = "metal-3d")]
    pub metal_3d: bool,

    /// Set PDF color hue. Range 0.0 to 1.0
    #[arg(
        long = "hue",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "0.0"
    )]
    pub hue: f64,

    /// Set PDF color saturation. Range 0.0 to 1.0
    #[arg(
        long = "saturation",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "0.6"
    )]
    pub saturation: f64,

    /// Set PDF color brightness. Range 0.0 to 1.0
    #[arg(
        long = "brightness",
        value_name = "DOUBLE",
        allow_hyphen_values = true,
        default_value = "0.9"
    )]
    pub brightness: f64,

    /// Raw output. Do not join adjacent or overlapping copper.
    #[arg(short = 'r', long = "raw")]
    pub raw: bool,

    /// Increase debugging level. Repeat for more detailed debugging.
    #[arg(short = 'd', long = "debug", action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Print board summary.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Write CSXCAD XML instead of the native MATLAB script (extension).
    #[arg(long = "csxcad-xml")]
    pub csxcad_xml: bool,

    /// Input .hyp file (default: standard input).
    #[arg(value_name = "infile")]
    pub infile: Vec<String>,
}

fn read_input(name: &str) -> Result<String, ()> {
    if name == "-" {
        let mut buffer = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buffer)
            .map_err(|e| eprintln!("Can't read standard input: {e}"))?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    } else {
        match std::fs::read(name) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => {
                eprintln!("Can't open input file {}", name);
                Err(())
            }
        }
    }
}

/// Build conversion options from the parsed command line.
pub fn conversion_options(cli: &Cli) -> Options {
    let grid = if cli.grid > 0.0 { cli.grid } else { 1e-5 };
    let arc_precision = if cli.arc_precision >= 0.0 {
        cli.arc_precision
    } else {
        0.0
    };
    let clearance = if cli.clearance >= 0.0 {
        cli.clearance
    } else {
        -1.0
    };

    let any_bounds = cli.xmin.is_some()
        || cli.xmax.is_some()
        || cli.ymin.is_some()
        || cli.ymax.is_some()
        || cli.zmin.is_some()
        || cli.zmax.is_some();
    let bounds = any_bounds.then(|| Bounds {
        x0: cli.xmin.unwrap_or(-f64::MAX),
        x1: cli.xmax.unwrap_or(f64::MAX),
        y0: cli.ymin.unwrap_or(-f64::MAX),
        y1: cli.ymax.unwrap_or(f64::MAX),
        z0: cli.zmin.unwrap_or(-f64::MAX),
        z1: cli.zmax.unwrap_or(f64::MAX),
    });

    Options {
        layers: cli.layer.clone(),
        nets: cli.net.clone(),
        raw: cli.raw,
        grid,
        arc_precision,
        clearance,
        flood_layers: cli.flood.clone(),
        epsilon_r: cli.epsilonr,
        loss_tangent: cli.loss_tangent,
        bulk_resistivity: cli.bulk_resistivity,
        bounds,
    }
}

/// Build writer options from the parsed command line.
pub fn output_options(cli: &Cli) -> OutputOptions {
    OutputOptions {
        csxcad: CsxcadOptions {
            pcb_outline: cli.pcb_outline,
            lossy_copper: cli.lossy_copper,
            metal_3d: cli.metal_3d,
        },
        csxcad_xml: cli.csxcad_xml,
        pdf: PdfOptions {
            hue: cli.hue,
            saturation: cli.saturation,
            brightness: cli.brightness,
        },
    }
}

/// Run the converter and return the process exit code.
pub fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 1,
            };
        }
    };

    if cli.infile.len() > 1 {
        eprintln!("more than one input file given");
        return 1;
    }
    let input_name = cli.infile.first().map(String::as_str).unwrap_or("-");
    let input = match read_input(input_name) {
        Ok(input) => input,
        Err(()) => return 1,
    };

    let geometry = GeomSettings {
        grid: if cli.grid > 0.0 { cli.grid } else { 1e-5 },
        arc_precision: if cli.arc_precision >= 0.0 {
            cli.arc_precision
        } else {
            0.0
        },
    };
    let (hyp, diagnostics) = parse_hypfile(&input, geometry);
    for diagnostic in &diagnostics {
        eprintln!("{diagnostic}");
    }

    if cli.debug > 0 {
        eprintln!("hyp2mat {} using Clipper2 (clipper2 crate)", VERSION);
        let mut dumped = hyp.clone();
        if dumped.ended {
            if let Err(error) = dumped.flood_layers_with_copper(&cli.flood) {
                eprintln!("{error}");
            }
        }
        if let Err(error) = std::fs::write("hyp_debug.txt", dumped.dump()) {
            eprintln!("could not open 'hyp_debug.txt' for writing: {error}");
        }
    }

    if cli.net.iter().any(|net| net == "?") {
        print!("{}", print_nets(&hyp));
    }

    let options = conversion_options(&cli);
    let (pcb, warnings) = match read_hyp(&hyp, &options) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    for warning in &warnings {
        eprintln!("{warning}");
    }

    if cli.verbose {
        eprint!("{}", print_summary(&pcb));
    }

    if cli.output_format == "pdf" && cli.output.is_none() {
        eprintln!("pdf output file missing ");
        return 0;
    }
    let output = cli.output.as_deref().unwrap_or("-");

    match write_pcb(&pcb, &cli.output_format, output, &output_options(&cli)) {
        Ok(warnings) => {
            for warning in warnings {
                eprintln!("{warning}");
            }
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

/// Native `hyp2mat --version` text (without a trailing newline).
pub fn version_string() -> String {
    format!("hyp2mat {}", VERSION)
}

/// Format a value the way the native PDF/QucsStudio writers do.
pub fn format_float(value: f64) -> String {
    format_cpp_float(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).unwrap()
    }

    #[test]
    fn defaults_match_native() {
        let cli = parse(&["hyp2mat"]);
        assert_eq!(cli.output, None);
        assert_eq!(cli.output_format, "pdf");
        assert_eq!(cli.grid, 10e-6);
        assert_eq!(cli.arc_precision, 0.0);
        assert_eq!(cli.clearance, 0.0002);
        assert_eq!(cli.hue, 0.0);
        assert_eq!(cli.saturation, 0.6);
        assert_eq!(cli.brightness, 0.9);
        assert!(cli.infile.is_empty());
    }

    #[test]
    fn short_options_match_native() {
        let cli = parse(&[
            "hyp2mat",
            "-o",
            "pcb.m",
            "-f",
            "csxcad",
            "-n",
            "GND",
            "-n",
            "VCC",
            "-l",
            "Top",
            "-x",
            "0.1",
            "-X",
            "0.2",
            "-y",
            "0.3",
            "-Y",
            "0.4",
            "-z",
            "0.0",
            "-Z",
            "0.001",
            "-r",
            "-v",
            "-d",
            "-d",
            "board.hyp",
        ]);
        assert_eq!(cli.output.as_deref(), Some("pcb.m"));
        assert_eq!(cli.output_format, "csxcad");
        assert_eq!(cli.net, vec!["GND", "VCC"]);
        assert_eq!(cli.layer, vec!["Top"]);
        assert_eq!(cli.xmin, Some(0.1));
        assert_eq!(cli.xmax, Some(0.2));
        assert_eq!(cli.ymin, Some(0.3));
        assert_eq!(cli.ymax, Some(0.4));
        assert_eq!(cli.zmin, Some(0.0));
        assert_eq!(cli.zmax, Some(0.001));
        assert!(cli.raw);
        assert!(cli.verbose);
        assert_eq!(cli.debug, 2);
        assert_eq!(cli.infile, vec!["board.hyp"]);
    }

    #[test]
    fn bounds_only_when_given() {
        let cli = parse(&["hyp2mat", "board.hyp"]);
        assert_eq!(conversion_options(&cli).bounds, None);
        let cli = parse(&["hyp2mat", "--xmin", "0.1", "board.hyp"]);
        let bounds = conversion_options(&cli).bounds.unwrap();
        assert_eq!(bounds.x0, 0.1);
        assert_eq!(bounds.x1, f64::MAX);
    }

    #[test]
    fn negative_grid_and_arc_precision_fall_back() {
        let cli = parse(&["hyp2mat", "--grid", "-1", "--arc-precision", "-1", "x.hyp"]);
        let options = conversion_options(&cli);
        assert_eq!(options.grid, 1e-5);
        assert_eq!(options.arc_precision, 0.0);
    }

    #[test]
    fn version_text_matches_native_shape() {
        assert!(version_string().starts_with("hyp2mat "));
    }
}
