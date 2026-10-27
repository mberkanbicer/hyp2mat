//! CSXCAD export — port of `hyp2mat/lib/csxcad.cc`.
//!
//! Two representations are produced from the same model:
//!
//! * [`write_csxcad`] writes the native MATLAB/Octave script that builds a
//!   CSXCAD structure (`AddHyperLynxDielectric`, `AddPolygon`, ...). The output
//!   is byte-compatible with the native writer, including C++ default
//!   floating point formatting (6 significant digits).
//! * [`write_csxcad_xml`] writes the equivalent CSXCAD `ContinuousStructure`
//!   XML document. The native tool emits a script; the XML representation is
//!   the serialized form of the same model (same material names, priorities,
//!   polygon vertices and grid lines) and is validated by parsing it back with
//!   `csxcad-core`.
//!
//! Ports and devices have no CSXCAD XML representation; they are carried by
//! the MATLAB script as `CSX.HyperLynxDevice`/`CSX.HyperLynxPort` metadata.

use crate::model::{Bounds, FloatPolygon, FloatPolygons, Layer, PCB};

/// CSXCAD export priorities (matching the C++ implementation).
const PRIO_DIELECTRIC: i32 = 100; // FR4 dielectric
const PRIO_MATERIAL: i32 = 200; // copper
const PRIO_VIA: i32 = 300; // via metal

/// Options of the native `--pcb-outline`, `--lossy-copper` and `--metal-3d`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsxcadOptions {
    pub pcb_outline: bool,
    pub lossy_copper: bool,
    pub metal_3d: bool,
}

/// Format a float like C++ `operator<<(std::ostream&, double)` (`%g`, 6
/// significant digits, trailing zeros removed).
pub use crate::format::format_cpp_float;

/// Compute the bounding box from PCB data (native `PCB::GetBounds`).
pub fn compute_bounds(pcb: &PCB) -> Bounds {
    crate::pcb::get_bounds(pcb)
}

/// Quote a string using MATLAB/Octave conventions.
fn string2matlab(value: &str) -> String {
    let mut out = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            out.push('\'');
        }
        if ch == '%' {
            out.push('%');
        }
        out.push(ch);
    }
    out.push('\'');
    out
}

fn contains_polygon(polygons: &FloatPolygons) -> bool {
    polygons.polys.iter().any(|p| !p.is_hole)
}

fn contains_hole(polygons: &FloatPolygons) -> bool {
    polygons.polys.iter().any(|p| p.is_hole)
}

fn export_edge(out: &mut String, edge: &FloatPolygon) {
    out.push_str("pgon = [];\n");
    for point in &edge.points {
        out.push_str(&format!(
            "pgon(:, end+1) = [{};{}];\n",
            format_cpp_float(point.x),
            format_cpp_float(point.y)
        ));
    }
}

/// Write the native MATLAB script representation of the PCB.
pub fn write_csxcad(pcb: &PCB, options: &CsxcadOptions) -> Result<String, String> {
    let mut out = String::new();
    let bounds = compute_bounds(pcb);

    export_board(&mut out, pcb, &bounds, options.pcb_outline)?;

    out.push_str("% copper\n");
    let mut odd = false;
    for layer in &pcb.stackup {
        if layer.layer_type == "dielectric" {
            odd = !odd;
        }
        export_layer(&mut out, layer, options, odd)?;
    }

    export_vias(&mut out, pcb);
    export_devices(&mut out, pcb);
    export_ports(&mut out, pcb);

    out.push_str("%not truncated\n");
    Ok(out)
}

fn export_board(
    out: &mut String,
    pcb: &PCB,
    bounds: &Bounds,
    pcb_outline: bool,
) -> Result<(), String> {
    out.push_str("function CSX = pcb(CSX)\n");
    out.push_str("% matlab script created by hyp2mat\n");
    out.push_str("% create minimal mesh\n");
    out.push_str("mesh = {};\n");
    out.push_str(&format!(
        "mesh.x = [{} {}];\n",
        format_cpp_float(bounds.x0),
        format_cpp_float(bounds.x1)
    ));
    out.push_str(&format!(
        "mesh.y = [{} {}];\n",
        format_cpp_float(bounds.y0),
        format_cpp_float(bounds.y1)
    ));
    out.push_str(&format!(
        "mesh.z = [{} {}];\n",
        format_cpp_float(bounds.z0),
        format_cpp_float(bounds.z1)
    ));
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
                    layer.name,
                    format_cpp_float(layer.epsilon_r),
                    format_cpp_float(layer.loss_tangent)
                ));
                out.push_str(&format!("% board outline, layer {}\n", layer.name));
                let priority = PRIO_DIELECTRIC + poly.nesting_level;
                if pcb_outline {
                    export_edge(out, &poly.poly);
                    out.push_str(&format!(
                        "CSX = AddLinPoly(CSX, 'Dielectric_{}', {}, 2, {}, pgon, {});\n",
                        layer.name,
                        priority,
                        format_cpp_float(layer.z0),
                        format_cpp_float(layer.z1 - layer.z0)
                    ));
                } else {
                    out.push_str(&format!(
                        "CSX = AddBox(CSX, 'Dielectric_{}', {}, [ {}, {}, {}], [ {}, {}, {}]  );\n",
                        layer.name,
                        priority,
                        format_cpp_float(bounds.x0),
                        format_cpp_float(bounds.y0),
                        format_cpp_float(layer.z0),
                        format_cpp_float(bounds.x1),
                        format_cpp_float(bounds.y1),
                        format_cpp_float(layer.z1)
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
                format_cpp_float(bounds.z0),
                format_cpp_float(bounds.z1 - bounds.z0)
            ));
        }
    }
    Ok(())
}

fn export_layer(
    out: &mut String,
    layer: &Layer,
    options: &CsxcadOptions,
    odd: bool,
) -> Result<(), String> {
    let layer_material = format!("{}_copper", layer.name);
    let layer_cutout = format!("{}_cutout", layer.name);

    if contains_polygon(&layer.metal) {
        out.push_str(&format!("% create layer {} material\n", layer.name));
        let mut copper_resistivity = 0.0;
        if options.lossy_copper && layer.bulk_resistivity > 0.0 {
            copper_resistivity = layer.bulk_resistivity;
        }
        if options.metal_3d {
            out.push_str(&format!(
                "CSX = AddHyperLynxMetal3D(CSX, '{}', {}, {});\n",
                layer_material,
                format_cpp_float(copper_resistivity),
                format_cpp_float(layer.thickness)
            ));
        } else {
            out.push_str(&format!(
                "CSX = AddHyperLynxMetal2D(CSX, '{}', {}, {});\n",
                layer_material,
                format_cpp_float(copper_resistivity),
                format_cpp_float(layer.thickness)
            ));
        }
    }

    if contains_hole(&layer.metal) {
        out.push_str(&format!("% create layer {} cutout\n", layer.name));
        out.push_str(&format!("CSX = AddMaterial( CSX, '{}');\n", layer_cutout));
        out.push_str(&format!(
            "CSX = SetMaterialProperty( CSX, '{}', 'Epsilon', {}, 'Mue', 1);\n",
            layer_cutout,
            format_cpp_float(layer.epsilon_r)
        ));
    }

    for poly in &layer.metal.polys {
        let material = if poly.is_hole {
            out.push_str("% cutout\n");
            layer_cutout.clone()
        } else {
            out.push_str("% copper\n");
            layer_material.clone()
        };
        export_edge(out, &poly.poly);
        let priority = PRIO_MATERIAL + poly.nesting_level;
        if options.metal_3d {
            let mut thickness = layer.thickness;
            if odd {
                thickness = -thickness;
            }
            out.push_str(&format!(
                "CSX = AddLinPoly(CSX, '{}', {}, 2, {}, pgon, {});\n",
                material,
                priority,
                format_cpp_float(layer.z0),
                format_cpp_float(thickness)
            ));
        } else {
            out.push_str(&format!(
                "CSX = AddPolygon(CSX, '{}', {}, 2, {}, pgon);\n",
                material,
                priority,
                format_cpp_float(layer.z0)
            ));
        }
    }
    Ok(())
}

fn export_vias(out: &mut String, pcb: &PCB) {
    if pcb.via.is_empty() {
        return;
    }
    out.push_str("% via copper\n");
    out.push_str("CSX = AddMetal( CSX, 'via' );\n");
    for via in &pcb.via {
        out.push_str(&format!(
            "CSX = AddCylinder(CSX, 'via', {}, [ {} , {} , {} ], [ {} , {} , {} ], {});\n",
            PRIO_VIA,
            format_cpp_float(via.x),
            format_cpp_float(via.y),
            format_cpp_float(via.z0),
            format_cpp_float(via.x),
            format_cpp_float(via.y),
            format_cpp_float(via.z1),
            format_cpp_float(via.radius)
        ));
    }
}

fn export_devices(out: &mut String, pcb: &PCB) {
    out.push_str("% devices\n");
    out.push_str("CSX.HyperLynxDevice = {};\n");
    for device in &pcb.device {
        out.push_str(&format!(
            "CSX.HyperLynxDevice{{end+1}} = struct('name', {}, 'ref', {}",
            string2matlab(&device.name),
            string2matlab(&device.ref_)
        ));
        if device.value_type == "float" {
            out.push_str(&format!(
                ", 'value', {}",
                format_cpp_float(device.value_float)
            ));
        } else if device.value_type == "string" {
            out.push_str(&format!(
                ", 'value', {}",
                string2matlab(&device.value_string)
            ));
        }
        out.push_str(&format!(
            ", 'layer_name', {});\n",
            string2matlab(&device.layer_name)
        ));
    }
}

fn export_ports(out: &mut String, pcb: &PCB) {
    out.push_str("% ports\n");
    out.push_str("CSX.HyperLynxPort = {};\n");
    for pin in &pcb.pin {
        let mut x_max = 0.0;
        let mut y_max = 0.0;
        let mut x_min = 0.0;
        let mut y_min = 0.0;
        let mut first = true;
        for point in &pin.metal.points {
            if point.x > x_max || first {
                x_max = point.x;
            }
            if point.y > y_max || first {
                y_max = point.y;
            }
            if point.x < x_min || first {
                x_min = point.x;
            }
            if point.y < y_min || first {
                y_min = point.y;
            }
            first = false;
        }

        let (bottom, top) = match (pcb.stackup.last(), pcb.stackup.first()) {
            (Some(bottom), Some(top)) => (bottom.z0, top.z1),
            _ => (0.0, 0.0),
        };
        let dbottom = (pin.z0 - bottom).abs();
        let dtop = (pin.z1 - top).abs();
        let on_top = dtop <= dbottom;

        out.push_str(&format!(
            "CSX.HyperLynxPort{{end+1}} = struct('ref', {}, 'xc', {}, 'yc', {}, 'z', {}, 'x1', {}, 'y1', {}, 'x2', {}, 'y2', {}, 'position', {}, 'layer_name', {});\n",
            string2matlab(&pin.ref_),
            format_cpp_float(pin.x),
            format_cpp_float(pin.y),
            format_cpp_float(pin.z0),
            format_cpp_float(x_min),
            format_cpp_float(y_min),
            format_cpp_float(x_max),
            format_cpp_float(y_max),
            if on_top { "'top'" } else { "'bottom'" },
            string2matlab(&pin.layer_name)
        ));
    }
}

/// Write the PCB as a CSXCAD `ContinuousStructure` XML document.
///
/// The document carries the same materials, priorities, polygon vertices and
/// grid lines as the MATLAB script. `HyperLynxDevice`/`HyperLynxPort` metadata
/// only exists in the script form.
pub fn write_csxcad_xml(pcb: &PCB, options: &CsxcadOptions) -> Result<String, String> {
    use csxcad_core::grid::RectGrid;
    use csxcad_core::material::MaterialField;
    use csxcad_core::model::{Primitive, Property, PropertyType, Structure};

    let bounds = compute_bounds(pcb);
    let mut structure = Structure::new();
    {
        let grid: &mut RectGrid = structure.grid_mut();
        for (direction, (start, end)) in [
            (0usize, (bounds.x0, bounds.x1)),
            (1, (bounds.y0, bounds.y1)),
            (2, (bounds.z0, bounds.z1)),
        ] {
            grid.add_line(direction, start);
            if end != start {
                grid.add_line(direction, end);
            }
        }
        grid.sort(0);
        grid.sort(1);
        grid.sort(2);
    }

    let add_material = |structure: &mut Structure, name: &str, epsilon: f64| {
        let mut property = Property::new(name, PropertyType::Material);
        if let Some(material) = property.material_mut() {
            material.set_value(MaterialField::Epsilon, 0, epsilon);
            material.set_value(MaterialField::Mue, 0, 1.0);
        }
        structure.add_property(property)
    };
    let add_metal = |structure: &mut Structure, name: &str, conductivity: Option<f64>| {
        if let Some(conductivity) = conductivity {
            let mut property = Property::new(name, PropertyType::Material);
            if let Some(material) = property.material_mut() {
                material.set_value(MaterialField::Kappa, 0, conductivity);
                material.set_value(MaterialField::Mue, 0, 1.0);
            }
            structure.add_property(property)
        } else {
            structure.add_property(Property::new(name, PropertyType::Metal))
        }
    };
    let add_polygon = |structure: &mut Structure,
                       property_id,
                       polygon: &FloatPolygon,
                       z0: f64,
                       thickness: Option<f64>,
                       priority: i32|
     -> Result<(), String> {
        let mut primitive = if let Some(thickness) = thickness {
            Primitive::lin_poly(thickness)
        } else {
            Primitive::polygon()
        };
        primitive.set_priority(priority);
        {
            let shape = primitive
                .polygon_shape_mut()
                .ok_or("failed to create polygon primitive")?;
            shape.set_normal_direction(2);
            shape.set_elevation(z0);
            for point in &polygon.points {
                shape.add_vertex([point.x, point.y]);
            }
        }
        structure
            .add_primitive(property_id, primitive)
            .map_err(|e| e.to_string())?;
        Ok(())
    };

    // Board dielectrics and cutouts.
    if contains_polygon(&pcb.board) {
        for poly in &pcb.board.polys {
            if poly.is_hole {
                continue;
            }
            for layer in pcb.stackup.iter().rev() {
                if layer.layer_type != "dielectric" {
                    continue;
                }
                let name = format!("Dielectric_{}", layer.name);
                let property = add_material(&mut structure, &name, layer.epsilon_r);
                let priority = PRIO_DIELECTRIC + poly.nesting_level;
                if options.pcb_outline {
                    add_polygon(
                        &mut structure,
                        property,
                        &poly.poly,
                        layer.z0,
                        Some(layer.z1 - layer.z0),
                        priority,
                    )?;
                } else {
                    let mut primitive = Primitive::box_primitive(
                        [bounds.x0, bounds.y0, layer.z0],
                        [bounds.x1, bounds.y1, layer.z1],
                    );
                    primitive.set_priority(priority);
                    structure
                        .add_primitive(property, primitive)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    if options.pcb_outline && contains_hole(&pcb.board) {
        let drill = add_material(&mut structure, "Drill", 1.0);
        for poly in &pcb.board.polys {
            if !poly.is_hole {
                continue;
            }
            add_polygon(
                &mut structure,
                drill,
                &poly.poly,
                bounds.z0,
                Some(bounds.z1 - bounds.z0),
                PRIO_DIELECTRIC + poly.nesting_level,
            )?;
        }
    }

    // Copper layers.
    let mut odd = false;
    for layer in &pcb.stackup {
        if layer.layer_type == "dielectric" {
            odd = !odd;
        }
        let layer_material = format!("{}_copper", layer.name);
        let layer_cutout = format!("{}_cutout", layer.name);

        let copper_property = if contains_polygon(&layer.metal) {
            let conductivity = if options.lossy_copper && layer.bulk_resistivity > 0.0 {
                Some(1.0 / layer.bulk_resistivity)
            } else {
                None
            };
            Some(add_metal(&mut structure, &layer_material, conductivity))
        } else {
            None
        };

        let cutout_property = if contains_hole(&layer.metal) {
            Some(add_material(&mut structure, &layer_cutout, layer.epsilon_r))
        } else {
            None
        };

        for poly in &layer.metal.polys {
            let property = if poly.is_hole {
                cutout_property
            } else {
                copper_property
            };
            let Some(property) = property else {
                continue;
            };
            let priority = PRIO_MATERIAL + poly.nesting_level;
            if options.metal_3d {
                let mut thickness = layer.thickness;
                if odd {
                    thickness = -thickness;
                }
                add_polygon(
                    &mut structure,
                    property,
                    &poly.poly,
                    layer.z0,
                    Some(thickness),
                    priority,
                )?;
            } else {
                add_polygon(
                    &mut structure,
                    property,
                    &poly.poly,
                    layer.z0,
                    None,
                    priority,
                )?;
            }
        }
    }

    // Vias.
    if !pcb.via.is_empty() {
        let via = add_metal(&mut structure, "via", None);
        for via_model in &pcb.via {
            let mut primitive = Primitive::cylinder(
                [via_model.x, via_model.y, via_model.z0],
                [via_model.x, via_model.y, via_model.z1],
                via_model.radius,
            );
            primitive.set_priority(PRIO_VIA);
            structure
                .add_primitive(via, primitive)
                .map_err(|e| e.to_string())?;
        }
    }

    structure.to_xml_string().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FloatPoint, FloatPoly, Via};

    fn square(x: f64, y: f64, w: f64) -> FloatPolygon {
        FloatPolygon {
            points: vec![
                FloatPoint { x, y },
                FloatPoint { x: x + w, y: y + w },
                FloatPoint { x, y: y + w },
            ],
        }
    }

    fn sample_pcb() -> PCB {
        let board = FloatPolygons {
            polys: vec![FloatPoly {
                poly: crate::model::FloatPolygon {
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
        let metal = FloatPolygons {
            polys: vec![FloatPoly {
                poly: square(0.002, 0.002, 0.004),
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
                Layer {
                    name: "Bottom".into(),
                    layer_type: "signal".into(),
                    thickness: 3.5e-5,
                    z0: 0.0,
                    z1: 0.0,
                    epsilon_r: 1.0,
                    loss_tangent: 0.0,
                    bulk_resistivity: 1.724e-8,
                    metal,
                },
            ],
            via: vec![Via {
                x: 0.01,
                y: 0.005,
                z0: 0.0,
                z1: 1e-3,
                radius: 2e-4,
            }],
            via_plating_thickness: 2e-5,
            device: Vec::new(),
            pin: Vec::new(),
        }
    }

    #[test]
    fn xml_lossy_copper_and_metal_3d() {
        use csxcad_core::material::MaterialField;
        use csxcad_core::model::Structure;

        let pcb = sample_pcb();
        let xml = write_csxcad_xml(
            &pcb,
            &CsxcadOptions {
                lossy_copper: true,
                metal_3d: true,
                ..CsxcadOptions::default()
            },
        )
        .unwrap();
        let structure = Structure::from_xml_str(&xml).expect("parse XML");

        // Lossy copper is a Material with Kappa = 1 / resistivity.
        let copper = (0..structure.property_count())
            .map(|index| structure.property_by_index(index).unwrap())
            .find(|property| property.name() == "Top_copper")
            .expect("Top_copper property");
        let material = copper.material().expect("material copper");
        let kappa = material
            .value(MaterialField::Kappa, 0, None)
            .unwrap()
            .unwrap();
        assert!((kappa - 1.0 / 1.724e-8).abs() / kappa < 1e-12);

        // 3D copper is a LinPoly; the second layer is pushed down (negative
        // extrusion), matching the native odd/even rule.
        let top = copper.primitive(0).unwrap();
        assert!(top.polygon_shape().is_some());
        assert_eq!(
            top.polygon_shape().unwrap().extrusion_length(None).unwrap(),
            Some(3.5e-5)
        );
        let bottom = (0..structure.property_count())
            .map(|index| structure.property_by_index(index).unwrap())
            .find(|property| property.name() == "Bottom_copper")
            .expect("Bottom_copper property");
        let bottom_primitive = bottom.primitive(0).unwrap();
        assert_eq!(
            bottom_primitive
                .polygon_shape()
                .unwrap()
                .extrusion_length(None)
                .unwrap(),
            Some(-3.5e-5)
        );
    }

    #[test]
    fn cpp_float_matches_native_formatting() {
        assert_eq!(format_cpp_float(0.0508), "0.0508");
        assert_eq!(format_cpp_float(0.00151638), "0.00151638");
        assert_eq!(format_cpp_float(0.00001778), "1.778e-05");
        assert_eq!(format_cpp_float(4.8), "4.8");
        assert_eq!(format_cpp_float(0.02), "0.02");
        assert_eq!(format_cpp_float(100.0), "100");
        assert_eq!(format_cpp_float(0.0), "0");
        assert_eq!(format_cpp_float(1e-7), "1e-07");
        assert_eq!(format_cpp_float(-0.001), "-0.001");
        assert_eq!(format_cpp_float(1234567.0), "1.23457e+06");
    }

    #[test]
    fn matlab_script_has_native_structure() {
        let pcb = sample_pcb();
        let script = write_csxcad(&pcb, &CsxcadOptions::default()).unwrap();
        assert!(script.starts_with("function CSX = pcb(CSX)\n"));
        assert!(script.contains("mesh.x = [0 0.02];"));
        assert!(script.contains("CSX = AddHyperLynxDielectric(CSX, 'Dielectric_DL01', 4.4, 0.02);"));
        assert!(script.contains(
            "CSX = AddBox(CSX, 'Dielectric_DL01', 100, [ 0, 0, 0], [ 0.02, 0.01, 0.001]  );"
        ));
        assert!(script.contains("CSX = AddHyperLynxMetal2D(CSX, 'Top_copper', 0, 3.5e-05);"));
        assert!(script.contains("CSX = AddPolygon(CSX, 'Top_copper', 200, 2, 0.001, pgon);"));
        assert!(script.contains("CSX = AddCylinder(CSX, 'via', 300, [ 0.01 , 0.005 , 0 ], [ 0.01 , 0.005 , 0.001 ], 0.0002);"));
        assert!(script.ends_with("%not truncated\n"));
    }

    #[test]
    fn matlab_script_pcb_outline_uses_linpoly() {
        let pcb = sample_pcb();
        let script = write_csxcad(
            &pcb,
            &CsxcadOptions {
                pcb_outline: true,
                ..CsxcadOptions::default()
            },
        )
        .unwrap();
        assert!(
            script.contains("CSX = AddLinPoly(CSX, 'Dielectric_DL01', 100, 2, 0, pgon, 0.001);")
        );
        assert!(!script.contains("CSX = AddBox(CSX, 'Dielectric_DL01'"));
    }

    #[test]
    fn matlab_script_lossy_copper_uses_resistivity() {
        let pcb = sample_pcb();
        let script = write_csxcad(
            &pcb,
            &CsxcadOptions {
                lossy_copper: true,
                ..CsxcadOptions::default()
            },
        )
        .unwrap();
        assert!(script.contains("AddHyperLynxMetal2D(CSX, 'Top_copper', 1.724e-08, 3.5e-05)"));
    }

    #[test]
    fn matlab_script_metal_3d_alternates_sign() {
        let pcb = sample_pcb();
        let script = write_csxcad(
            &pcb,
            &CsxcadOptions {
                metal_3d: true,
                ..CsxcadOptions::default()
            },
        )
        .unwrap();
        // After the first dielectric the layer thickness is negated.
        assert!(script.contains("AddLinPoly(CSX, 'Bottom_copper', 200, 2, 0, pgon, -3.5e-05);"));
    }

    #[test]
    fn xml_round_trips_through_csxcad_core() {
        use csxcad_core::material::MaterialField;
        use csxcad_core::model::Structure;

        let pcb = sample_pcb();
        let xml = write_csxcad_xml(
            &pcb,
            &CsxcadOptions {
                pcb_outline: true,
                ..CsxcadOptions::default()
            },
        )
        .unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\""));
        let mut structure = Structure::from_xml_str(&xml).expect("parse generated CSXCAD XML");

        let grid = structure.grid_mut();
        assert_eq!(grid.lines(0), Some(&[0.0, 0.02][..]));
        assert_eq!(grid.lines(1), Some(&[0.0, 0.01][..]));
        assert_eq!(grid.lines(2), Some(&[0.0, 0.001][..]));

        let names: Vec<&str> = (0..structure.property_count())
            .map(|i| structure.property_by_index(i).unwrap().name())
            .collect();
        assert_eq!(
            names,
            vec!["Dielectric_DL01", "Top_copper", "Bottom_copper", "via"]
        );

        let dielectric = structure.property_by_index(0).unwrap();
        let material = dielectric.material().expect("material property");
        assert!(
            (material
                .value(MaterialField::Epsilon, 0, None)
                .unwrap()
                .unwrap()
                - 4.4)
                .abs()
                < 1e-12
        );
        assert_eq!(dielectric.primitive_count(), 1);
        let primitive = dielectric.primitive(0).unwrap();
        assert_eq!(primitive.priority(), PRIO_DIELECTRIC);
        let shape = primitive.polygon_shape().expect("polygon primitive");
        assert_eq!(shape.normal_direction(), 2);
        assert_eq!(shape.extrusion_length(None).unwrap(), Some(0.001));
        assert_eq!(shape.vertex_count(), 4);

        let copper = structure.property_by_index(1).unwrap();
        assert_eq!(copper.primitive_count(), 1);
        let primitive = copper.primitive(0).unwrap();
        assert_eq!(primitive.priority(), PRIO_MATERIAL);
        let shape = primitive.polygon_shape().expect("polygon primitive");
        assert_eq!(shape.vertex_count(), 3);
        assert_eq!(shape.vertex(0, None).unwrap(), Some([0.002, 0.002]));

        let via = structure.property_by_index(3).unwrap();
        assert_eq!(via.primitive_count(), 1);
        let primitive = via.primitive(0).unwrap();
        assert_eq!(primitive.priority(), PRIO_VIA);
        let cylinder = primitive.cylinder_shape().expect("cylinder primitive");
        assert!((cylinder.radius(None).unwrap() - 0.0002).abs() < 1e-15);
    }
}
