//! PCB model types — port of `hyp2mat/lib/hyp2mat.h`.

/// A 2D point with floating point coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatPoint {
    pub x: f64,
    pub y: f64,
}

/// A single polygon (list of points).
#[derive(Debug, Clone, PartialEq)]
pub struct FloatPolygon {
    pub points: Vec<FloatPoint>,
}

/// A polygon with hole flag and nesting level.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatPoly {
    pub poly: FloatPolygon,
    pub is_hole: bool,
    pub nesting_level: i32,
}

/// A list of polygons (with hole/nesting info).
#[derive(Debug, Clone, PartialEq)]
pub struct FloatPolygons {
    pub polys: Vec<FloatPoly>,
}

/// Bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub z0: f64,
    pub z1: f64,
}

/// A PCB layer.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub layer_type: String,
    pub thickness: f64,
    pub z0: f64,
    pub z1: f64,
    pub epsilon_r: f64,
    pub loss_tangent: f64,
    pub bulk_resistivity: f64,
    pub metal: FloatPolygons,
}

/// A via.
#[derive(Debug, Clone, PartialEq)]
pub struct Via {
    pub x: f64,
    pub y: f64,
    pub z0: f64,
    pub z1: f64,
    pub radius: f64,
}

/// A device.
#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub name: String,
    pub ref_: String,
    pub value_type: String,
    pub value_float: f64,
    pub value_string: String,
    pub layer_name: String,
}

/// A pin.
#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    pub x: f64,
    pub y: f64,
    pub z0: f64,
    pub z1: f64,
    pub ref_: String,
    pub layer_name: String,
    pub metal: FloatPolygon,
}

/// A complete PCB.
#[derive(Debug, Clone, PartialEq)]
pub struct PCB {
    pub board: FloatPolygons,
    pub stackup: Vec<Layer>,
    pub via: Vec<Via>,
    /// Via plating thickness (taken from the first metal layer).
    pub via_plating_thickness: f64,
    pub device: Vec<Device>,
    pub pin: Vec<Pin>,
}

impl Default for PCB {
    fn default() -> Self {
        PCB {
            board: FloatPolygons { polys: Vec::new() },
            stackup: Vec::new(),
            via: Vec::new(),
            via_plating_thickness: 0.0,
            device: Vec::new(),
            pin: Vec::new(),
        }
    }
}
