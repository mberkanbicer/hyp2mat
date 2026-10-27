//! Fixture-corpus differential tests (RM-107, RM-108, RM-109, RM-110).
//!
//! The reference outputs in `tests/reference/` were produced by the native
//! hyp2mat 0.0.18 binary built from the pinned sources. The tests below compare
//! the Rust port against them:
//!
//! * the parser model (`hyp_debug.txt`) is compared **byte for byte**;
//! * `--raw` polygons are compared **byte for byte**;
//! * the CSXCAD script and QucsStudio layout are compared structurally, with
//!   polygon geometry compared as vertex cycles. Seven of the nine fixtures are
//!   geometrically identical; the two fixtures with copper arcs differ by at
//!   most one 10 µm grid step on one intersection vertex, because Clipper2 and
//!   the native Clipper 1 resolve shallow intersections with different integer
//!   rounding. That bound is asserted explicitly below.

mod common;

use common::{
    canonical_polygon, cyclic_distance, fixture_names, fixtures_dir, parse_qucs_items,
    parse_script_polygons, polygon_area, reference_dir, run_cli, sha256_hex, to_float_pairs,
    TempDir,
};

/// One grid step (the model's coordinate resolution).
const GRID: f64 = 1e-5;

fn geometry_tolerance(fixture: &str) -> f64 {
    match fixture {
        // Copper arcs: the polygon union of consecutive arc segments produces a
        // shallow intersection that Clipper1 and Clipper2 round differently.
        "synthetic_metric_weight" | "synthetic_plane_clearance" => 1.5 * GRID,
        _ => 0.0,
    }
}

fn run_script(fixture: &str, extra_args: &[&str]) -> (String, TempDir) {
    let temp = TempDir::new("script");
    let output = temp.join("pcb.m");
    let input = fixtures_dir().join(format!("{fixture}.hyp"));
    let mut args: Vec<String> = vec![
        "-f".into(),
        "csxcad".into(),
        "-o".into(),
        output.to_string_lossy().into_owned(),
    ];
    args.extend(extra_args.iter().map(|s| s.to_string()));
    args.push(input.to_string_lossy().into_owned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = run_cli(&refs, None);
    assert_eq!(result.status, 0, "stderr: {}", result.stderr);
    let script = std::fs::read_to_string(&output).expect("read script");
    (script, temp)
}

/// Split a script into lines, removing `pgon` data so polygon ordering and
/// rotation do not affect the comparison.
fn script_without_polygons(script: &str) -> Vec<String> {
    let mut lines: Vec<String> = script
        .lines()
        .filter(|line| !line.starts_with("pgon"))
        .map(|line| line.to_string())
        .collect();
    lines.sort();
    lines
}

fn sorted_polygons(script: &str) -> Vec<Vec<(i64, i64)>> {
    let mut polygons: Vec<Vec<(i64, i64)>> = parse_script_polygons(script)
        .iter()
        .map(|p| canonical_polygon(p))
        .collect();
    polygons.sort();
    polygons
}

#[test]
fn manifest_matches_files() {
    let manifest =
        std::fs::read_to_string(fixtures_dir().join("manifest.sha256")).expect("read manifest");
    let mut listed = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 3, "malformed manifest line: {line}");
        assert_eq!(parts[0], "sha256");
        let name = parts[2];
        let bytes = std::fs::read(fixtures_dir().join(name)).expect("fixture file");
        assert_eq!(
            sha256_hex(&bytes),
            parts[1],
            "SHA-256 mismatch for fixture {name}"
        );
        listed.push(name.trim_end_matches(".hyp").to_string());
    }
    listed.sort();
    assert_eq!(listed, fixture_names());
}

#[test]
fn dump_matches_native_reference() {
    for fixture in fixture_names() {
        let temp = TempDir::new("dump");
        let output = temp.join("pcb.m");
        let input = fixtures_dir().join(format!("{fixture}.hyp"));
        let result = common::run_cli_in(
            Some(temp.path()),
            &[
                "-d",
                "-f",
                "csxcad",
                "-o",
                &output.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(result.status, 0, "stderr: {}", result.stderr);
        let dump = std::fs::read_to_string(temp.join("hyp_debug.txt")).expect("dump");
        let reference = std::fs::read_to_string(
            reference_dir()
                .join("dump")
                .join(format!("{fixture}.dump.txt")),
        )
        .expect("reference dump");
        assert_eq!(dump, reference, "model dump differs for {fixture}");
    }
}

#[test]
fn raw_output_matches_native() {
    for fixture in [
        "real_hairpinfilter",
        "real_notchfilter",
        "synthetic_metric_weight",
        "synthetic_plane_clearance",
    ] {
        let (script, _temp) = run_script(fixture, &["--raw"]);
        let reference =
            std::fs::read_to_string(reference_dir().join("raw").join(format!("{fixture}.raw.m")))
                .expect("raw reference");
        assert_eq!(
            script, reference,
            "--raw output differs from native for {fixture}"
        );
    }
}

#[test]
fn csxcad_script_matches_native() {
    for fixture in fixture_names() {
        let (script, _temp) = run_script(&fixture, &[]);
        let reference = std::fs::read_to_string(reference_dir().join(format!("{fixture}.m")))
            .expect("reference script");

        // Structure, materials, priorities, devices and ports must match
        // exactly (as a multiset, because polygon order follows the clipping
        // library).
        assert_eq!(
            script_without_polygons(&script),
            script_without_polygons(&reference),
            "non-geometry lines differ for {fixture}"
        );

        let ours = sorted_polygons(&script);
        let native = sorted_polygons(&reference);
        assert_eq!(
            ours.len(),
            native.len(),
            "polygon count differs for {fixture}"
        );
        let tolerance = geometry_tolerance(&fixture);
        let mut worst = 0.0f64;
        for (our, native) in ours.iter().zip(native.iter()) {
            assert_eq!(
                our.len(),
                native.len(),
                "vertex count differs for {fixture}"
            );
            let our_points: Vec<[f64; 2]> = our
                .iter()
                .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
                .collect();
            let native_points: Vec<[f64; 2]> = native
                .iter()
                .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
                .collect();
            let distance = cyclic_distance(&our_points, &native_points);
            worst = worst.max(distance);
        }
        assert!(
            worst <= tolerance,
            "polygon geometry differs by {worst:e} m for {fixture} (allowed {tolerance:e})"
        );
    }
}

#[test]
fn verbose_stderr_matches_native() {
    for fixture in fixture_names() {
        let temp = TempDir::new("verbose");
        let output = temp.join("pcb.m");
        let input = fixtures_dir().join(format!("{fixture}.hyp"));
        let result = run_cli(
            &[
                "--verbose",
                "-f",
                "csxcad",
                "-o",
                &output.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(result.status, 0);
        let reference = std::fs::read(reference_dir().join(format!("{fixture}.summary.txt")))
            .expect("summary reference");
        assert_eq!(
            result.stderr.as_bytes(),
            reference.as_slice(),
            "--verbose stderr differs for {fixture}"
        );
    }
}

#[test]
fn csxcad_script_is_deterministic() {
    for fixture in fixture_names() {
        let (first, _first_dir) = run_script(&fixture, &[]);
        let (second, _second_dir) = run_script(&fixture, &[]);
        assert_eq!(
            sha256_hex(first.as_bytes()),
            sha256_hex(second.as_bytes()),
            "non-deterministic output for {fixture}"
        );
    }
}

#[test]
fn csxcad_variants_match_native() {
    let (script, _temp) = run_script(
        "real_notchfilter",
        &["--pcb-outline", "--metal-3d", "--lossy-copper"],
    );
    let reference =
        std::fs::read_to_string(reference_dir().join("real_notchfilter_pcboutline_metal3d.m"))
            .expect("variant reference");
    assert_eq!(
        script_without_polygons(&script),
        script_without_polygons(&reference)
    );
    assert_eq!(sorted_polygons(&script), sorted_polygons(&reference));

    let (script, _temp) = run_script(
        "real_notchfilter",
        &[
            "--xmin", "0.005", "--xmax", "0.015", "--ymin", "0.002", "--ymax", "0.008",
        ],
    );
    let reference =
        std::fs::read_to_string(reference_dir().join("real_notchfilter_crop.m")).expect("crop");
    assert_eq!(
        script_without_polygons(&script),
        script_without_polygons(&reference)
    );
    assert_eq!(sorted_polygons(&script), sorted_polygons(&reference));

    let (script, _temp) = run_script(
        "synthetic_flood",
        &["--flood", "plane_layers", "--flood", "Top"],
    );
    let reference = std::fs::read_to_string(reference_dir().join("synthetic_flood_flooded.m"))
        .expect("flood reference");
    assert_eq!(
        script_without_polygons(&script),
        script_without_polygons(&reference)
    );
    let ours = sorted_polygons(&script);
    let native = sorted_polygons(&reference);
    assert_eq!(ours.len(), native.len());
    for (our, native) in ours.iter().zip(native.iter()) {
        assert_eq!(our.len(), native.len());
        let our_points: Vec<[f64; 2]> = our
            .iter()
            .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
            .collect();
        let native_points: Vec<[f64; 2]> = native
            .iter()
            .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
            .collect();
        assert!(cyclic_distance(&our_points, &native_points) <= GRID);
    }
}

#[test]
fn qucs_layout_matches_native() {
    for fixture in fixture_names() {
        let temp = TempDir::new("qucs");
        let output = temp.join("pcb.qucs");
        let input = fixtures_dir().join(format!("{fixture}.hyp"));
        let result = run_cli(
            &[
                "-f",
                "qucs",
                "-o",
                &output.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(result.status, 0, "stderr: {}", result.stderr);
        let layout = std::fs::read_to_string(&output).expect("layout");
        let reference = std::fs::read_to_string(reference_dir().join(format!("{fixture}.qucs")))
            .expect("reference layout");

        let properties = |text: &str| -> Vec<String> {
            text.lines()
                .skip_while(|line| !line.starts_with("<Properties>"))
                .take_while(|line| !line.starts_with("</Properties>"))
                .map(|line| line.to_string())
                .collect()
        };
        assert_eq!(
            properties(&layout),
            properties(&reference),
            "QucsStudio properties differ for {fixture}"
        );

        let mut ours: Vec<Vec<(i64, i64)>> = parse_qucs_items(&layout)
            .iter()
            .map(|p| canonical_polygon(p))
            .collect();
        let mut native: Vec<Vec<(i64, i64)>> = parse_qucs_items(&reference)
            .iter()
            .map(|p| canonical_polygon(p))
            .collect();
        ours.sort();
        native.sort();
        assert_eq!(ours.len(), native.len(), "item count differs for {fixture}");
        let tolerance = geometry_tolerance(&fixture);
        for (our, native) in ours.iter().zip(native.iter()) {
            assert_eq!(our.len(), native.len());
            let our_points: Vec<[f64; 2]> = our
                .iter()
                .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
                .collect();
            let native_points: Vec<[f64; 2]> = native
                .iter()
                .map(|&(x, y)| [x as f64 * GRID, y as f64 * GRID])
                .collect();
            let distance = cyclic_distance(&our_points, &native_points);
            assert!(
                distance <= tolerance,
                "QucsStudio geometry differs by {distance:e} m for {fixture}"
            );
        }

        // The native exporter reports devices, ports and multilayer warnings
        // on stderr; compare them as a multiset.
        let reference_stderr =
            std::fs::read_to_string(reference_dir().join(format!("{fixture}.qucs.stderr.txt")))
                .expect("reference stderr");
        let mut our_messages: Vec<String> = result
            .stderr
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        let mut native_messages: Vec<String> = reference_stderr
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        our_messages.sort();
        native_messages.sort();
        assert_eq!(
            our_messages, native_messages,
            "QucsStudio stderr differs for {fixture}"
        );
    }
}

#[test]
fn csxcad_xml_round_trips_with_matching_model() {
    use csxcad_core::material::MaterialField;
    use csxcad_core::model::Structure;

    for fixture in fixture_names() {
        let temp = TempDir::new("xml");
        let output = temp.join("pcb.xml");
        let script_path = temp.join("pcb.m");
        let input = fixtures_dir().join(format!("{fixture}.hyp"));
        let result = run_cli(
            &[
                "--csxcad-xml",
                "--pcb-outline",
                "-f",
                "csxcad",
                "-o",
                &output.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(result.status, 0, "stderr: {}", result.stderr);
        let xml = std::fs::read_to_string(&output).expect("read XML");
        let mut structure = Structure::from_xml_str(&xml)
            .unwrap_or_else(|error| panic!("{fixture}: generated XML does not parse: {error}"));

        // Grid lines are the mesh bounds written to the MATLAB script.
        let script = run_cli(
            &[
                "-f",
                "csxcad",
                "-o",
                &script_path.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(script.status, 0);
        let script_text = std::fs::read_to_string(&script_path).expect("script");
        let mesh_value = |axis: char| -> Vec<f64> {
            let prefix = format!("mesh.{axis} = [");
            let line = script_text
                .lines()
                .find(|line| line.starts_with(&prefix))
                .unwrap_or_else(|| panic!("{fixture}: missing {prefix}"));
            line.trim_start_matches(&prefix)
                .trim_end_matches("];")
                .split_whitespace()
                .map(|value| value.parse::<f64>().unwrap())
                .collect()
        };
        let grid = structure.grid_mut();
        let x = mesh_value('x');
        let y = mesh_value('y');
        let z = mesh_value('z');
        let close = |a: &[f64], b: &[f64]| {
            assert_eq!(a.len(), b.len(), "{fixture}: grid line count");
            for (left, right) in a.iter().zip(b) {
                // The MATLAB script prints 6 significant digits (half a unit
                // in the last digit).
                assert!(
                    (left - right).abs() <= 5e-6 * right.abs() + 1e-12,
                    "{fixture}: grid lines {left} vs {right}"
                );
            }
        };
        close(grid.lines(0).expect("x lines"), &x);
        close(grid.lines(1).expect("y lines"), &y);
        close(grid.lines(2).expect("z lines"), &z);

        // Materials: every dielectric layer becomes a Dielectric_* material
        // with the same epsilon as the script; copper layers become
        // <layer>_copper and holes <layer>_cutout; vias become 'via'.
        for layer_line in script_text
            .lines()
            .filter(|line| line.starts_with("CSX = AddHyperLynxDielectric("))
        {
            let name = layer_line
                .split('\'')
                .nth(1)
                .expect("material name")
                .to_string();
            let property = (0..structure.property_count())
                .map(|index| structure.property_by_index(index).unwrap())
                .find(|property| property.name() == name)
                .unwrap_or_else(|| panic!("{fixture}: missing material {name}"));
            let material = property.material().expect("material property");
            let epsilon = material
                .value(MaterialField::Epsilon, 0, None)
                .unwrap()
                .unwrap();
            let script_epsilon: f64 = layer_line
                .split(',')
                .nth(2)
                .expect("epsilon field")
                .trim()
                .parse()
                .expect("epsilon number");
            assert!(
                (epsilon - script_epsilon).abs() < 1e-12,
                "{fixture}: epsilon of {name}"
            );
        }

        // Copper primitives must correspond one-to-one with the script's
        // copper polygons, with the script priorities.
        let mut script_copper = 0usize;
        for line in script_text.lines() {
            if line.starts_with("CSX = AddPolygon(") || line.starts_with("CSX = AddLinPoly(") {
                if let Some(material) = line.split('\'').nth(1) {
                    if material.ends_with("_copper") || material.ends_with("_cutout") {
                        script_copper += 1;
                    }
                }
            }
        }
        let mut xml_polygons = 0usize;
        for index in 0..structure.property_count() {
            let property = structure.property_by_index(index).unwrap();
            let copper =
                property.name().ends_with("_copper") || property.name().ends_with("_cutout");
            for primitive_index in 0..property.primitive_count() {
                let primitive = property.primitive(primitive_index).unwrap();
                if primitive.polygon_shape().is_some() && copper {
                    xml_polygons += 1;
                }
                assert!(
                    (100..=120).contains(&primitive.priority())
                        || (200..=220).contains(&primitive.priority())
                        || primitive.priority() == 300,
                    "{fixture}: unexpected priority {}",
                    primitive.priority()
                );
            }
        }
        assert_eq!(
            script_copper, xml_polygons,
            "{fixture}: copper primitive count"
        );

        // XML serialization is stable and round-trips again.
        let xml_again = structure.to_xml_string().expect("serialize XML");
        let reparsed = Structure::from_xml_str(&xml_again).expect("reparse XML");
        assert_eq!(reparsed.property_count(), structure.property_count());
    }
}

#[test]
fn pdf_is_structurally_valid_and_deterministic() {
    for fixture in fixture_names() {
        let temp = TempDir::new("pdf");
        let output = temp.join("pcb.pdf");
        let input = fixtures_dir().join(format!("{fixture}.hyp"));
        let result = run_cli(
            &[
                "-f",
                "pdf",
                "-o",
                &output.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(result.status, 0, "stderr: {}", result.stderr);
        let pdf = std::fs::read(&output).expect("read PDF");
        let text = String::from_utf8(pdf.clone()).expect("ASCII PDF");
        assert!(text.starts_with("%PDF-1.4\n"));
        assert!(text.ends_with("%%EOF\n"));

        // Every object referenced by the xref must start with "N 0 obj".
        let xref_start = text.find("\nxref\n").expect("xref") + 1;
        let mut lines = text[xref_start..].lines();
        lines.next();
        let header = lines.next().expect("xref header");
        let count: usize = header.split_whitespace().nth(1).unwrap().parse().unwrap();
        for (index, line) in lines.take(count - 1).enumerate() {
            if index == 0 {
                assert_eq!(line, "0000000000 65535 f ");
                continue;
            }
            let offset: usize = line[..10].parse().unwrap();
            assert!(
                text[offset..].starts_with(&format!("{index} 0 obj")),
                "{fixture}: xref entry {index} is wrong"
            );
        }

        // One page per copper layer plus the composite view. The layer stackup
        // comes from the `--verbose` summary (metal layers without copper still
        // get a page).
        let verbose = run_cli(
            &[
                "--verbose",
                "-f",
                "csxcad",
                "-o",
                &temp.join("verbose.m").to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(verbose.status, 0);
        let mut layers = Vec::new();
        for line in verbose.stderr.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("signal ") || trimmed.starts_with("plane ") {
                let name = line
                    .split('\'')
                    .nth(1)
                    .unwrap_or_else(|| panic!("{fixture}: layer name in {line:?}"));
                layers.push(name.to_string());
            }
        }
        let pages = layers.len() + 1;
        assert!(
            text.contains(&format!("/Count {pages}")),
            "{fixture}: expected {pages} pages"
        );
        assert!(text.contains("(Composite) Tj"));
        for layer in &layers {
            assert!(
                text.contains(&format!("({layer}) Tj")),
                "{fixture}: missing caption for {layer}"
            );
        }

        // Deterministic output.
        let output2 = temp.join("pcb2.pdf");
        let second = run_cli(
            &[
                "-f",
                "pdf",
                "-o",
                &output2.to_string_lossy(),
                &input.to_string_lossy(),
            ],
            None,
        );
        assert_eq!(second.status, 0);
        assert_eq!(
            sha256_hex(&pdf),
            sha256_hex(&std::fs::read(&output2).unwrap())
        );
    }
}

#[test]
fn reference_pdf_has_same_page_count_and_captions() {
    let reference = std::fs::read(reference_dir().join("real_notchfilter.pdf")).expect("pdf");
    let text = String::from_utf8_lossy(&reference);
    assert!(text.contains("/Count 3"));
    let script = run_script("real_notchfilter", &[]).0;
    assert_eq!(
        script
            .lines()
            .filter(|line| line.starts_with("CSX = AddHyperLynxMetal"))
            .count()
            + 1,
        3
    );
}

#[test]
fn polygon_areas_are_preserved() {
    // Independent invariant: the total copper area of every fixture is
    // unchanged (relative differences only come from the 1-grid-step vertex
    // shifts allowed above).
    for fixture in fixture_names() {
        let (script, _temp) = run_script(&fixture, &[]);
        let reference = std::fs::read_to_string(reference_dir().join(format!("{fixture}.m")))
            .expect("reference");
        let mut our_total = 0.0;
        let mut native_total = 0.0;
        for polygon in parse_script_polygons(&script) {
            our_total += polygon_area(&to_float_pairs(&polygon)).abs();
        }
        for polygon in parse_script_polygons(&reference) {
            native_total += polygon_area(&to_float_pairs(&polygon)).abs();
        }
        let relative = (our_total - native_total).abs() / native_total.max(1e-30);
        let tolerance = if geometry_tolerance(&fixture) > 0.0 {
            5e-3
        } else {
            1e-12
        };
        assert!(
            relative <= tolerance,
            "{fixture}: total area relative difference {relative:e}"
        );
    }
}
