//! QucsStudio layout export — port of `hyp2mat/lib/qucsstudio.cc`.
//!
//! The native exporter only writes the *first* stackup layer as layout items
//! and warns when more layers are present; device and port information is
//! written to stderr instead of the layout. Both behaviours are preserved.

use crate::csxcad::format_cpp_float;
use crate::model::{Layer, PCB};

/// QucsStudio layout file version written by the native exporter.
const PACKAGE_VERSION: &str = "3.3.1";

/// Render a PCB as a QucsStudio layout file.
///
/// Returns the file contents together with the messages the native exporter
/// writes to stderr.
pub fn write_qucsstudio(pcb: &PCB) -> Result<(String, Vec<String>), String> {
    let mut out = String::new();
    let mut warnings = Vec::new();

    out.push_str(&format!("<QucsStudio Layout {}>\n", PACKAGE_VERSION));
    export_board(&mut out, pcb, &mut warnings);
    out.push_str("<Items>\n");

    if let Some(first_layer) = pcb.stackup.first() {
        export_layer(&mut out, first_layer);
        if pcb.stackup.len() > 1 {
            warnings.push("Warning: More than one layer found. Ignoring it.".to_string());
        }
    }

    export_vias(&mut out, pcb);
    export_devices(pcb, &mut warnings);
    export_ports(pcb, &mut warnings);

    out.push_str("</Items>\n");
    Ok((out, warnings))
}

fn export_board(out: &mut String, pcb: &PCB, warnings: &mut Vec<String>) {
    let brightness = pcb.stackup.first().map_or(0.0, |l| l.thickness);
    let bounds = crate::pcb::get_bounds(pcb);
    let dx = 0.5 * (bounds.x1 - bounds.x0);
    let dy = 0.5 * (bounds.y1 - bounds.y0);

    out.push_str("<Properties>\n");
    out.push_str("DataSet=*.dat\n");
    out.push_str("DataDisplay=*.dpl\n");
    out.push_str("OpenDisplay=1\n");
    out.push_str("Grid=0.1 mm\n");
    out.push_str("Mesh=20\n");
    out.push_str("Excite=2|0.0001|0\n");
    out.push_str("Frequency=1 GHz|9 GHz|401\n");
    out.push_str(&format!(
        "Boundary={}|{}|2\n",
        format_cpp_float(dx),
        format_cpp_float(dy)
    ));

    let mut no_substrate_found = true;
    for poly in &pcb.board.polys {
        if poly.is_hole {
            continue;
        }
        for layer in pcb.stackup.iter().rev() {
            if layer.layer_type != "dielectric" {
                continue;
            }
            if no_substrate_found {
                no_substrate_found = false;
                out.push_str(&format!(
                    "Substrate={}|{}|{}|{}|0|{}\n",
                    layer.name,
                    format_cpp_float(layer.epsilon_r),
                    format_cpp_float(layer.z1 - layer.z0),
                    format_cpp_float(layer.loss_tangent),
                    format_cpp_float(brightness)
                ));
            } else {
                warnings.push("Warning: More than one substrate found. Ignoring it.".to_string());
            }
        }
    }

    if no_substrate_found {
        out.push_str("Substrate=Subst1|1.0|1 mm|0|0|0\n");
    }
    out.push_str("</Properties>\n");
}

fn export_layer(out: &mut String, layer: &Layer) {
    let mut n = 0usize;
    for poly in &layer.metal.polys {
        if poly.is_hole {
            continue;
        }
        let edge = &poly.poly;
        let Some(first) = edge.points.first() else {
            continue;
        };
        let mut cx = first.x;
        let mut cy = first.y;

        if edge.points.len() == 4 {
            let p0 = edge.points[0];
            let p1 = edge.points[1];
            let p2 = edge.points[2];
            let p3 = edge.points[3];
            if p0.x == p1.x && p1.y == p2.y && p2.x == p3.x && p3.y == p0.y {
                let mut dx = p2.x - p1.x;
                if dx < 0.0 {
                    dx = -dx;
                    cx = p2.x;
                }
                let mut dy = p1.y - p0.y;
                if dy < 0.0 {
                    dy = -dy;
                    cy = p1.y;
                }
                out.push_str(&format!(
                    "Rectangle metal{} {} {} 0|0|{}|{}\n",
                    n,
                    format_cpp_float(cx),
                    format_cpp_float(cy),
                    format_cpp_float(dx),
                    format_cpp_float(dy)
                ));
                n += 1;
                continue;
            } else if p0.y == p1.y && p1.x == p2.x && p2.y == p3.y && p3.x == p0.x {
                let mut dx = p1.x - p0.x;
                if dx < 0.0 {
                    dx = -dx;
                    cx = p1.x;
                }
                let mut dy = p2.y - p1.y;
                if dy < 0.0 {
                    dy = -dy;
                    cy = p2.y;
                }
                out.push_str(&format!(
                    "Rectangle metal{} {} {} 0|0|{}|{}\n",
                    n,
                    format_cpp_float(cx),
                    format_cpp_float(cy),
                    format_cpp_float(dx),
                    format_cpp_float(dy)
                ));
                n += 1;
                continue;
            }
        }

        out.push_str(&format!(
            "Polygon metal{} {} {} ",
            n,
            format_cpp_float(cx),
            format_cpp_float(cy)
        ));
        for point in &edge.points {
            out.push_str(&format!(
                "|{}|{}",
                format_cpp_float(point.x - cx),
                format_cpp_float(point.y - cy)
            ));
        }
        out.push('\n');
        n += 1;
    }
}

fn export_vias(out: &mut String, pcb: &PCB) {
    for via in &pcb.via {
        out.push_str(&format!(
            "Via X {} {} {}\n",
            format_cpp_float(via.x),
            format_cpp_float(via.y),
            format_cpp_float(via.radius)
        ));
    }
}

fn export_devices(pcb: &PCB, warnings: &mut Vec<String>) {
    for device in &pcb.device {
        let mut line = format!("Device: name={}, ref={}", device.name, device.ref_);
        if device.value_type == "float" {
            line.push_str(&format!(", value={}", format_cpp_float(device.value_float)));
        } else if device.value_type == "string" {
            line.push_str(&format!(", value={}", device.value_string));
        }
        line.push_str(&format!(", layer_name={}", device.layer_name));
        warnings.push(line);
    }
}

fn export_ports(pcb: &PCB, warnings: &mut Vec<String>) {
    for pin in &pcb.pin {
        warnings.push(format!(
            "Port: ref={}, x={}, y={}",
            pin.ref_,
            format_cpp_float(pin.x),
            format_cpp_float(pin.y)
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FloatPoint, FloatPoly, FloatPolygon, FloatPolygons, Layer, PCB};

    fn pcb_with_rect() -> PCB {
        let board = FloatPolygons {
            polys: vec![FloatPoly {
                poly: FloatPolygon {
                    points: vec![
                        FloatPoint { x: 0.0, y: 0.0 },
                        FloatPoint { x: 0.02, y: 0.0 },
                        FloatPoint { x: 0.02, y: 0.01 },
                        FloatPoint { x: 0.0, y: 0.01 },
                    ],
                },
                is_hole: false,
                nesting_level: 0,
            }],
        };
        // Clockwise rectangle as produced by the polygon library.
        let metal = FloatPolygons {
            polys: vec![FloatPoly {
                poly: FloatPolygon {
                    points: vec![
                        FloatPoint { x: 0.004, y: 0.002 },
                        FloatPoint { x: 0.004, y: 0.006 },
                        FloatPoint { x: 0.008, y: 0.006 },
                        FloatPoint { x: 0.008, y: 0.002 },
                    ],
                },
                is_hole: false,
                nesting_level: 0,
            }],
        };
        PCB {
            board,
            stackup: vec![
                Layer {
                    name: "Top".into(),
                    layer_type: "signal".into(),
                    thickness: 3.5e-5,
                    z0: 1e-3,
                    z1: 1e-3,
                    epsilon_r: 1.0,
                    loss_tangent: 0.0,
                    bulk_resistivity: 1.724e-8,
                    metal: metal.clone(),
                },
                Layer {
                    name: "DL01".into(),
                    layer_type: "dielectric".into(),
                    thickness: 1e-3,
                    z0: 0.0,
                    z1: 1e-3,
                    epsilon_r: 4.4,
                    loss_tangent: 0.02,
                    bulk_resistivity: 0.0,
                    metal: FloatPolygons { polys: Vec::new() },
                },
            ],
            via: Vec::new(),
            via_plating_thickness: 0.0,
            device: Vec::new(),
            pin: Vec::new(),
        }
    }

    #[test]
    fn qucs_layout_structure() {
        let (text, warnings) = write_qucsstudio(&pcb_with_rect()).unwrap();
        assert!(text.starts_with("<QucsStudio Layout 3.3.1>\n<Properties>\n"));
        assert!(text.contains("Substrate=DL01|4.4|0.001|0.02|0|3.5e-05\n"));
        assert!(text.contains("Boundary=0.01|0.005|2\n"));
        assert!(text.contains("Rectangle metal0 0.004 0.002 0|0|0.004|0.004\n"));
        assert!(text.ends_with("</Items>\n"));
        assert!(warnings
            .iter()
            .any(|w| w.contains("More than one layer found")));
    }

    #[test]
    fn qucs_substrate_falls_back_to_default() {
        let mut pcb = pcb_with_rect();
        pcb.stackup.retain(|l| l.layer_type != "dielectric");
        let (text, _) = write_qucsstudio(&pcb).unwrap();
        assert!(text.contains("Substrate=Subst1|1.0|1 mm|0|0|0\n"));
    }
}
