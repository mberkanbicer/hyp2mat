//! CSXCAD output writer — port of `hyp2mat/lib/csxcad.cc`.
//! Outputs a MATLAB/Octave script that builds a CSXCAD structure.

use crate::model::{Bounds, FloatPolygon, FloatPolygons, Layer, PCB};

/// CSXCAD export priorities (matching the C++ implementation).
const PRIO_DIELECTRIC: i32 = 100; // FR4 dielectric
const PRIO_MATERIAL: i32 = 200;   // copper
const PRIO_VIA: i32 = 300;        // via metal

/// Write a PCB as a CSXCAD MATLAB script.
pub fn write_csxcad(pcb: &PCB) -> Result<String, String> {
    let mut out = String::new();

    // Compute bounding box from board polygons and stackup
    let bounds = compute_bounds(pcb);

    // Export dielectric / board
    export_board(&mut out, pcb, &bounds, true)?;

    // Export copper layers
    out.push_str("% copper\n");
    let mut odd = false;
    for layer in &pcb.stackup {
        if layer.layer_type == "dielectric" {
            odd = !odd;
        }
        export_layer(&mut out, pcb, layer, false, false, odd)?;
    }

    // Export vias
    export_vias(&mut out, pcb)?;

    // Export devices
    export_devices(&mut out, pcb)?;

    // Export ports (pins)
    export_ports(&mut out, pcb)?;

    out.push_str("%not truncated\n");

    Ok(out)
}

/// Compute bounding box from PCB data.
fn compute_bounds(pcb: &PCB) -> Bounds {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    // From board polygons
    for poly in &pcb.board.polys {
        for pt in &poly.poly.points {
            x_min = x_min.min(pt.x);
            x_max = x_max.max(pt.x);
            y_min = y_min.min(pt.y);
            y_max = y_max.max(pt.y);
        }
    }

    // From layer metals
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

    // From vias
    for via in &pcb.via {
        x_min = x_min.min(via.x - via.radius);
        x_max = x_max.max(via.x + via.radius);
        y_min = y_min.min(via.y - via.radius);
        y_max = y_max.max(via.y + via.radius);
    }

    // From pins
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

    // Z bounds from stackup
    let _z_min = pcb.stackup.first().map_or(0.0, |l| l.z0);
    let _z_max = pcb.stackup.last().map_or(0.0, |l| l.z1);

    Bounds {
        x0: x_min,
        y0: y_min,
        x1: x_max,
        y1: y_max,
    }
}

/// Quote a string using MATLAB/Octave conventions.
fn string2matlab(str: &str) -> String {
    let mut ostring = String::from("'");
    for ch in str.chars() {
        if ch == '\'' {
            ostring.push('\'');
        }
        if ch == '%' {
            ostring.push('%');
        }
        ostring.push(ch);
    }
    ostring.push('\'');
    ostring
}

/// True if polygon list contains at least one (positive) polygon.
fn contains_polygon(polygons: &FloatPolygons) -> bool {
    polygons.polys.iter().any(|p| !p.is_hole)
}

/// True if polygon list contains at least one hole.
fn contains_hole(polygons: &FloatPolygons) -> bool {
    polygons.polys.iter().any(|p| p.is_hole)
}

/// Export a single polygon edge as MATLAB pgon assignments.
fn export_edge(out: &mut String, edge: &FloatPolygon) {
    out.push_str("pgon = [];\n");
    for pt in &edge.points {
        out.push_str(&format!("pgon(:, end+1) = [{};{}];\n", pt.x, pt.y));
    }
}

/// Export the board (dielectric layers and cutouts).
fn export_board(out: &mut String, pcb: &PCB, bounds: &Bounds, pcb_outline: bool) -> Result<(), String> {
    out.push_str("function CSX = pcb(CSX)\n");
    out.push_str("% matlab script created by hyp2mat\n");
    out.push_str("% create minimal mesh\n");
    out.push_str("mesh = {};\n");
    out.push_str(&format!("mesh.x = [{} {}];\n", bounds.x0, bounds.x1));
    out.push_str(&format!("mesh.y = [{} {}];\n", bounds.y0, bounds.y1));
    out.push_str(&format!("mesh.z = [{} {}];\n", 0.0, 0.0)); // placeholder
    out.push_str("% add mesh\n");
    out.push_str("CSX = DefineRectGrid(CSX, 1, mesh);\n");

    if contains_polygon(&pcb.board) {
        for poly in &pcb.board.polys {
            if poly.is_hole {
                continue;
            }
            for layer in pcb.stackup.iter().rev() {
                if layer.layer_type != "dielectric" {
                    continue;
                }
                out.push_str(&format!(
                    "CSX = AddHyperLynxDielectric(CSX, 'Dielectric_{}', {}, {});\n",
                    string2matlab(&layer.name),
                    layer.epsilon_r,
                    layer.loss_tangent
                ));
                out.push_str(&format!("% board outline, layer {}\n", layer.name));
                let priority = PRIO_DIELECTRIC + poly.nesting_level;
                if pcb_outline {
                    export_edge(out, &poly.poly);
                    out.push_str(&format!(
                        "CSX = AddLinPoly(CSX, 'Dielectric_{}', {}, 2, {}, pgon, {});\n",
                        string2matlab(&layer.name),
                        priority,
                        layer.z0,
                        layer.z1 - layer.z0
                    ));
                } else {
                    out.push_str(&format!(
                        "CSX = AddBox(CSX, 'Dielectric_{}', {}, [ {}, {}, {}], [ {}, {}, {}]  );\n",
                        string2matlab(&layer.name),
                        priority,
                        bounds.x0, bounds.y0, layer.z0,
                        bounds.x1, bounds.y1, layer.z1
                    ));
                }
            }
        }
    }

    if pcb_outline && contains_hole(&pcb.board) {
        out.push_str("% create board cutout material\n");
        out.push_str("CSX = AddMaterial( CSX, 'Drill');\n");
        out.push_str("CSX = SetMaterialProperty( CSX, 'Drill', 'Epsilon', 1, 'Mue', 1);\n");

        for poly in &pcb.board.polys {
            if !poly.is_hole {
                continue;
            }
            out.push_str("% board cutout\n");
            export_edge(out, &poly.poly);
            let priority = PRIO_DIELECTRIC + poly.nesting_level;
            out.push_str(&format!(
                "CSX = AddLinPoly(CSX, 'Drill', {}, 2, {}, pgon, {});\n",
                priority,
                0.0, // bounds.z_min placeholder
                0.0  // bounds.z_max - bounds.z_min placeholder
            ));
        }
    }

    Ok(())
}

/// Export a copper layer.
fn export_layer(
    out: &mut String,
    _pcb: &PCB,
    layer: &Layer,
    _lossy_copper: bool,
    _metal_3d: bool,
    odd: bool,
) -> Result<(), String> {
    let layer_material = format!("{}_copper", layer.name);
    let layer_cutout = format!("{}_cutout", layer.name);

    if contains_polygon(&layer.metal) {
        out.push_str(&format!("% create layer {} material\n", layer.name));
        let mut copper_resistivity = 0.0;
        if layer.bulk_resistivity > 0.0 {
            copper_resistivity = layer.bulk_resistivity;
        }
        if _metal_3d {
            out.push_str(&format!(
                "CSX = AddHyperLynxMetal3D(CSX, '{}', {}, {});\n",
                string2matlab(&layer_material),
                copper_resistivity,
                layer.thickness
            ));
        } else {
            out.push_str(&format!(
                "CSX = AddHyperLynxMetal2D(CSX, '{}', {}, {});\n",
                string2matlab(&layer_material),
                copper_resistivity,
                layer.thickness
            ));
        }
    }

    if contains_hole(&layer.metal) {
        out.push_str(&format!("% create layer {} cutout\n", layer.name));
        out.push_str(&format!(
            "CSX = AddMaterial( CSX, '{}');\n",
            string2matlab(&layer_cutout)
        ));
        out.push_str(&format!(
            "CSX = SetMaterialProperty( CSX, '{}', 'Epsilon', {}, 'Mue', 1);\n",
            string2matlab(&layer_cutout),
            layer.epsilon_r
        ));
    }

    for poly in &layer.metal.polys {
        if !poly.is_hole {
            out.push_str("% copper\n");
        } else {
            out.push_str("% cutout\n");
        }
        let material = if poly.is_hole {
            layer_cutout.clone()
        } else {
            layer_material.clone()
        };
        export_edge(out, &poly.poly);
        let priority = PRIO_MATERIAL + poly.nesting_level;
        if _metal_3d {
            let mut thickness = layer.thickness;
            if odd {
                thickness = -thickness;
            }
            out.push_str(&format!(
                "CSX = AddLinPoly(CSX, '{}', {}, 2, {}, pgon, {});\n",
                string2matlab(&material),
                priority,
                layer.z0,
                thickness
            ));
        } else {
            out.push_str(&format!(
                "CSX = AddPolygon(CSX, '{}', {}, 2, {}, pgon);\n",
                string2matlab(&material),
                priority,
                layer.z0
            ));
        }
    }

    Ok(())
}

/// Export vias as cylinders.
fn export_vias(out: &mut String, pcb: &PCB) -> Result<(), String> {
    if !pcb.via.is_empty() {
        out.push_str("% via copper\n");
        out.push_str("CSX = AddMetal( CSX, 'via' );\n");
        for via in &pcb.via {
            out.push_str(&format!(
                "CSX = AddCylinder(CSX, 'via', {}, [ {} , {} , {} ], [ {} , {} , {} ], {});\n",
                PRIO_VIA,
                via.x, via.y, via.z0,
                via.x, via.y, via.z1,
                via.radius
            ));
        }
    }
    Ok(())
}

/// Export devices as structs.
fn export_devices(out: &mut String, pcb: &PCB) -> Result<(), String> {
    out.push_str("% devices\n");
    out.push_str("CSX.HyperLynxDevice = {};\n");
    for device in &pcb.device {
        out.push_str(&format!(
            "CSX.HyperLynxDevice{{end+1}} = struct('name', {}, 'ref', {}",
            string2matlab(&device.name),
            string2matlab(&device.ref_)
        ));
        if device.value_type == "float" {
            out.push_str(&format!(", 'value', {}", device.value_float));
        } else if device.value_type == "string" {
            out.push_str(&format!(", 'value', {}", string2matlab(&device.value_string)));
        }
        out.push_str(&format!(
            ", 'layer_name', {});\n",
            string2matlab(&device.layer_name)
        ));
    }
    Ok(())
}

/// Export pins as ports.
fn export_ports(out: &mut String, pcb: &PCB) -> Result<(), String> {
    out.push_str("% ports\n");
    out.push_str("CSX.HyperLynxPort = {};\n");
    for pin in &pcb.pin {
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

        let stackup = &pcb.stackup;
        let dbottom = (pin.z0 - stackup.last().map_or(0.0, |l| l.z0)).abs();
        let dtop = (pin.z1 - stackup.first().map_or(0.0, |l| l.z1)).abs();
        let on_top = dtop <= dbottom;

        out.push_str(&format!(
            "CSX.HyperLynxPort{{end+1}} = struct('ref', {}, 'xc', {}, 'yc', {}, 'z', {}, 'x1', {}, 'y1', {}, 'x2', {}, 'y2', {}, 'position', {}, 'layer_name', {});\n",
            string2matlab(&pin.ref_),
            pin.x,
            pin.y,
            pin.z0,
            x_min,
            y_min,
            x_max,
            y_max,
            if on_top { "'top'" } else { "'bottom'" },
            string2matlab(&pin.layer_name)
        ));
    }
    Ok(())
}
