//! HyperLynx file model — port of `hyp2mat/lib/hypfile.h` and the `exec_*`
//! handlers in `hyp2mat/lib/exec_*.cc` and `hyp2mat/lib/draw.cc`.
//!
//! Units follow the original implementation exactly:
//! * `{UNITS=ENGLISH ...}` means inches (`unit = 0.0254` m),
//! * `{UNITS=METRIC ...}` means **centimetres** (`unit = 0.01` m),
//! * copper thickness is either the same length unit or a weight unit.
//!
//! The HyperLynx grammar stores all lengths in these units and multiplies by
//! `unit`/`metal_thickness_unit` while parsing, so the model below is in
//! metres once a record has been accepted.

use std::collections::BTreeMap;

/// Physical constants of the original implementation (`HypFile::Hyp::Hyp`).
pub const INCHES: f64 = 0.0254;
pub const COPPER_IMPERIAL_WEIGHT: f64 = 1.341;
pub const COPPER_METRIC_WEIGHT: f64 = 0.1116;
pub const COPPER_BULK_RESISTIVITY: f64 = 1.724e-8;
pub const COPPER_TEMPERATURE_COEFFICIENT: f64 = 0.00393;
pub const FR4_EPSILON_R: f64 = 4.3;
pub const FR4_LOSS_TANGENT: f64 = 0.020;
pub const CONFORMAL_EPSILON_R: f64 = 3.3;
/// Minimum number of polygon segments used to draw a full circle.
pub const MIN_CIRCLE_SEGMENTS: i32 = 8;

/// A point in HyperLynx coordinates (metres once converted).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HypPoint {
    pub x: f64,
    pub y: f64,
}

impl HypPoint {
    pub fn new(x: f64, y: f64) -> Self {
        HypPoint { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Signal,
    Dielectric,
    Plane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadType {
    Metal,
    Antipad,
    ThermalRelief,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadShape {
    Oval,
    Rectangular,
    Oblong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinFunction {
    SimIn,
    SimOut,
    SimBoth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypPolygonType {
    Pour,
    Plane,
    Copper,
    Pad,
    Antipad,
}

/// A HyperLynx polygon (single edge, or one edge of a multi-edge polygon).
#[derive(Debug, Clone, PartialEq)]
pub struct HypPolygon {
    pub vertex: Vec<HypPoint>,
    pub polygon_type: HypPolygonType,
    pub net_name: String,
    pub id: i32,
    /// `false` for `POLYVOID` holes.
    pub positive: bool,
    /// Edge width of the polygon; `0` for filled areas.
    pub width: f64,
    /// Trace-to-plane clearance; `-1.0` if not set.
    pub plane_separation: f64,
    /// Left-hand trace-to-plane clearance; `-1.0` if not set.
    pub left_plane_separation: f64,
    pub layer_name: String,
}

impl Default for HypPolygon {
    fn default() -> Self {
        HypPolygon {
            vertex: Vec::new(),
            polygon_type: HypPolygonType::Copper,
            net_name: String::new(),
            id: -1,
            positive: true,
            width: 0.0,
            plane_separation: -1.0,
            left_plane_separation: -1.0,
            layer_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HypVia {
    pub x: f64,
    pub y: f64,
    pub layer0_name: String,
    pub layer1_name: String,
    pub z0: f64,
    pub z1: f64,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HypPin {
    pub x: f64,
    pub y: f64,
    pub pin_reference: String,
    pub padstack_name: String,
    pub pin_function: PinFunction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnroutedSegment {
    pub x1: f64,
    pub y1: f64,
    pub layer1_name: String,
    pub x2: f64,
    pub y2: f64,
    pub layer2_name: String,
    pub zlayer_name: String,
    pub zlayer_name_set: bool,
    pub width: f64,
    pub impedance: f64,
    pub impedance_set: bool,
    pub delay: f64,
    pub resistance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HypLayer {
    pub layer_type: LayerType,
    pub bulk_resistivity: f64,
    pub conformal: bool,
    pub epsilon_r: f64,
    pub layer_name: String,
    pub loss_tangent: f64,
    pub material_name: String,
    pub plane_separation: f64,
    pub plating_thickness: f64,
    pub prepreg: bool,
    pub temperature_coefficient: f64,
    pub thickness: f64,
    pub z0: f64,
    pub z1: f64,
}

impl HypLayer {
    fn new(layer_type: LayerType) -> Self {
        HypLayer {
            layer_type,
            bulk_resistivity: 0.0,
            conformal: false,
            epsilon_r: 0.0,
            layer_name: String::new(),
            loss_tangent: 0.0,
            material_name: String::new(),
            plane_separation: 0.0,
            plating_thickness: 0.0,
            prepreg: false,
            temperature_coefficient: 0.0,
            thickness: 0.0,
            z0: 0.0,
            z1: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub device_type: String,
    pub ref_: String,
    pub name: String,
    pub value_float: f64,
    pub value_float_set: bool,
    pub value_string: String,
    pub value_string_set: bool,
    pub layer_name: String,
    pub package: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Supply {
    pub name: String,
    pub value_float: f64,
    pub voltage_specified: bool,
    pub conversion: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pad {
    pub layer_name: String,
    pub pad_type: PadType,
    pub pad_shape: PadShape,
    pub pad_sx: f64,
    pub pad_sy: f64,
    pub pad_angle: f64,
    pub thermal_clear_shape: PadShape,
    pub thermal_clear_sx: f64,
    pub thermal_clear_sy: f64,
    pub thermal_clear_angle: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Padstack {
    pub padstack_name: String,
    pub drill_size: f64,
    pub pads: Vec<Pad>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NetClass {
    pub net_class_name: String,
    pub net_class_element: Vec<String>,
    pub attribute: Vec<Attribute>,
}

/// All polygons of one net that share a HyperLynx polygon id.
///
/// `std::map<int, PolygonList>` in the original; `BTreeMap` preserves the
/// sorted-by-id iteration order the exporter relies on.
pub type PolygonMap = BTreeMap<i32, Vec<HypPolygon>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Net {
    pub net_name: String,
    pub metal: PolygonMap,
    pub plane_separation: f64,
    pub via: Vec<HypVia>,
    pub unrouted_segment: Vec<UnroutedSegment>,
    pub pin: Vec<HypPin>,
    pub attribute: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Board {
    pub edge: Vec<HypPolygon>,
    pub segments: Vec<HypPolygon>,
    pub attribute: Vec<Attribute>,
    pub plane_separation: f64,
}

/// Geometry settings shared by the model and the polygon library.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeomSettings {
    /// Coordinate resolution in metres (default 10 µm).
    pub grid: f64,
    /// Maximum difference between a circular arc and its polygon (0 = octagon).
    pub arc_precision: f64,
}

impl Default for GeomSettings {
    fn default() -> Self {
        GeomSettings {
            grid: 1e-5,
            arc_precision: 0.0,
        }
    }
}

/// A parsed HyperLynx file.
#[derive(Debug, Clone, PartialEq)]
pub struct HypFile {
    pub board: Board,
    pub stackup: Vec<HypLayer>,
    pub device: Vec<Device>,
    pub supply: Vec<Supply>,
    pub padstack: Vec<Padstack>,
    pub net_class: Vec<NetClass>,
    pub net: Vec<Net>,
    pub version: f64,
    pub detailed: bool,
    pub unit: f64,
    pub metal_thickness_unit: f64,
    pub use_die_for_metal: bool,
    pub geometry: GeomSettings,
    pub key: String,
    /// Set when an `{END}` record was parsed. The native implementation
    /// floods layers with copper at that point.
    pub ended: bool,
}

impl Default for HypFile {
    fn default() -> Self {
        Self::new(GeomSettings::default())
    }
}

impl HypFile {
    pub fn new(geometry: GeomSettings) -> Self {
        HypFile {
            board: Board {
                plane_separation: -1.0,
                ..Board::default()
            },
            stackup: Vec::new(),
            device: Vec::new(),
            supply: Vec::new(),
            padstack: Vec::new(),
            net_class: Vec::new(),
            net: Vec::new(),
            version: 0.0,
            detailed: false,
            unit: 1.0,
            metal_thickness_unit: 1.0,
            use_die_for_metal: false,
            geometry,
            key: String::new(),
            ended: false,
        }
    }

    /// `exec_units`: set the length and metal thickness conversion factors.
    pub fn set_units(&mut self, english: bool, metal_weight: bool) {
        if english {
            self.unit = INCHES;
            if metal_weight {
                // Native quirk: `copper_imperial_weight` is in mils but is
                // multiplied by the inch-to-metre factor as if it were
                // inches, making imperial weight layers 1000x too thick.
                self.metal_thickness_unit = COPPER_IMPERIAL_WEIGHT * self.unit;
            } else {
                self.metal_thickness_unit = self.unit;
            }
        } else {
            self.unit = 0.01;
            if metal_weight {
                self.metal_thickness_unit = COPPER_METRIC_WEIGHT * self.unit;
            } else {
                self.metal_thickness_unit = self.unit;
            }
        }
    }

    pub fn arc_precision(&self) -> f64 {
        self.geometry.arc_precision
    }

    pub fn grid(&self) -> f64 {
        self.geometry.grid
    }

    /// Add a signal or plane layer (`add_metal_layer`).
    ///
    /// The argument list mirrors the native `parse_param` fields the handler
    /// reads; grouping them into a struct here would obscure that mapping.
    #[allow(clippy::too_many_arguments)]
    pub fn add_metal_layer(
        &mut self,
        layer_type: LayerType,
        thickness: f64,
        plating_thickness: Option<f64>,
        bulk_resistivity: Option<f64>,
        temperature_coefficient: Option<f64>,
        epsilon_r: Option<f64>,
        loss_tangent: Option<f64>,
        layer_name: Option<String>,
        material_name: Option<String>,
        plane_separation: Option<f64>,
    ) {
        let mut layer = HypLayer::new(LayerType::Signal);
        layer.layer_type = layer_type;
        layer.thickness = thickness * self.metal_thickness_unit;
        layer.plating_thickness = plating_thickness.unwrap_or(0.0) * self.metal_thickness_unit;
        layer.bulk_resistivity = bulk_resistivity.unwrap_or(COPPER_BULK_RESISTIVITY);
        layer.temperature_coefficient =
            temperature_coefficient.unwrap_or(COPPER_TEMPERATURE_COEFFICIENT);
        layer.epsilon_r = epsilon_r.unwrap_or(FR4_EPSILON_R);
        layer.loss_tangent = loss_tangent.unwrap_or(FR4_LOSS_TANGENT);
        layer.layer_name = layer_name.unwrap_or_default();
        layer.material_name = material_name.unwrap_or_default();
        layer.plane_separation = match plane_separation {
            Some(ps) => ps * self.unit,
            None => self.board.plane_separation,
        };
        self.stackup.push(layer);
        self.calc_layer_position();
        self.calc_layer_epsilon();
    }

    /// Add a dielectric layer (`add_dielectric_layer`).
    #[allow(clippy::too_many_arguments)]
    pub fn add_dielectric_layer(
        &mut self,
        thickness: f64,
        epsilon_r: Option<f64>,
        loss_tangent: Option<f64>,
        conformal: Option<bool>,
        prepreg: Option<bool>,
        layer_name: Option<String>,
        material_name: Option<String>,
    ) {
        let mut layer = HypLayer::new(LayerType::Dielectric);
        layer.thickness = thickness * self.unit;
        layer.plating_thickness = 0.0;
        layer.bulk_resistivity = 0.0;
        layer.temperature_coefficient = 0.0;
        layer.conformal = conformal.unwrap_or(false);
        layer.epsilon_r = epsilon_r.unwrap_or(if layer.conformal {
            CONFORMAL_EPSILON_R
        } else {
            FR4_EPSILON_R
        });
        layer.loss_tangent = loss_tangent.unwrap_or(FR4_LOSS_TANGENT);
        layer.prepreg = prepreg.unwrap_or(true);
        layer.layer_name = layer_name.unwrap_or_default();
        layer.material_name = material_name.unwrap_or_default();
        layer.plane_separation = 0.0;
        self.stackup.push(layer);
        self.calc_layer_position();
        self.calc_layer_epsilon();
    }

    /// `calc_layer_position`: stackup is stored top first, bottom last; metal
    /// layers are infinitely thin sheets at the top of the stack position.
    pub fn calc_layer_position(&mut self) {
        let mut z_top = 0.0;
        for layer in self.stackup.iter_mut().rev() {
            if layer.layer_type == LayerType::Dielectric {
                layer.z0 = z_top;
                layer.z1 = z_top + layer.thickness;
                z_top = layer.z1;
            } else {
                layer.z0 = z_top;
                layer.z1 = z_top;
            }
        }
    }

    /// `calc_layer_epsilon`: give metal layers the dielectric constant of the
    /// adjacent substrate (or air for the outer layers).
    pub fn calc_layer_epsilon(&mut self) {
        if self.use_die_for_metal {
            return;
        }
        for i in 0..self.stackup.len() {
            if self.stackup[i].layer_type == LayerType::Dielectric {
                continue;
            }
            let upper = (0..i)
                .rev()
                .find(|&j| self.stackup[j].layer_type == LayerType::Dielectric);
            let lower = (i..self.stackup.len())
                .find(|&j| self.stackup[j].layer_type == LayerType::Dielectric);
            match (upper, lower) {
                (Some(u), Some(l)) => {
                    let upper_layer = &self.stackup[u];
                    let lower_layer = &self.stackup[l];
                    let (eps, lt) = if upper_layer.prepreg && !lower_layer.prepreg {
                        (upper_layer.epsilon_r, upper_layer.loss_tangent)
                    } else if !upper_layer.prepreg && lower_layer.prepreg {
                        (lower_layer.epsilon_r, lower_layer.loss_tangent)
                    } else {
                        (
                            (upper_layer.epsilon_r + lower_layer.epsilon_r) / 2.0,
                            (upper_layer.loss_tangent + lower_layer.loss_tangent) / 2.0,
                        )
                    };
                    self.stackup[i].epsilon_r = eps;
                    self.stackup[i].loss_tangent = lt;
                }
                _ => {
                    self.stackup[i].epsilon_r = 1.0;
                    self.stackup[i].loss_tangent = 0.0;
                }
            }
        }
    }

    /// `flood_layers_with_copper`: fill the requested layers with the board
    /// outline as copper. Returns an error when there is no board outline to
    /// flood with (the native implementation dereferences `board.edge.front()`
    /// and crashes).
    pub fn flood_layers_with_copper(&mut self, flood_layers: &[String]) -> Result<(), String> {
        if flood_layers.is_empty() {
            return Ok(());
        }
        let outline = self
            .board
            .edge
            .first()
            .map(|e| e.vertex.clone())
            .ok_or_else(|| "cannot flood layers: board outline is empty".to_string())?;
        let flood_plane_layers = flood_layers.iter().any(|l| l == "plane_layers");

        let layer_names: Vec<(String, LayerType)> = self
            .stackup
            .iter()
            .filter(|l| l.layer_type != LayerType::Dielectric)
            .map(|l| (l.layer_name.clone(), l.layer_type))
            .collect();

        for (layer_name, layer_type) in layer_names {
            let flood_this = flood_layers.iter().any(|l| l == &layer_name)
                || (flood_plane_layers && layer_type == LayerType::Plane);
            if !flood_this {
                continue;
            }
            let mut copper = HypPolygon {
                vertex: outline.clone(),
                polygon_type: HypPolygonType::Plane,
                net_name: layer_name.clone(),
                id: -1,
                positive: true,
                width: 0.0,
                plane_separation: -1.0,
                left_plane_separation: -1.0,
                layer_name: layer_name.clone(),
            };
            copper.id = -1;
            match self.net.iter_mut().find(|n| n.net_name == layer_name) {
                Some(net) => Self::add_polygon_to_net(net, copper),
                None => {
                    let mut net = Net {
                        net_name: layer_name.clone(),
                        metal: PolygonMap::new(),
                        plane_separation: -1.0,
                        via: Vec::new(),
                        unrouted_segment: Vec::new(),
                        pin: Vec::new(),
                        attribute: Vec::new(),
                    };
                    Self::add_polygon_to_net(&mut net, copper);
                    self.net.push(net);
                }
            }
        }
        Ok(())
    }

    /// `add_polygon_to_net`: assign an id to polygons that have none, then
    /// append the edge to that id's edge list.
    pub fn add_polygon_to_net(net: &mut Net, mut polygon: HypPolygon) {
        polygon.net_name = net.net_name.clone();
        if polygon.id < 0 {
            let mut new_id = 0;
            if let Some((&smallest, _)) = net.metal.iter().next() {
                new_id = smallest - 1;
            }
            if new_id >= -1 {
                new_id = -2;
            }
            polygon.id = new_id;
        }
        net.metal.entry(polygon.id).or_default().push(polygon);
    }

    /// `add_polygon`: add a polygon to the current (last) net.
    pub fn add_polygon(&mut self, polygon: HypPolygon) -> Result<(), String> {
        if polygon.vertex.is_empty() {
            return Ok(());
        }
        let net = self
            .net
            .last_mut()
            .ok_or_else(|| "polygon outside a net".to_string())?;
        Self::add_polygon_to_net(net, polygon);
        Ok(())
    }

    /// `add_pad`: convert a pad to a polygon and add it to the current net.
    pub fn add_pad(&mut self, x: f64, y: f64, pad: &Pad) -> Result<(), String> {
        let polygon = self.pad2poly(x, y, pad);
        self.add_polygon(polygon)
    }

    /// `add_via`: append a via hole spanning the given layers.
    pub fn add_via(&mut self, x: f64, y: f64, layer0_name: &str, layer1_name: &str, radius: f64) {
        if radius == 0.0 {
            return;
        }
        let mut z0 = self.stackup.last().map(|l| l.z0).unwrap_or(0.0);
        let mut z1 = self.stackup.first().map(|l| l.z1).unwrap_or(0.0);
        for layer in &self.stackup {
            if layer0_name == layer.layer_name {
                z0 = layer.z0;
                z1 = layer.z1;
            }
        }
        for layer in &self.stackup {
            if layer1_name == layer.layer_name {
                if layer.z1 > z1 {
                    z1 = layer.z1;
                }
                if layer.z0 < z0 {
                    z0 = layer.z0;
                }
            }
        }
        let via = HypVia {
            x,
            y,
            layer0_name: layer0_name.to_string(),
            layer1_name: layer1_name.to_string(),
            z0,
            z1,
            radius,
        };
        if let Some(net) = self.net.last_mut() {
            net.via.push(via);
        }
    }

    /// `add_perimeter_polygon`: append a perimeter segment/arc and rebuild the
    /// closed board edge loops from all segments collected so far.
    pub fn add_perimeter_polygon(&mut self, new_segment: HypPolygon) {
        if new_segment.vertex.is_empty() {
            return;
        }
        if new_segment.vertex.len() == 1 {
            // Native: error("runt perimeter segment") and return.
            return;
        }
        self.board.segments.push(new_segment);

        let mut segs = self.board.segments.clone();
        self.board.edge.clear();
        let mut current_edge = HypPolygon::default();
        current_edge.vertex.clear();

        let mut found = true;
        while found {
            found = false;
            let mut index = None;
            for (i, seg) in segs.iter().enumerate() {
                let last = current_edge.vertex.last().copied();
                if last.is_none() || seg.vertex.first().copied() == last {
                    found = true;
                    current_edge.vertex.extend_from_slice(&seg.vertex);
                    index = Some(i);
                } else if seg.vertex.last().copied() == last {
                    found = true;
                    current_edge.vertex.extend(seg.vertex.iter().rev().copied());
                    index = Some(i);
                }
                if found {
                    segs.remove(i);
                    if current_edge.vertex.first() == current_edge.vertex.last() {
                        current_edge.positive = false;
                        self.board.edge.push(current_edge.clone());
                        current_edge.vertex.clear();
                    }
                    break;
                }
            }
            if found && index.is_none() {
                break;
            }
        }

        if let Some(first) = self.board.edge.first_mut() {
            first.positive = true;
        }
    }

    /// `arc2poly`: approximate a circular arc with a polygon.
    ///
    /// The signature matches the native helper, which takes the two end
    /// points, the centre and the radius.
    #[allow(clippy::too_many_arguments)]
    pub fn arc2poly(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        xc: f64,
        yc: f64,
        r: f64,
        clockwise: bool,
    ) -> HypPolygon {
        let pi = std::f64::consts::PI;
        let mut arc = HypPolygon {
            width: 0.0,
            positive: true,
            layer_name: String::new(),
            ..HypPolygon::default()
        };

        let mut alpha = (y1 - yc).atan2(x1 - xc);
        let mut beta = (y2 - yc).atan2(x2 - xc);

        if clockwise {
            if alpha < beta {
                alpha += 2.0 * pi;
            }
        } else {
            if beta < alpha {
                beta += 2.0 * pi;
            }
            if x1 == x2 && y1 == y2 {
                beta = alpha + 2.0 * pi;
            }
        }

        let mut segments = MIN_CIRCLE_SEGMENTS;
        let arc_precision = self.arc_precision();
        if arc_precision > 0.0 {
            loop {
                let arc_error = r * (1.0 - (pi / segments as f64).cos());
                if arc_error > arc_precision {
                    segments += 4;
                } else {
                    break;
                }
            }
        }

        let mut poly_points = (segments as f64 * (beta - alpha).abs() / (2.0 * pi)).round() as i32;
        if poly_points < 1 {
            poly_points = 1;
        }

        arc.vertex.push(HypPoint::new(x1, y1));
        for i in 1..poly_points {
            let angle = alpha + (beta - alpha) * i as f64 / poly_points as f64;
            arc.vertex
                .push(HypPoint::new(xc + r * angle.cos(), yc + r * angle.sin()));
        }
        arc.vertex.push(HypPoint::new(x2, y2));
        arc
    }

    /// `segment2poly`: draw a straight trace segment with rounded ends.
    pub fn segment2poly(&self, x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> HypPolygon {
        let mut line = HypPolygon {
            width: 0.0,
            positive: true,
            layer_name: String::new(),
            ..HypPolygon::default()
        };

        let (dx, dy) = if x1 == x2 && y1 == y2 {
            (width / 2.0, 0.0)
        } else {
            let length = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
            (
                width / 2.0 * (y2 - y1) / length,
                width / 2.0 * (x2 - x1) / length,
            )
        };

        let arc2 = self.arc2poly(
            x2 + dx,
            y2 - dy,
            x2 - dx,
            y2 + dy,
            x2,
            y2,
            width / 2.0,
            false,
        );
        let arc1 = self.arc2poly(
            x1 - dx,
            y1 + dy,
            x1 + dx,
            y1 - dy,
            x1,
            y1,
            width / 2.0,
            false,
        );

        line.vertex = arc2.vertex;
        line.vertex.extend_from_slice(&arc1.vertex);
        line
    }

    /// `pad2poly`: convert a pad definition to a polygon at (pad_x, pad_y).
    pub fn pad2poly(&self, pad_x: f64, pad_y: f64, pad: &Pad) -> HypPolygon {
        let pi = std::f64::consts::PI;
        let mut pad_poly = HypPolygon {
            width: 0.0,
            layer_name: pad.layer_name.clone(),
            positive: true,
            polygon_type: if pad.pad_type == PadType::Antipad {
                HypPolygonType::Antipad
            } else {
                HypPolygonType::Pad
            },
            ..HypPolygon::default()
        };

        let mut sx = pad.pad_sx;
        let mut sy = pad.pad_sy;
        let mut angle = pad.pad_angle;

        if angle % 180.0 == 0.0 {
            angle = 0.0;
        } else if angle % 90.0 == 0.0 {
            std::mem::swap(&mut sx, &mut sy);
            angle = 0.0;
        }

        match pad.pad_shape {
            PadShape::Oval => {
                let mut segments = MIN_CIRCLE_SEGMENTS;
                if self.arc_precision() > 0.0 {
                    loop {
                        let arc_error_x = sx / 2.0 * (1.0 - (pi / segments as f64).cos());
                        let arc_error_y = sy / 2.0 * (1.0 - (pi / segments as f64).cos());
                        if arc_error_x > self.arc_precision() || arc_error_y > self.arc_precision()
                        {
                            segments += 4;
                        } else {
                            break;
                        }
                    }
                }
                for i in 0..segments {
                    let alpha = 2.0 * pi * i as f64 / segments as f64;
                    pad_poly.vertex.push(HypPoint::new(
                        sx / 2.0 / (pi / segments as f64).cos() * alpha.cos(),
                        sy / 2.0 / (pi / segments as f64).cos() * alpha.sin(),
                    ));
                }
            }
            PadShape::Rectangular => {
                pad_poly.vertex.push(HypPoint::new(sx / 2.0, sy / 2.0));
                pad_poly.vertex.push(HypPoint::new(sx / 2.0, -sy / 2.0));
                pad_poly.vertex.push(HypPoint::new(-sx / 2.0, -sy / 2.0));
                pad_poly.vertex.push(HypPoint::new(-sx / 2.0, sy / 2.0));
            }
            PadShape::Oblong => {
                if sx > sy {
                    let x1 = (sx - sy) / 2.0;
                    let y1 = -sy / 2.0;
                    let x2 = x1;
                    let y2 = -y1;
                    let xc = (sx - sy) / 2.0;
                    let yc = 0.0;
                    let r = sy / 2.0;
                    let arc_right = self.arc2poly(x1, y1, x2, y2, xc, yc, r, false);
                    let arc_left = self.arc2poly(-x1, -y1, -x2, -y2, -xc, -yc, r, false);
                    pad_poly.vertex = arc_right.vertex;
                    pad_poly.vertex.extend_from_slice(&arc_left.vertex);
                } else {
                    let x1 = sx / 2.0;
                    let y1 = (sy - sx) / 2.0;
                    let x2 = -x1;
                    let y2 = y1;
                    let xc = 0.0;
                    let yc = (sy - sx) / 2.0;
                    let r = sx / 2.0;
                    let arc_top = self.arc2poly(x1, y1, x2, y2, xc, yc, r, false);
                    let arc_bottom = self.arc2poly(-x1, -y1, -x2, -y2, -xc, -yc, r, false);
                    pad_poly.vertex = arc_top.vertex;
                    pad_poly.vertex.extend_from_slice(&arc_bottom.vertex);
                }
            }
        }

        if angle != 0.0 {
            let cosa = (angle * pi / 180.0).cos();
            let sina = (angle * pi / 180.0).sin();
            for point in pad_poly.vertex.iter_mut() {
                let x_rot = cosa * point.x - sina * point.y;
                let y_rot = sina * point.x + cosa * point.y;
                point.x = x_rot;
                point.y = y_rot;
            }
        }

        for point in pad_poly.vertex.iter_mut() {
            point.x += pad_x;
            point.y += pad_y;
        }

        pad_poly
    }

    /// `exec_padstack_end`: expand MDEF/ADEF default pads and fill in missing
    pub fn finish_padstack(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();
        let Some(padstack) = self.padstack.last() else {
            return warnings;
        };
        let stackup = self.stackup.clone();
        let pads = padstack.pads.clone();
        let drill_size = padstack.drill_size;
        let mut new_pads: Vec<Pad> = Vec::new();

        for layer in &stackup {
            let mut layer_has_pad = false;
            let mut layer_has_antipad = false;
            for pad in &pads {
                if layer.layer_name == pad.layer_name {
                    new_pads.push(pad.clone());
                    layer_has_pad |= pad.pad_type == PadType::Metal;
                    layer_has_antipad |= pad.pad_type == PadType::Antipad;
                }
            }

            if !layer_has_antipad && layer.layer_type == LayerType::Plane {
                for pad in &pads {
                    if pad.layer_name == "ADEF" {
                        let mut new_pad = pad.clone();
                        new_pad.layer_name = layer.layer_name.clone();
                        new_pads.push(new_pad);
                    }
                }
            }

            if !layer_has_pad
                && (layer.layer_type == LayerType::Signal || layer.layer_type == LayerType::Plane)
            {
                for pad in &pads {
                    if pad.layer_name == "MDEF" {
                        let mut new_pad = pad.clone();
                        new_pad.layer_name = layer.layer_name.clone();
                        new_pads.push(new_pad);
                        layer_has_pad = true;
                    }
                }
            }

            let is_through_hole_via = pads
                .iter()
                .any(|p| p.layer_name == "MDEF" && p.pad_type == PadType::Metal);

            // Native warns once per stackup layer when a padstack has no metal
            // pad at all (the message is emitted inside the layer loop).
            if !pads.iter().any(|p| p.pad_type == PadType::Metal) {
                warnings.push(format!(
                    "warning: padstack without metal: {}",
                    self.padstack
                        .last()
                        .map(|p| p.padstack_name.clone())
                        .unwrap_or_default()
                ));
            }

            let is_through_hole_pad = is_through_hole_via && drill_size > 0.0;
            if !layer_has_pad
                && is_through_hole_pad
                && (layer.layer_type == LayerType::Signal || layer.layer_type == LayerType::Plane)
            {
                let mut new_pad: Option<Pad> = None;
                for pad in pads.iter().filter(|p| p.pad_type == PadType::Metal) {
                    match &new_pad {
                        None => {
                            let mut copy = pad.clone();
                            copy.layer_name = layer.layer_name.clone();
                            new_pad = Some(copy);
                        }
                        Some(current) => {
                            if current.pad_shape != pad.pad_shape
                                || current.pad_sx != pad.pad_sx
                                || current.pad_sy != pad.pad_sy
                                || current.pad_angle != pad.pad_angle
                            {
                                let mut radius = current.pad_sx;
                                if radius > current.pad_sy {
                                    radius = current.pad_sy;
                                }
                                if radius > pad.pad_sx {
                                    radius = pad.pad_sx;
                                }
                                if radius > pad.pad_sy {
                                    radius = pad.pad_sy;
                                }
                                new_pad = Some(Pad {
                                    pad_shape: PadShape::Oval,
                                    pad_sx: radius,
                                    pad_sy: radius,
                                    pad_angle: 0.0,
                                    ..current.clone()
                                });
                            }
                        }
                    }
                }
                if let Some(pad) = new_pad {
                    new_pads.push(pad);
                }
            }
        }

        // A padstack without any metal pad cannot span a via; the warning is
        // emitted per layer above.

        if let Some(padstack) = self.padstack.last_mut() {
            padstack.pads = new_pads;
        }
        warnings
    }

    /// Native `HypFile::Hyp::save`: the `-d` model dump.
    ///
    /// The format (field names, quoting and 6-significant-digit numbers) is
    /// byte-compatible with the native `hyp_debug.txt` dump.
    pub fn dump(&self) -> String {
        use crate::format::format_cpp_float as f;
        let mut out = String::new();
        let b = |value: bool| if value { "1" } else { "0" };
        out.push_str(&format!("unit = {}\n", f(self.unit)));
        out.push_str(&format!(
            "metal_thickness_unit = {}\n",
            f(self.metal_thickness_unit)
        ));
        out.push_str(&format!(
            "use_die_for_metal = {}\n",
            b(self.use_die_for_metal)
        ));
        out.push_str(&format!("min_circle_segments = {}\n", MIN_CIRCLE_SEGMENTS));
        out.push_str(&format!(
            "arc_precision = {}\n",
            f(self.geometry.arc_precision)
        ));
        out.push_str(&format!("pi = {}\n", f(std::f64::consts::PI)));
        out.push_str(&format!("inches = {}\n", f(INCHES)));
        out.push_str(&format!(
            "copper_imperial_weight = {}\n",
            f(COPPER_IMPERIAL_WEIGHT)
        ));
        out.push_str(&format!(
            "copper_metric_weight = {}\n",
            f(COPPER_METRIC_WEIGHT)
        ));
        out.push_str(&format!(
            "copper_bulk_resistivity = {}\n",
            f(COPPER_BULK_RESISTIVITY)
        ));
        out.push_str(&format!(
            "copper_temperature_coefficient = {}\n",
            f(COPPER_TEMPERATURE_COEFFICIENT)
        ));
        out.push_str(&format!("fr4_epsilon_r = {}\n", f(FR4_EPSILON_R)));
        out.push_str(&format!("fr4_loss_tangent = {}\n", f(FR4_LOSS_TANGENT)));
        out.push_str(&format!(
            "conformal_epsilon_r = {}\n",
            f(CONFORMAL_EPSILON_R)
        ));

        out.push_str("board\n");
        for edge in &self.board.edge {
            out.push_str(" polygon\n");
            out.push_str(&format!("  positive {}\n", b(edge.positive)));
            for point in &edge.vertex {
                out.push_str(&format!("  x = {}\n", f(point.x)));
                out.push_str(&format!("  y = {}\n", f(point.y)));
            }
        }
        out.push_str(&format!(
            " plane_separation = {}\n",
            f(self.board.plane_separation)
        ));
        out.push_str(" board_attribute\n");
        for attribute in &self.board.attribute {
            out.push_str(&format!("  name = '{}'\n", attribute.name));
            out.push_str(&format!("  value = '{}'\n", attribute.value));
        }

        out.push_str("device\n");
        for device in &self.device {
            out.push_str(" device\n");
            out.push_str(&format!("  device_type = '{}'\n", device.device_type));
            out.push_str(&format!("  ref = '{}'\n", device.ref_));
            out.push_str(&format!("  name = '{}'\n", device.name));
            out.push_str(&format!("  value_float = {}\n", f(device.value_float)));
            out.push_str(&format!(
                "  value_float_set = {}\n",
                b(device.value_float_set)
            ));
            out.push_str(&format!("  value_string = '{}'\n", device.value_string));
            out.push_str(&format!(
                "  value_string_set = {}\n",
                b(device.value_string_set)
            ));
            out.push_str(&format!("  layer_name = '{}'\n", device.layer_name));
            out.push_str(&format!("  package = '{}'\n", device.package));
        }

        out.push_str("supplies\n");
        for supply in &self.supply {
            out.push_str(" supply\n");
            out.push_str(&format!("  name = '{}'\n", supply.name));
            out.push_str(&format!("  value_float = {}\n", f(supply.value_float)));
            out.push_str(&format!(
                "  voltage_specified = {}\n",
                b(supply.voltage_specified)
            ));
            out.push_str(&format!("  conversion = {}\n", b(supply.conversion)));
        }

        out.push_str("layers\n");
        for layer in &self.stackup {
            out.push_str(" layer\n");
            out.push_str(&format!("  layer_name = '{}'\n", layer.layer_name));
            let (type_number, type_name) = match layer.layer_type {
                LayerType::Signal => (0, "LAYER_SIGNAL"),
                LayerType::Dielectric => (1, "LAYER_DIELECTRIC"),
                LayerType::Plane => (2, "LAYER_PLANE"),
            };
            out.push_str(&format!("  layer_type = {} {}\n", type_number, type_name));
            out.push_str(&format!(
                "  bulk_resistivity = {}\n",
                f(layer.bulk_resistivity)
            ));
            out.push_str(&format!("  conformal = {}\n", b(layer.conformal)));
            out.push_str(&format!("  epsilon_r = {}\n", f(layer.epsilon_r)));
            out.push_str(&format!("  layer_name = {}\n", layer.layer_name));
            out.push_str(&format!("  loss_tangent = {}\n", f(layer.loss_tangent)));
            out.push_str(&format!("  material_name = {}\n", layer.material_name));
            out.push_str(&format!(
                "  plane_separation = {}\n",
                f(layer.plane_separation)
            ));
            out.push_str(&format!(
                "  plating_thickness = {}\n",
                f(layer.plating_thickness)
            ));
            out.push_str(&format!("  prepreg = {}\n", b(layer.prepreg)));
            out.push_str(&format!(
                "  temperature_coefficient = {}\n",
                f(layer.temperature_coefficient)
            ));
            out.push_str(&format!("  thickness = {}\n", f(layer.thickness)));
            out.push_str(&format!("  z0 = {}\n", f(layer.z0)));
            out.push_str(&format!("  z1 = {}\n", f(layer.z1)));
        }

        out.push_str("padstacks\n");
        for padstack in &self.padstack {
            out.push_str(" padstack\n");
            out.push_str(&format!("  padstack_name = '{}'\n", padstack.padstack_name));
            out.push_str(&format!("  drill_size = {}\n", f(padstack.drill_size)));
            out.push_str("  pads\n");
            for pad in &padstack.pads {
                out.push_str("   pad\n");
                out.push_str(&format!("    layer_name = '{}'\n", pad.layer_name));
                out.push_str(&format!(
                    "    pad_type = {} {}\n",
                    pad_type_number(pad.pad_type),
                    pad_type_label(pad.pad_type)
                ));
                out.push_str(&format!(
                    "    pad_shape = {} {}\n",
                    pad_shape_number(pad.pad_shape),
                    pad_shape_label(pad.pad_shape)
                ));
                out.push_str(&format!("    pad_sx = {}\n", f(pad.pad_sx)));
                out.push_str(&format!("    pad_sy = {}\n", f(pad.pad_sy)));
                out.push_str(&format!("    pad_angle = {}\n", f(pad.pad_angle)));
                out.push_str(&format!(
                    "    thermal_clear_shape = {} {}\n",
                    pad_shape_number(pad.thermal_clear_shape),
                    pad_shape_label(pad.thermal_clear_shape)
                ));
                out.push_str(&format!(
                    "    thermal_clear_sx = {}\n",
                    f(pad.thermal_clear_sx)
                ));
                out.push_str(&format!(
                    "    thermal_clear_sy = {}\n",
                    f(pad.thermal_clear_sy)
                ));
                out.push_str(&format!(
                    "    thermal_clear_angle = {}\n",
                    f(pad.thermal_clear_angle)
                ));
            }
        }

        out.push_str("net\n");
        for net in &self.net {
            out.push_str(" net\n");
            out.push_str(&format!("  net_name = '{}'\n", net.net_name));
            for (id, edges) in &net.metal {
                out.push_str(&format!("  polygon id {}\n", id));
                for edge in edges {
                    out.push_str("   polygon\n");
                    out.push_str(&format!("    positive {}\n", b(edge.positive)));
                    out.push_str(&format!("    width {}\n", f(edge.width)));
                    out.push_str(&format!("    layer_name {}\n", edge.layer_name));
                    for point in &edge.vertex {
                        out.push_str(&format!("    x = {}\n", f(point.x)));
                        out.push_str(&format!("    y = {}\n", f(point.y)));
                    }
                }
            }
            out.push_str("  vias\n");
            for via in &net.via {
                out.push_str("   via\n");
                out.push_str(&format!("    x = {}\n", f(via.x)));
                out.push_str(&format!("    y = {}\n", f(via.y)));
                out.push_str(&format!("    layer0_name = {}\n", via.layer0_name));
                out.push_str(&format!("    layer1_name = {}\n", via.layer1_name));
                out.push_str(&format!("    radius = {}\n", f(via.radius)));
            }
            out.push_str("  pins\n");
            for pin in &net.pin {
                out.push_str("   pin\n");
                out.push_str(&format!("    x = {}\n", f(pin.x)));
                out.push_str(&format!("    y = {}\n", f(pin.y)));
                out.push_str(&format!("    pin_reference = {}\n", pin.pin_reference));
                out.push_str(&format!("    padstack_name = {}\n", pin.padstack_name));
                let (number, label) = match pin.pin_function {
                    PinFunction::SimIn => (0, "PIN_SIM_IN"),
                    PinFunction::SimOut => (1, "PIN_SIM_OUT"),
                    PinFunction::SimBoth => (2, "PIN_SIM_BOTH"),
                };
                out.push_str(&format!("    pin_function = {} {}\n", number, label));
            }
            out.push_str("  unrouted_segments\n");
            for segment in &net.unrouted_segment {
                out.push_str("   unrouted_segment\n");
                out.push_str(&format!("    x1 = {}\n", f(segment.x1)));
                out.push_str(&format!("    y1 = {}\n", f(segment.y1)));
                out.push_str(&format!("    layer1_name = {}\n", segment.layer1_name));
                out.push_str(&format!("    x2 = {}\n", f(segment.x2)));
                out.push_str(&format!("    y2 = {}\n", f(segment.y2)));
                out.push_str(&format!("    layer2_name = {}\n", segment.layer2_name));
                out.push_str(&format!(
                    "    zlayer_name_set = {}\n",
                    b(segment.zlayer_name_set)
                ));
                out.push_str(&format!("    zlayer_name = {}\n", segment.zlayer_name));
                out.push_str(&format!("    width = {}\n", f(segment.width)));
                out.push_str(&format!(
                    "    impedance_set = {}\n",
                    b(segment.impedance_set)
                ));
                out.push_str(&format!("    impedance = {}\n", f(segment.impedance)));
                out.push_str(&format!("    delay = {}\n", f(segment.delay)));
                out.push_str(&format!("    resistance = {}\n", f(segment.resistance)));
            }
            if !net.attribute.is_empty() {
                out.push_str("  attribute\n");
            }
            for attribute in &net.attribute {
                out.push_str(&format!("   name = '{}'\n", attribute.name));
                out.push_str(&format!("   value = '{}'\n", attribute.value));
            }
        }

        out.push_str("net_class\n");
        for net_class in &self.net_class {
            out.push_str(" net_class\n");
            out.push_str(&format!(
                "  net_class_name = '{}'\n",
                net_class.net_class_name
            ));
            out.push_str("  net_class_element\n");
            for element in &net_class.net_class_element {
                out.push_str(&format!("   net_name = '{}'\n", element));
            }
            if !net_class.attribute.is_empty() {
                out.push_str("  attribute\n");
            }
            for attribute in &net_class.attribute {
                out.push_str(&format!("   name = '{}'\n", attribute.name));
                out.push_str(&format!("   value = '{}'\n", attribute.value));
            }
        }

        out.push_str("end\n");
        out
    }
}

fn pad_type_number(pad_type: PadType) -> i32 {
    match pad_type {
        PadType::Metal => 0,
        PadType::Antipad => 1,
        PadType::ThermalRelief => 2,
    }
}

fn pad_type_label(pad_type: PadType) -> &'static str {
    match pad_type {
        PadType::Metal => "PAD_TYPE_METAL",
        PadType::Antipad => "PAD_TYPE_ANTIPAD",
        PadType::ThermalRelief => "PAD_TYPE_THERMAL_RELIEF",
    }
}

fn pad_shape_number(shape: PadShape) -> i32 {
    match shape {
        PadShape::Oval => 0,
        PadShape::Rectangular => 1,
        PadShape::Oblong => 2,
    }
}

fn pad_shape_label(shape: PadShape) -> &'static str {
    match shape {
        PadShape::Oval => "PAD_SHAPE_OVAL",
        PadShape::Rectangular => "PAD_SHAPE_RECTANGULAR",
        PadShape::Oblong => "PAD_SHAPE_OBLONG",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, w: f64) -> HypPolygon {
        HypPolygon {
            vertex: vec![
                HypPoint::new(x, y),
                HypPoint::new(x + w, y),
                HypPoint::new(x + w, y + w),
                HypPoint::new(x, y + w),
            ],
            ..HypPolygon::default()
        }
    }

    #[test]
    fn metric_units_are_centimetres() {
        let mut hyp = HypFile::default();
        hyp.set_units(false, false);
        assert_eq!(hyp.unit, 0.01);
        let mut english = HypFile::default();
        english.set_units(true, false);
        assert_eq!(english.unit, INCHES);
    }

    #[test]
    fn imperial_weight_thickness_matches_native_quirk() {
        let mut hyp = HypFile::default();
        hyp.set_units(true, true);
        // Native multiplies 1.341 (mils) by 0.0254 m/inch: 1000x too large.
        assert!((hyp.metal_thickness_unit - 1.341 * 0.0254).abs() < 1e-15);
    }

    #[test]
    fn metric_weight_thickness_is_correct() {
        let mut hyp = HypFile::default();
        hyp.set_units(false, true);
        assert!((hyp.metal_thickness_unit - 0.001116).abs() < 1e-15);
    }

    #[test]
    fn layer_positions_stack_from_the_bottom() {
        let mut hyp = HypFile::default();
        hyp.set_units(false, false);
        hyp.add_metal_layer(
            LayerType::Signal,
            0.000035,
            None,
            None,
            None,
            None,
            None,
            Some("Top".into()),
            None,
            None,
        );
        hyp.add_dielectric_layer(0.1, None, None, None, None, Some("DL".into()), None);
        hyp.add_metal_layer(
            LayerType::Signal,
            0.000035,
            None,
            None,
            None,
            None,
            None,
            Some("Bottom".into()),
            None,
            None,
        );
        assert_eq!(hyp.stackup[2].z0, 0.0);
        assert_eq!(hyp.stackup[2].z1, 0.0);
        assert_eq!(hyp.stackup[1].z0, 0.0);
        assert!((hyp.stackup[1].z1 - 0.001).abs() < 1e-15);
        assert_eq!(hyp.stackup[0].z0, hyp.stackup[1].z1);
    }

    #[test]
    fn outer_copper_is_in_air() {
        let mut hyp = HypFile::default();
        hyp.set_units(false, false);
        hyp.add_metal_layer(
            LayerType::Signal,
            0.000035,
            None,
            None,
            None,
            None,
            None,
            Some("Top".into()),
            None,
            None,
        );
        hyp.add_dielectric_layer(0.1, Some(4.4), None, None, None, Some("DL".into()), None);
        hyp.add_metal_layer(
            LayerType::Signal,
            0.000035,
            None,
            None,
            None,
            None,
            None,
            Some("Bottom".into()),
            None,
            None,
        );
        assert_eq!(hyp.stackup[0].epsilon_r, 1.0);
        assert_eq!(hyp.stackup[2].epsilon_r, 1.0);
    }

    #[test]
    fn arc_full_circle_has_eight_segments() {
        let hyp = HypFile::default();
        let arc = hyp.arc2poly(1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, false);
        assert_eq!(arc.vertex.len(), 9);
        assert_eq!(arc.vertex.first(), arc.vertex.last());
    }

    #[test]
    fn segment_polygon_contains_two_caps() {
        let hyp = HypFile::default();
        let seg = hyp.segment2poly(0.0, 0.0, 1.0, 0.0, 0.2);
        // Two half circles of 8 segments each: 4 + 1 + 4 + 1 points.
        assert_eq!(seg.vertex.len(), 10);
    }

    #[test]
    fn pad_rotation_normalisation() {
        let hyp = HypFile::default();
        let pad = Pad {
            layer_name: "Top".into(),
            pad_type: PadType::Metal,
            pad_shape: PadShape::Rectangular,
            pad_sx: 0.2,
            pad_sy: 0.1,
            pad_angle: 90.0,
            thermal_clear_shape: PadShape::Oval,
            thermal_clear_sx: 0.0,
            thermal_clear_sy: 0.0,
            thermal_clear_angle: 0.0,
        };
        let rect = hyp.pad2poly(0.0, 0.0, &pad);
        // 90 degrees swaps sx and sy, so the polygon is 0.1 wide, 0.2 high.
        let xs: Vec<f64> = rect.vertex.iter().map(|p| p.x).collect();
        let ys: Vec<f64> = rect.vertex.iter().map(|p| p.y).collect();
        let width = xs.iter().cloned().fold(f64::MIN, f64::max)
            - xs.iter().cloned().fold(f64::MAX, f64::min);
        let height = ys.iter().cloned().fold(f64::MIN, f64::max)
            - ys.iter().cloned().fold(f64::MAX, f64::min);
        assert!((width - 0.1).abs() < 1e-12);
        assert!((height - 0.2).abs() < 1e-12);
    }

    #[test]
    fn pad_rotation_by_45_degrees_is_applied() {
        let hyp = HypFile::default();
        let pad = Pad {
            layer_name: "Top".into(),
            pad_type: PadType::Metal,
            pad_shape: PadShape::Rectangular,
            pad_sx: 0.2,
            pad_sy: 0.1,
            pad_angle: 45.0,
            thermal_clear_shape: PadShape::Oval,
            thermal_clear_sx: 0.0,
            thermal_clear_sy: 0.0,
            thermal_clear_angle: 0.0,
        };
        let rect = hyp.pad2poly(0.0, 0.0, &pad);
        let c = std::f64::consts::FRAC_1_SQRT_2;
        assert!((rect.vertex[0].x - (0.1 * c - 0.05 * c)).abs() < 1e-12);
        assert!((rect.vertex[0].y - (0.1 * c + 0.05 * c)).abs() < 1e-12);
    }

    #[test]
    fn perimeter_segments_form_closed_loops() {
        let mut hyp = HypFile::default();
        for (x1, y1, x2, y2) in [
            (0.0, 0.0, 1.0, 0.0),
            (1.0, 0.0, 1.0, 1.0),
            (1.0, 1.0, 0.0, 1.0),
            (0.0, 1.0, 0.0, 0.0),
        ] {
            hyp.add_perimeter_polygon(HypPolygon {
                vertex: vec![HypPoint::new(x1, y1), HypPoint::new(x2, y2)],
                ..HypPolygon::default()
            });
        }
        assert_eq!(hyp.board.edge.len(), 1);
        assert_eq!(hyp.board.edge[0].vertex.len(), 8);
        assert!(hyp.board.edge[0].positive);
    }

    #[test]
    fn polygon_ids_are_assigned_like_native() {
        let mut net = Net {
            net_name: "N".into(),
            metal: PolygonMap::new(),
            plane_separation: -1.0,
            via: Vec::new(),
            unrouted_segment: Vec::new(),
            pin: Vec::new(),
            attribute: Vec::new(),
        };
        HypFile::add_polygon_to_net(&mut net, square(0.0, 0.0, 1.0));
        HypFile::add_polygon_to_net(&mut net, square(1.0, 0.0, 1.0));
        let ids: Vec<i32> = net.metal.keys().copied().collect();
        assert_eq!(ids, vec![-3, -2]);
    }
}
