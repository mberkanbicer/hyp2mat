//! CLI parity tests (RM-111) for the native `hyp2mat` interface.

mod common;

use common::{fixtures_dir, reference_dir, run_cli, run_cli_in, TempDir};

fn fixture(name: &str) -> String {
    fixtures_dir()
        .join(format!("{name}.hyp"))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn help_lists_the_native_options() {
    let output = run_cli(&["--help"], None);
    assert_eq!(output.status, 0);
    let text = output.stdout_str();
    for needle in [
        "-o, --output",
        "-f, --output-format",
        "-n, --net",
        "-l, --layer",
        "-x, --xmin",
        "-X, --xmax",
        "-y, --ymin",
        "-Y, --ymax",
        "-z, --zmin",
        "-Z, --zmax",
        "--grid",
        "--arc-precision",
        "--clearance",
        "--flood",
        "--epsilonr",
        "--loss-tangent",
        "--bulk-resistivity",
        "--pcb-outline",
        "--lossy-copper",
        "--metal-3d",
        "--hue",
        "--saturation",
        "--brightness",
        "-r, --raw",
        "-d, --debug",
        "-v, --verbose",
    ] {
        assert!(text.contains(needle), "help is missing {needle}:\n{text}");
    }
    assert!(text.contains("All lengths are in meters."));
}

#[test]
fn version_matches_native_format() {
    let output = run_cli(&["--version"], None);
    assert_eq!(output.status, 0);
    let text = output.stdout_str();
    assert!(
        text.starts_with("hyp2mat "),
        "unexpected version text: {text}"
    );
    let short = run_cli(&["-V"], None);
    assert_eq!(short.stdout_str(), text);
}

#[test]
fn missing_input_file_fails_with_native_message() {
    let output = run_cli(&["/definitely/not/a/file.hyp"], None);
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("Can't open input file"));
    assert!(output.stderr.contains("/definitely/not/a/file.hyp"));
}

#[test]
fn more_than_one_input_file_is_rejected() {
    let output = run_cli(
        &[&fixture("synthetic_minimal"), &fixture("synthetic_flood")],
        None,
    );
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("more than one input file given"));
}

#[test]
fn pdf_without_output_file_matches_native() {
    let temp = TempDir::new("pdf-no-output");
    let output = run_cli_in(Some(temp.path()), &[&fixture("synthetic_minimal")], None);
    assert_eq!(output.status, 0);
    assert_eq!(output.stderr.trim_end(), "pdf output file missing");
    assert!(output.stdout.is_empty());
    // Nothing is written to the working directory.
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn csxcad_writes_to_stdout_with_dash() {
    let output = run_cli(
        &["-f", "csxcad", "-o", "-", &fixture("synthetic_minimal")],
        None,
    );
    assert_eq!(output.status, 0);
    assert!(output.stdout_str().starts_with("function CSX = pcb(CSX)\n"));
}

#[test]
fn stdin_input_matches_file_input() {
    let input = std::fs::read(fixtures_dir().join("synthetic_metric_weight.hyp")).unwrap();
    let from_stdin = run_cli(&["-f", "csxcad", "-o", "-"], Some(&input));
    assert_eq!(from_stdin.status, 0);
    let from_file = run_cli(
        &[
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("synthetic_metric_weight"),
        ],
        None,
    );
    assert_eq!(from_stdin.stdout, from_file.stdout);
}

#[test]
fn unknown_option_fails() {
    let output = run_cli(&["--not-an-option"], None);
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("unexpected argument"));
}

#[test]
fn invalid_output_format_fails() {
    // The native CLI restricts the enum; an invalid value is a usage error.
    let output = run_cli(
        &["-f", "gerber", "-o", "-", &fixture("synthetic_minimal")],
        None,
    );
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("invalid value"));
}

#[test]
fn syntax_errors_are_reported_but_conversion_continues() {
    let temp = TempDir::new("syntax");
    let input = temp.join("bad.hyp");
    std::fs::write(
        &input,
        "{VERSION=2.10}\n{UNITS=METRIC LENGTH}\n{BOGUS 1}\n{STACKUP\n(SIGNAL T=0.00035 L=Top)\n(DIELECTRIC T=0.1 L=DL)\n(SIGNAL T=0.00035 L=Bottom)\n}\n{NET=N1\n(SEG X1=0.1 Y1=0.1 X2=0.5 Y2=0.1 W=0.02 L=Top)\n}\n{END}\n",
    )
    .unwrap();
    let output_path = temp.join("out.m");
    let output = run_cli(
        &[
            "-f",
            "csxcad",
            "-o",
            &output_path.to_string_lossy(),
            &input.to_string_lossy(),
        ],
        None,
    );
    assert_eq!(output.status, 0, "native exits 0 after recovering");
    assert!(output.stderr.contains("syntax error"), "{}", output.stderr);
    let script = std::fs::read_to_string(&output_path).unwrap();
    assert!(script.contains("CSX = AddHyperLynxMetal2D(CSX, 'Top_copper'"));
}

#[test]
fn verbose_summary_matches_native() {
    let output = run_cli(
        &[
            "--verbose",
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("real_notchfilter"),
        ],
        None,
    );
    assert_eq!(output.status, 0);
    let reference = std::fs::read_to_string(reference_dir().join("real_notchfilter.summary.txt"))
        .expect("summary reference");
    let expected: Vec<&str> = reference
        .lines()
        .filter(|line| !line.starts_with("Warning:"))
        .collect();
    let actual: Vec<&str> = output.stderr.lines().collect();
    assert_eq!(actual, expected);
}

#[test]
fn net_question_mark_lists_nets() {
    let output = run_cli(
        &["-n", "?", "-f", "csxcad", "-o", "-", &fixture("real_msl")],
        None,
    );
    assert_eq!(output.status, 0);
    let first_line = output
        .stdout_str()
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    assert_eq!(first_line, "nets: GND V1 V2");
}

#[test]
fn layer_selection_limits_the_stackup() {
    let output = run_cli(
        &["-l", "Top", "-f", "csxcad", "-o", "-", &fixture("real_msl")],
        None,
    );
    assert_eq!(output.status, 0);
    let script = output.stdout_str();
    assert!(script.contains("CSX = AddHyperLynxMetal2D(CSX, 'Top_copper'"));
    assert!(!script.contains("Bottom_copper"));
    // Only one layer remains, so the mesh height collapses to the metal sheet.
    assert!(script.contains("mesh.z = [0.00183617 0.00183617];"));
}

#[test]
fn net_selection_limits_the_copper() {
    let output = run_cli(
        &["-n", "V1", "-f", "csxcad", "-o", "-", &fixture("real_msl")],
        None,
    );
    assert_eq!(output.status, 0);
    let script = output.stdout_str();
    // The V1 pour plus its two pins remain; the GND pour is gone.
    assert!(script.contains("CSX = AddPolygon(CSX, 'Top_copper'"));
    assert!(!script.contains("CSX = AddHyperLynxMetal2D(CSX, 'Bottom_copper'"));
}

#[test]
fn unwritable_output_fails() {
    let temp = TempDir::new("unwritable");
    let output = run_cli(
        &[
            "-f",
            "csxcad",
            "-o",
            &temp.join("missing/directory/out.m").to_string_lossy(),
            &fixture("synthetic_minimal"),
        ],
        None,
    );
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("cannot open"));
}

#[test]
fn debug_writes_the_model_dump_in_the_working_directory() {
    let temp = TempDir::new("debug");
    let output = run_cli_in(
        Some(temp.path()),
        &[
            "-d",
            "-f",
            "csxcad",
            "-o",
            &temp.join("out.m").to_string_lossy(),
            &fixture("synthetic_minimal"),
        ],
        None,
    );
    assert_eq!(output.status, 0);
    assert!(output.stderr.contains("using Clipper2"));
    let dump = std::fs::read_to_string(temp.join("hyp_debug.txt")).expect("hyp_debug.txt");
    assert!(dump.starts_with("unit = 0.01\n"));
}

#[test]
fn qucs_and_xml_formats_are_reachable_from_the_cli() {
    let qucs = run_cli(
        &["-f", "qucs", "-o", "-", &fixture("synthetic_minimal")],
        None,
    );
    assert_eq!(qucs.status, 0);
    assert!(qucs.stdout_str().starts_with("<QucsStudio Layout 3.3.1>"));

    let xml = run_cli(
        &[
            "--csxcad-xml",
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("synthetic_minimal"),
        ],
        None,
    );
    assert_eq!(xml.status, 0);
    assert!(xml.stdout_str().starts_with("<?xml"));
}

#[test]
fn pdf_can_be_written_to_stdout_explicitly() {
    let output = run_cli(
        &["-f", "pdf", "-o", "-", &fixture("synthetic_minimal")],
        None,
    );
    assert_eq!(output.status, 0);
    assert!(output.stdout.starts_with(b"%PDF-1.4\n"));
}

#[test]
fn material_overrides_reach_the_writer() {
    let output = run_cli(
        &[
            "--epsilonr",
            "2.5",
            "--loss-tangent",
            "0.01",
            "--bulk-resistivity",
            "1e-7",
            "--lossy-copper",
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("real_notchfilter"),
        ],
        None,
    );
    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    let script = output.stdout_str();
    assert!(script.contains("AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 2.5, 0.01)"));
    assert!(script.contains("AddHyperLynxMetal2D(CSX, 'Top_copper', 1e-07,"));
}

#[test]
fn flooding_without_a_board_outline_fails_cleanly() {
    // The native binary dereferences an empty board edge here; this port
    // reports an error instead of crashing.
    let output = run_cli(
        &[
            "--flood",
            "Top",
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("synthetic_minimal"),
        ],
        None,
    );
    assert_eq!(output.status, 1);
    assert!(
        output
            .stderr
            .contains("cannot flood layers: board outline is empty"),
        "{}",
        output.stderr
    );
}

#[test]
fn negative_numeric_options_are_accepted() {
    let output = run_cli(
        &[
            "--grid",
            "-1",
            "--arc-precision",
            "-1",
            "--clearance",
            "-1",
            "-f",
            "csxcad",
            "-o",
            "-",
            &fixture("synthetic_minimal"),
        ],
        None,
    );
    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
}
