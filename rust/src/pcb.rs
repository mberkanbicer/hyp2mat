//! Board conversion — port of `hyp2mat/lib/hyperlynx.cc`,
//! `hyp2mat/lib/copper.cc`, `hyp2mat/lib/crop.cc` and the top-level helpers in
//! `hyp2mat/lib/pcb.cc`.
//!
//! `read_hyp` turns the parsed HyperLynx model into the generic [`PCB`]
//! representation used by the CSXCAD, PDF and QucsStudio writers.

use crate::hypfile::{GeomSettings, HypFile, HypPolygon, HypPolygonType, LayerType, Padstack};
use crate::model::{
    Bounds, Device, FloatPoint, FloatPoly, FloatPolygon, FloatPolygons, Layer, Pin, Via, PCB,
};
use crate::polygon::{is_clockwise, reverse, Polygon, PolygonOps};

/// Conversion options, mirroring the native command line and library setters.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// Layers to import; empty means all layers.
    pub layers: Vec<String>,
    /// Nets to import; empty means all nets.
    pub nets: Vec<String>,
    /// Copy polygons without joining adjacent/overlapping copper.
    pub raw: bool,
    /// Coordinates resolution in metres.
    pub grid: f64,
    /// Maximum arc approximation error in metres.
    pub arc_precision: f64,
    /// Default trace-to-plane clearance in metres.
    pub clearance: f64,
    /// Layers to flood with copper (`"plane_layers"` floods all planes).
    pub flood_layers: Vec<String>,
    /// Dielectric constant override.
    pub epsilon_r: Option<f64>,
    /// Loss tangent override.
    pub loss_tangent: Option<f64>,
    /// Copper bulk resistivity override.
    pub bulk_resistivity: Option<f64>,
    /// Crop rectangle. `None` means no cropping.
    pub bounds: Option<Bounds>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            layers: Vec::new(),
            nets: Vec::new(),
            raw: false,
            grid: crate::polygon::DEFAULT_GRID,
            arc_precision: 0.0,
            clearance: 0.0002,
            flood_layers: Vec::new(),
            epsilon_r: None,
            loss_tangent: None,
            bulk_resistivity: None,
            bounds: None,
        }
    }
}

impl Options {
    fn geometry(&self) -> GeomSettings {
        GeomSettings {
            grid: self.grid,
            arc_precision: self.arc_precision,
        }
    }
}

/// A non-fatal conversion message (native stderr output).
pub type Warning = String;

/// Convert a parsed HyperLynx file to the generic PCB model.
pub fn read_hyp(hyp: &HypFile, options: &Options) -> Result<(PCB, Vec<Warning>), String> {
    let mut hyp = hyp.clone();
    hyp.geometry = options.geometry();
    if hyp.ended {
        hyp.flood_layers_with_copper(&options.flood_layers)?;
    }
    let mut converter = HyperLynx::new(&hyp, options);
    converter.read()
}

/// Bounds with the native "infinity" default.
fn infinite_bounds() -> Bounds {
    let max = f64::MAX;
    Bounds {
        x0: -max,
        y0: -max,
        z0: -max,
        x1: max,
        y1: max,
        z1: max,
    }
}

fn adjust_bounds(pcb: &PCB, bounds: Bounds) -> Bounds {
    let mut new_bounds = bounds;
    let size = get_bounds(pcb);
    if new_bounds.x0 < size.x0 {
        new_bounds.x0 = size.x0;
    }
    if new_bounds.x1 > size.x1 {
        new_bounds.x1 = size.x1;
    }
    if new_bounds.y0 < size.y0 {
        new_bounds.y0 = size.y0;
    }
    if new_bounds.y1 > size.y1 {
        new_bounds.y1 = size.y1;
    }
    if new_bounds.z0 < size.z0 {
        new_bounds.z0 = size.z0;
    }
    if new_bounds.z1 > size.z1 {
        new_bounds.z1 = size.z1;
    }
    new_bounds
}

/// Native `PCB::GetBounds`: board, layer copper, and vias. Note that the
/// search starts at the origin, so bounds always include (0, 0, 0).
pub fn get_bounds(pcb: &PCB) -> Bounds {
    let mut x_min = 0.0;
    let mut x_max = 0.0;
    let mut y_min = 0.0;
    let mut y_max = 0.0;

    let mut z_min = 0.0;
    let mut z_max = 0.0;
    if let Some(last) = pcb.stackup.last() {
        z_min = last.z0;
    }
    if let Some(first) = pcb.stackup.first() {
        z_max = first.z1;
    }
    for layer in &pcb.stackup {
        if layer.z0 < z_min {
            z_min = layer.z0;
        }
        if layer.z1 > z_max {
            z_max = layer.z1;
        }
    }

    let mut first_point = true;
    let mut update = |x: f64, y: f64, first: &mut bool| {
        if x > x_max || *first {
            x_max = x;
        }
        if y > y_max || *first {
            y_max = y;
        }
        if x < x_min || *first {
            x_min = x;
        }
        if y < y_min || *first {
            y_min = y;
        }
        *first = false;
    };

    for poly in &pcb.board.polys {
        for point in &poly.poly.points {
            update(point.x, point.y, &mut first_point);
        }
    }
    for layer in &pcb.stackup {
        for poly in &layer.metal.polys {
            for point in &poly.poly.points {
                update(point.x, point.y, &mut first_point);
            }
        }
    }
    for via in &pcb.via {
        update(via.x + via.radius, via.y + via.radius, &mut first_point);
        update(via.x - via.radius, via.y - via.radius, &mut first_point);
    }

    Bounds {
        x0: x_min,
        y0: y_min,
        x1: x_max,
        y1: y_max,
        z0: z_min,
        z1: z_max,
    }
}

/// Native `CropPolygon`: intersect with an axis-aligned rectangle.
fn crop_polygon(poly: &Polygon, bounds: Bounds, ops: &PolygonOps) -> Polygon {
    let mut result = ops.polygon();
    result.add_edge(&FloatPolygon {
        points: vec![
            FloatPoint {
                x: bounds.x0,
                y: bounds.y0,
            },
            FloatPoint {
                x: bounds.x1,
                y: bounds.y0,
            },
            FloatPoint {
                x: bounds.x1,
                y: bounds.y1,
            },
            FloatPoint {
                x: bounds.x0,
                y: bounds.y1,
            },
        ],
    });
    result.intersection(poly);
    result
}

/// Native `CropLayers`: drop layers outside the z bounds.
fn crop_layers(pcb: &mut PCB, bounds: Bounds) {
    let mut new_stackup = Vec::new();
    for layer in &pcb.stackup {
        if layer.z0 > bounds.z1 && layer.z1 > bounds.z1 {
            continue;
        }
        if layer.z0 < bounds.z0 && layer.z1 < bounds.z0 {
            continue;
        }
        let mut new_layer = layer.clone();
        if new_layer.z1 > bounds.z1 {
            new_layer.z1 = bounds.z1;
        }
        if new_layer.z0 < bounds.z0 {
            new_layer.z0 = bounds.z0;
        }
        new_stackup.push(new_layer);
    }
    pcb.stackup = new_stackup;
}

fn float_polygon(polygon: &HypPolygon) -> FloatPolygon {
    FloatPolygon {
        points: polygon
            .vertex
            .iter()
            .map(|p| FloatPoint { x: p.x, y: p.y })
            .collect(),
    }
}

struct HyperLynx<'a> {
    hyp: &'a HypFile,
    options: &'a Options,
    ops: PolygonOps,
    board: Polygon,
    saved_bounds: Bounds,
    bounds: Bounds,
    layer_plane_separation: f64,
    net_plane_separation: f64,
    polygon_plane_separation: f64,
    warnings: Vec<Warning>,
}

impl<'a> HyperLynx<'a> {
    fn new(hyp: &'a HypFile, options: &'a Options) -> Self {
        let mut ops = PolygonOps::new();
        ops.set_grid(options.grid);
        ops.set_arc_precision(options.arc_precision);
        let bounds = options.bounds.unwrap_or_else(infinite_bounds);
        HyperLynx {
            hyp,
            options,
            ops,
            board: ops.polygon(),
            saved_bounds: bounds,
            bounds,
            layer_plane_separation: -1.0,
            net_plane_separation: -1.0,
            polygon_plane_separation: -1.0,
            warnings: Vec::new(),
        }
    }

    fn read(&mut self) -> Result<(PCB, Vec<Warning>), String> {
        let mut pcb = PCB::default();
        self.board = self.ops.polygon();

        self.copy_board(&mut pcb);
        self.copy_stackup(&mut pcb);
        self.bounds = adjust_bounds(&pcb, self.saved_bounds);
        self.copy_vias(&mut pcb);
        self.copy_via_plating(&mut pcb);
        self.copy_devices(&mut pcb);
        self.copy_pins(&mut pcb);
        self.bounds = adjust_bounds(&pcb, self.saved_bounds);
        crop_layers(&mut pcb, self.bounds);
        if self.bounds.z0 == self.bounds.z1 {
            self.warnings.push("warning: zero height board".to_string());
        }
        self.check_board_outline(&mut pcb);
        self.user_overrides(&mut pcb);
        Ok((pcb, self.warnings.clone()))
    }

    fn copy_board(&mut self, pcb: &mut PCB) {
        let mut outer_edge = true;
        for edge in &self.hyp.board.edge {
            let mut poly = self.ops.polygon();
            poly.add_edge(&float_polygon(edge));
            if outer_edge {
                self.board.union(&poly);
            } else {
                self.board.difference(&poly);
            }
            outer_edge = false;
        }
        pcb.board = self.board.result();
        self.bounds = adjust_bounds(pcb, self.bounds);
        self.board = crop_polygon(&self.board, self.bounds, &self.ops);
        pcb.board = self.board.result();
    }

    fn layer_wanted(&self, layer_name: &str) -> bool {
        self.options.layers.is_empty() || self.options.layers.iter().any(|l| l == layer_name)
    }

    fn net_wanted(&self, net_name: &str) -> bool {
        self.options.nets.is_empty() || self.options.nets.iter().any(|n| n == net_name)
    }

    fn copy_stackup(&mut self, pcb: &mut PCB) {
        for layer in &self.hyp.stackup {
            if self.layer_wanted(&layer.layer_name) {
                self.copy_layer(pcb, layer);
            }
        }
    }

    fn copy_layer(&mut self, pcb: &mut PCB, hyp_layer: &crate::hypfile::HypLayer) {
        let layer_type = match hyp_layer.layer_type {
            LayerType::Signal => "signal",
            LayerType::Dielectric => "dielectric",
            LayerType::Plane => "plane",
        };
        let mut layer = Layer {
            name: hyp_layer.layer_name.clone(),
            layer_type: layer_type.to_string(),
            thickness: hyp_layer.thickness,
            z0: hyp_layer.z0,
            z1: hyp_layer.z1,
            epsilon_r: hyp_layer.epsilon_r,
            loss_tangent: hyp_layer.loss_tangent,
            bulk_resistivity: hyp_layer.bulk_resistivity,
            metal: FloatPolygons { polys: Vec::new() },
        };

        if hyp_layer.layer_type != LayerType::Dielectric {
            self.layer_plane_separation = self.hyp.board.plane_separation;
            if self.options.clearance >= 0.0 {
                self.layer_plane_separation = self.options.clearance;
            }
            if hyp_layer.plane_separation >= 0.0 {
                self.layer_plane_separation = hyp_layer.plane_separation;
            }

            let mut raw_polygons = FloatPolygons { polys: Vec::new() };
            let layer_copper = self.copy_copper(&layer, &mut raw_polygons);
            if self.options.raw {
                layer.metal = raw_polygons;
            } else {
                layer.metal = layer_copper.result();
            }
        }

        pcb.stackup.push(layer);
    }

    fn copy_copper(&mut self, layer: &Layer, raw_polygons: &mut FloatPolygons) -> Polygon {
        let net_count = self.hyp.net.len();
        let mut net_pour: Vec<Polygon> = (0..net_count).map(|_| self.ops.polygon()).collect();
        let mut net_plane: Vec<Polygon> = (0..net_count).map(|_| self.ops.polygon()).collect();
        let mut net_copper: Vec<Polygon> = (0..net_count).map(|_| self.ops.polygon()).collect();
        let mut net_pads: Vec<Polygon> = (0..net_count).map(|_| self.ops.polygon()).collect();
        let mut net_antipads: Vec<Polygon> = (0..net_count).map(|_| self.ops.polygon()).collect();

        let wanted: Vec<bool> = self
            .hyp
            .net
            .iter()
            .map(|net| self.net_wanted(&net.net_name))
            .collect();

        for (i, net) in self.hyp.net.iter().enumerate() {
            if !wanted[i] {
                continue;
            }
            net_pour[i] = self.copy_net(layer, net, HypPolygonType::Pour, raw_polygons);
            net_copper[i] = self.copy_net(layer, net, HypPolygonType::Copper, raw_polygons);
            net_plane[i] = self.copy_net(layer, net, HypPolygonType::Plane, raw_polygons);
            net_pads[i] = self.copy_net(layer, net, HypPolygonType::Pad, raw_polygons);
            net_antipads[i] = self.copy_net(layer, net, HypPolygonType::Antipad, raw_polygons);
        }

        let mut layer_copper = self.ops.polygon();
        for i in 0..net_count {
            if !wanted[i] {
                continue;
            }
            layer_copper.union(&net_pour[i]);
            layer_copper.union(&net_plane[i]);
        }
        for i in 0..net_count {
            if !wanted[i] {
                continue;
            }
            layer_copper.difference(&net_antipads[i]);
        }
        for i in 0..net_count {
            if !wanted[i] {
                continue;
            }
            layer_copper.union(&net_copper[i]);
            layer_copper.union(&net_pads[i]);
        }

        if !self.board.is_empty() {
            layer_copper.intersection(&self.board);
            layer_copper = crop_polygon(&layer_copper, self.bounds, &self.ops);
        }
        layer_copper
    }

    fn copy_net(
        &mut self,
        layer: &Layer,
        net: &crate::hypfile::Net,
        poly_type: HypPolygonType,
        raw_polygons: &mut FloatPolygons,
    ) -> Polygon {
        self.net_plane_separation = self.layer_plane_separation;
        if net.plane_separation >= 0.0 {
            self.net_plane_separation = net.plane_separation;
        }
        let mut net_copper = self.ops.polygon();
        for edges in net.metal.values() {
            if edges.is_empty() {
                continue;
            }
            if edges[0].layer_name != layer.name {
                continue;
            }
            if edges[0].polygon_type != poly_type {
                continue;
            }
            let poly_copper = self.copy_polygon(edges, raw_polygons);
            net_copper.union(&poly_copper);
        }
        net_copper
    }

    fn copy_polygon(&mut self, edges: &[HypPolygon], raw_polygons: &mut FloatPolygons) -> Polygon {
        let mut poly = self.ops.polygon();
        if edges.is_empty() {
            return poly;
        }
        for edge in edges {
            let mut polygon = float_polygon(edge);
            if is_clockwise(&polygon) != edge.positive {
                reverse(&mut polygon);
            }
            raw_polygons.polys.push(FloatPoly {
                poly: polygon.clone(),
                is_hole: !edge.positive,
                nesting_level: 0,
            });
            if edge.positive {
                poly.add_edge(&polygon);
            } else {
                poly.add_hole(&polygon);
            }
        }

        let width = edges[0].width;
        poly.simplify();
        poly.offset(width / 2.0);

        if edges[0].polygon_type == HypPolygonType::Plane {
            self.polygon_plane_separation = self.net_plane_separation;
            if edges[0].plane_separation >= 0.0 {
                self.polygon_plane_separation = edges[0].plane_separation;
            }
            let mask = self.plane_separation_mask(&edges[0].layer_name, &edges[0].net_name);
            poly.difference(&mask);
        }
        poly
    }

    /// Native `PlaneSeparation`. The `net_wanted` test in the original uses
    /// the *plane* net name instead of the masking net's name, so it never
    /// filters anything out when the plane net is wanted. Preserved.
    fn plane_separation_mask(&self, layer_name: &str, net_name: &str) -> Polygon {
        let mut mask = self.ops.polygon();
        if self.polygon_plane_separation < 0.0 {
            return mask;
        }
        for net in &self.hyp.net {
            if net.net_name == net_name {
                continue;
            }
            let net_wanted = self.net_wanted(net_name);
            if !net_wanted {
                continue;
            }
            for edges in net.metal.values() {
                if edges.is_empty() {
                    continue;
                }
                if edges[0].layer_name != layer_name {
                    continue;
                }
                let mut poly = self.ops.polygon();
                for edge in edges {
                    let polygon = float_polygon(edge);
                    if edge.positive {
                        poly.add_edge(&polygon);
                    } else {
                        poly.add_hole(&polygon);
                    }
                }
                let width = edges[0].width;
                let mut mask_plane_separation = self.layer_plane_separation;
                if net.plane_separation >= 0.0 {
                    mask_plane_separation = net.plane_separation;
                }
                if edges[0].plane_separation >= 0.0 {
                    mask_plane_separation = edges[0].plane_separation;
                }
                let biggest = if self.polygon_plane_separation > mask_plane_separation {
                    self.polygon_plane_separation
                } else {
                    mask_plane_separation
                };
                poly.offset(width / 2.0 + biggest);
                mask.union(&poly);
            }
        }
        mask
    }

    fn copy_vias(&mut self, pcb: &mut PCB) {
        for net in &self.hyp.net {
            if !self.net_wanted(&net.net_name) {
                continue;
            }
            for via in &net.via {
                if !self.board.is_empty()
                    && (via.x > self.bounds.x1
                        || via.x < self.bounds.x0
                        || via.y > self.bounds.y1
                        || via.y < self.bounds.y0)
                {
                    continue;
                }
                if via.z0 > self.bounds.z1 && via.z1 > self.bounds.z1 {
                    continue;
                }
                if via.z0 < self.bounds.z0 && via.z1 < self.bounds.z0 {
                    continue;
                }
                let mut z1 = via.z1;
                let mut z0 = via.z0;
                if z1 > self.bounds.z1 {
                    z1 = self.bounds.z1;
                }
                if z0 < self.bounds.z0 {
                    z0 = self.bounds.z0;
                }
                pcb.via.push(Via {
                    x: via.x,
                    y: via.y,
                    z0,
                    z1,
                    radius: via.radius,
                });
            }
        }
    }

    fn copy_via_plating(&mut self, pcb: &mut PCB) {
        for layer in &self.hyp.stackup {
            if layer.layer_type != LayerType::Dielectric {
                pcb.via_plating_thickness = layer.plating_thickness;
                break;
            }
        }
    }

    fn copy_devices(&mut self, pcb: &mut PCB) {
        for device in &self.hyp.device {
            let (value_type, value_float, value_string) = if device.value_float_set {
                ("float", device.value_float, String::new())
            } else if device.value_string_set {
                ("string", 0.0, device.value_string.clone())
            } else {
                ("none", 0.0, String::new())
            };
            pcb.device.push(Device {
                name: device.name.clone(),
                ref_: device.ref_.clone(),
                value_type: value_type.to_string(),
                value_float,
                value_string,
                layer_name: device.layer_name.clone(),
            });
        }
    }

    fn copy_pins(&mut self, pcb: &mut PCB) {
        for net in &self.hyp.net {
            if !self.net_wanted(&net.net_name) {
                continue;
            }
            for pin in &net.pin {
                for padstack in &self.hyp.padstack {
                    if padstack.padstack_name == pin.padstack_name {
                        self.copy_pin(pcb, pin, padstack);
                    }
                }
            }
        }
    }

    fn copy_pin(&mut self, pcb: &mut PCB, pin: &crate::hypfile::HypPin, padstack: &Padstack) {
        let x = self.ops.align_to_grid(pin.x);
        let y = self.ops.align_to_grid(pin.y);
        let mut new_pin = Pin {
            x,
            y,
            z0: 0.0,
            z1: 0.0,
            ref_: pin.pin_reference.clone(),
            layer_name: String::new(),
            metal: FloatPolygon { points: Vec::new() },
        };

        let pad = if padstack.pads.len() == 1 {
            padstack.pads[0].clone()
        } else {
            let Some(dot) = pin.pin_reference.find('.') else {
                return;
            };
            if dot == 0 {
                return;
            }
            let device_name = &pin.pin_reference[..dot];
            let mut layer_name = String::new();
            let mut device_found = false;
            for device in &pcb.device {
                if device.ref_ == device_name {
                    device_found = true;
                    layer_name = device.layer_name.clone();
                }
            }
            if !device_found {
                return;
            }
            match padstack.pads.iter().find(|p| p.layer_name == layer_name) {
                Some(pad) => pad.clone(),
                None => return,
            }
        };

        for layer in &pcb.stackup {
            if pad.layer_name == layer.name {
                new_pin.layer_name = layer.name.clone();
                new_pin.z0 = layer.z0;
                new_pin.z1 = layer.z1;
            }
        }

        let pad_poly = self.hyp.pad2poly(x, y, &pad);
        new_pin.metal.points = pad_poly
            .vertex
            .iter()
            .map(|p| FloatPoint {
                x: self.ops.align_to_grid(p.x),
                y: self.ops.align_to_grid(p.y),
            })
            .collect();
        pcb.pin.push(new_pin);
    }

    fn check_board_outline(&mut self, pcb: &mut PCB) {
        if !pcb.board.polys.is_empty() {
            return;
        }
        let bounds = get_bounds(pcb);
        if bounds.x0 != bounds.x1 && bounds.y0 != bounds.y1 {
            pcb.board.polys.push(FloatPoly {
                poly: FloatPolygon {
                    points: vec![
                        FloatPoint {
                            x: bounds.x0,
                            y: bounds.y0,
                        },
                        FloatPoint {
                            x: bounds.x0,
                            y: bounds.y1,
                        },
                        FloatPoint {
                            x: bounds.x1,
                            y: bounds.y1,
                        },
                        FloatPoint {
                            x: bounds.x1,
                            y: bounds.y0,
                        },
                    ],
                },
                is_hole: false,
                nesting_level: 0,
            });
        }
    }

    fn user_overrides(&mut self, pcb: &mut PCB) {
        for layer in pcb.stackup.iter_mut() {
            if let Some(epsilon_r) = self.options.epsilon_r {
                if layer.layer_type == "dielectric" {
                    layer.epsilon_r = epsilon_r;
                }
            }
            if let Some(loss_tangent) = self.options.loss_tangent {
                if layer.layer_type == "dielectric" {
                    layer.loss_tangent = loss_tangent;
                }
            }
            if let Some(bulk_resistivity) = self.options.bulk_resistivity {
                if layer.layer_type != "dielectric" {
                    layer.bulk_resistivity = bulk_resistivity;
                }
            }
        }

        // Outer copper layers are assumed to be in air.
        let len = pcb.stackup.len();
        for i in 0..len {
            if pcb.stackup[i].layer_type == "dielectric" {
                continue;
            }
            let upper_found = (0..i).any(|j| pcb.stackup[j].layer_type == "dielectric");
            let lower_found = (i..len).any(|j| pcb.stackup[j].layer_type == "dielectric");
            if !upper_found || !lower_found {
                pcb.stackup[i].epsilon_r = 1.0;
                pcb.stackup[i].loss_tangent = 0.0;
            }
        }
    }
}

/// Native `PCB::PrintSummary` output (stderr). Returns the formatted text.
pub fn print_summary(pcb: &PCB) -> String {
    let bounds = get_bounds(pcb);
    let mut out = String::new();
    let m_to_mm = 1000.0;
    out.push_str("board size:\n");
    out.push_str(&format!(
        " x = {:>8.2} : {:>8.2} mm\n",
        bounds.x0 * m_to_mm,
        bounds.x1 * m_to_mm
    ));
    out.push_str(&format!(
        " y = {:>8.2} : {:>8.2} mm\n",
        bounds.y0 * m_to_mm,
        bounds.y1 * m_to_mm
    ));
    out.push_str(&format!(
        " z = {:>8.2} : {:>8.2} mm\n",
        bounds.z0 * m_to_mm,
        bounds.z1 * m_to_mm
    ));
    out.push_str("layers:\n");
    if pcb.stackup.is_empty() {
        out.push_str("(none)");
    }
    for layer in &pcb.stackup {
        match layer.layer_type.as_str() {
            "signal" => out.push_str(&format!(" signal     {:>8.2} um", layer.thickness * 1e6)),
            "dielectric" => out.push_str(&format!(" dielectric {:>8.2} mm", layer.thickness * 1e3)),
            "plane" => out.push_str(&format!(" plane      {:>8.2} um", layer.thickness * 1e6)),
            _ => out.push_str(&format!(" ***        {:>8.2} mm", layer.thickness * 1e3)),
        }
        out.push_str(&format!(" eps_r {:.2} '{}'\n", layer.epsilon_r, layer.name));
    }
    out
}

/// Native `HyperLynx::PrintNets` output (stdout).
pub fn print_nets(hyp: &HypFile) -> String {
    let mut names: Vec<String> = hyp.net.iter().map(|n| n.net_name.clone()).collect();
    names.sort();
    names.dedup();
    let mut out = String::from("nets:");
    for name in names {
        out.push(' ');
        out.push_str(&name);
    }
    out.push('\n');
    out
}
