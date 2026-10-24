//! QucsStudio output writer — port of `hyp2mat/lib/qucsstudio.cc`.
//! Outputs a QucsStudio layout file.

use crate::model::{Layer, PCB};

/// QucsStudio layout file version.
const PACKAGE_VERSION: &str = "3.3.1";

/// Write a PCB as a QucsStudio layout file.
pub fn write_qucsstudio(pcb: &PCB) -> Result<String, String> {
    let mut out = String::new();

    out.push_str(&format!("<QucsStudio Layout {}>\n", PACKAGE_VERSION));

    export_board(&mut out, pcb)?;

    out.push_str("<Items>\n");

    // export metalization (first layer only, matching C++ behavior)
    if let Some(first_layer) = pcb.stackup.first() {
        export_layer(&mut out, first_layer)?;
        if pcb.stackup.len() > 1 {
            // C++ prints a warning to stderr; we ignore it here
        }
    }

    export_vias(&mut out, pcb)?;
    export_devices(&mut out, pcb)?;
    export_ports(&mut out, pcb)?;

    out.push_str("</Items>\n");

    Ok(out)
}

/// Export the board properties.
fn export_board(out: &mut String, pcb: &PCB) -> Result<(), String> {
    let bounds = compute_bounds(pcb);
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
    out.push_str(&format!("Boundary={}|{}|2\n", dx, dy));

    let mut no_substrate_found = true;
    let thickness = pcb.stackup.first().map_or(0.0, |l| l.thickness);

    for poly in &pcb.board.polys {
        if poly.is_hole {
            continue; // don't output holes
        }

        for layer in pcb.stackup.iter().rev() {
            if layer.layer_type != "dielectric" {
                continue; // only output dielectrics
            }

            if no_substrate_found {
                no_substrate_found = false;
                out.push_str(&format!(
                    "Substrate={}|{}|{}|{}|0|{}\n",
                    layer.name,
                    layer.epsilon_r,
                    layer.z1 - layer.z0,
                    layer.loss_tangent,
                    thickness
                ));
            }
            // C++ prints a warning to stderr for more than one substrate; we ignore it
        }
    }

    if no_substrate_found {
        out.push_str("Substrate=Subst1|1.0|1 mm|0|0|0\n");
    }

    out.push_str("</Properties>\n");

    Ok(())
}

/// Compute bounding box from PCB data.
fn compute_bounds(pcb: &PCB) -> crate::model::Bounds {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for poly in &pcb.board.polys {
        for pt in &poly.poly.points {
            x_min = x_min.min(pt.x);
            x_max = x_max.max(pt.x);
            y_min = y_min.min(pt.y);
            y_max = y_max.max(pt.y);
        }
    }

    for layer in &pcb.stackup {
        for poly in &layer.metal.polys {
            for pt in &poly.poly.points {
                x_min = x_min.min(pt.x);
                x_max = x_max.max(pt.x);
                y_min = y_min.min(pt.y);
                y_max = y_max.max(pt.y);
            }
        }
    }

    for via in &pcb.via {
        x_min = x_min.min(via.x - via.radius);
        x_max = x_max.max(via.x + via.radius);
        y_min = y_min.min(via.y - via.radius);
        y_max = y_max.max(via.y + via.radius);
    }

    for pin in &pcb.pin {
        x_min = x_min.min(pin.x);
        x_max = x_max.max(pin.x);
        y_min = y_min.min(pin.y);
        y_max = y_max.max(pin.y);
    }

    if x_min.is_infinite() {
        x_min = 0.0;
        x_max = 0.0;
        y_min = 0.0;
        y_max = 0.0;
    }

    let z_min = pcb.stackup.first().map_or(0.0, |l| l.z0);
    let z_max = pcb.stackup.last().map_or(0.0, |l| l.z1);

    crate::model::Bounds {
        x0: x_min,
        y0: y_min,
        x1: x_max,
        y1: y_max,
        z0: z_min,
        z1: z_max,
    }
}

/// Export a metallization layer.
fn export_layer(out: &mut String, layer: &Layer) -> Result<(), String> {
    let mut n = 0usize;

    for poly in &layer.metal.polys {
        if poly.is_hole {
            continue;
        }

        let edge = &poly.poly;
        if edge.points.is_empty() {
            continue;
        }

        let first = &edge.points[0];
        let mut cx = first.x;
        let mut cy = first.y;

        // Check if it's a rectangle (4 points)
        if edge.points.len() == 4 {
            let p0 = &edge.points[0];
            let p1 = &edge.points[1];
            let p2 = &edge.points[2];
            let p3 = &edge.points[3];

            // Case 1: p0.x == p1.x, p1.y == p2.y, p2.x == p3.x, p3.y == p0.y
            if (p0.x - p1.x).abs() < 1e-12
                && (p1.y - p2.y).abs() < 1e-12
                && (p2.x - p3.x).abs() < 1e-12
                && (p3.y - p0.y).abs() < 1e-12
            {
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
                    n, cx, cy, dx, dy
                ));
                n += 1;
                continue;
            }

            // Case 2: p0.y == p1.y, p1.x == p2.x, p2.y == p3.y, p3.x == p0.x
            if (p0.y - p1.y).abs() < 1e-12
                && (p1.x - p2.x).abs() < 1e-12
                && (p2.y - p3.y).abs() < 1e-12
                && (p3.x - p0.x).abs() < 1e-12
            {
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
                    n, cx, cy, dx, dy
                ));
                n += 1;
                continue;
            }
        }

        out.push_str(&format!("Polygon metal{} {} {} ", n, cx, cy));
        for pt in &edge.points {
            out.push_str(&format!("|{}|{}", pt.x - cx, pt.y - cy));
        }
        out.push('\n');
        n += 1;
    }

    Ok(())
}

/// Export vias.
fn export_vias(out: &mut String, pcb: &PCB) -> Result<(), String> {
    for via in &pcb.via {
        out.push_str(&format!("Via X {} {} {}\n", via.x, via.y, via.radius));
    }
    Ok(())
}

/// Export devices (to stderr in C++; we include them in output).
fn export_devices(out: &mut String, pcb: &PCB) -> Result<(), String> {
    for device in &pcb.device {
        out.push_str(&format!(
            "Device: name={}, ref={}",
            device.name, device.ref_
        ));
        if device.value_type == "float" {
            out.push_str(&format!(", value={}", device.value_float));
        } else if device.value_type == "string" {
            out.push_str(&format!(", value={}", device.value_string));
        }
        out.push_str(&format!(", layer_name={}\n", device.layer_name));
    }
    Ok(())
}

/// Export ports (pins).
fn export_ports(out: &mut String, pcb: &PCB) -> Result<(), String> {
    let mut no = 0usize;
    for pin in &pcb.pin {
        no += 1;

        // C++ has this block #if 0'd out; it computes port dimensions
        let mut x_max = 0.0f64;
        let mut y_max = 0.0f64;
        let mut x_min = 0.0f64;
        let mut y_min = 0.0f64;
        let mut first = true;
        for pt in &pin.metal.points {
            if pt.x > x_max || first {
                x_max = pt.x;
            }
            if pt.y > y_max || first {
                y_max = pt.y;
            }
            if pt.x < x_min || first {
                x_min = pt.x;
            }
            if pt.y < y_min || first {
                y_min = pt.y;
            }
            first = false;
        }

        // determine whether port is on top or bottom layer of pcb
        let stackup = &pcb.stackup;
        let dbottom = (pin.z0 - stackup.last().map_or(0.0, |l| l.z0)).abs();
        let dtop = (pin.z1 - stackup.first().map_or(0.0, |l| l.z1)).abs();
        if dtop > dbottom {
            // port on bottom? skip
            continue;
        }

        let mut x_max = x_max - x_min;
        let mut y_max = y_max - y_min;
        if x_max >= y_max {
            y_max = 0.0;
        } else {
            x_max = 0.0;
        }

        out.push_str(&format!(
            "Lumped {} {} {} {} 2 50 0 {} {}\n",
            pin.ref_, pin.x, pin.y, no, x_max, y_max
        ));
    }
    Ok(())
}
