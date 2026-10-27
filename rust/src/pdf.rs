//! PDF export — port of `hyp2mat/lib/pdf.cc`.
//!
//! The native writer uses libharu (HPDF) and produces one composite page
//! followed by one page per copper layer, sized to the board in PostScript
//! points (1/72 inch) and coloured through [`Palette`]. This module writes the
//! same page structure and vector content directly, without a C library.

use crate::csxcad::format_cpp_float;
use crate::model::{Bounds, FloatPolygon, FloatPolygons, Via, PCB};

/// PDF colour options (`--hue`, `--saturation`, `--brightness`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfOptions {
    pub hue: f64,
    pub saturation: f64,
    pub brightness: f64,
}

impl Default for PdfOptions {
    fn default() -> Self {
        // Library defaults; out-of-range values leave the palette defaults.
        PdfOptions {
            hue: -1.0,
            saturation: -1.0,
            brightness: -1.0,
        }
    }
}

/// Colour palette, port of `hyp2mat/lib/palette.cc` (after Martin Ankerl,
/// "How to Generate Random Colors Programmatically").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    hue: f64,
    saturation: f64,
    value: f64,
    step: f64,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            hue: 0.69,
            saturation: 0.99,
            value: 0.99,
            step: (5.0f64.sqrt() - 1.0) / 2.0,
        }
    }
}

impl Palette {
    pub fn new(hue: f64, saturation: f64, value: f64) -> Self {
        let mut palette = Palette::default();
        palette.set_palette(hue, saturation, value);
        palette
    }

    /// Native `SetPalette`: only in-range values are used.
    pub fn set_palette(&mut self, hue: f64, saturation: f64, value: f64) {
        if (0.0..=1.0).contains(&hue) {
            self.hue = hue;
        }
        if (0.0..=1.0).contains(&saturation) {
            self.saturation = saturation;
        }
        if (0.0..=1.0).contains(&value) {
            self.value = value;
        }
    }

    /// Native `SetNumberOfColors`.
    pub fn set_number_of_colors(&mut self, number_of_colors: usize) {
        if number_of_colors > 0 {
            self.step = 1.0 / number_of_colors as f64;
        } else {
            self.step = (5.0f64.sqrt() - 1.0) / 2.0;
        }
    }

    /// Native `Color`.
    pub fn color(&self, color: usize) -> (f64, f64, f64) {
        let h = (self.hue + color as f64 * self.step).fract();
        let h = if h < 0.0 { h + 1.0 } else { h };
        let h6 = h * 6.0;
        let hi = h6.floor();
        let f = h6 - hi;
        let v = self.value;
        let p = v * (1.0 - self.saturation);
        let q = v * (1.0 - f * self.saturation);
        let t = v * (1.0 - (1.0 - f) * self.saturation);
        match (hi + 0.5) as i32 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => (0.0, 0.0, 0.0),
        }
    }
}

/// Points per metre (`1 / (2.54 / 100 / 72)` in the native implementation).
pub const POINTS_PER_METRE: f64 = 1.0 / (2.54 / 100.0 / 72.0);
const TEXT_HEIGHT: f64 = 10.0;
const MARGIN: f64 = TEXT_HEIGHT;

/// Write a PCB as a PDF document.
pub fn write_pdf(pcb: &PCB, options: &PdfOptions) -> Result<Vec<u8>, String> {
    let bounds = crate::pcb::get_bounds(pcb);
    let x_min = bounds.x0 * POINTS_PER_METRE;
    let x_max = bounds.x1 * POINTS_PER_METRE;
    let y_min = bounds.y0 * POINTS_PER_METRE;
    let y_max = bounds.y1 * POINTS_PER_METRE;

    let mut width = x_max - x_min + 2.0 * MARGIN;
    let mut height = y_max - y_min + 2.0 * MARGIN + TEXT_HEIGHT;
    if width < 3.0 {
        width = 3.0;
    }
    if height < 3.0 {
        height = 3.0;
    }

    let copper_layers: Vec<&crate::model::Layer> = pcb
        .stackup
        .iter()
        .rev()
        .filter(|l| l.layer_type != "dielectric")
        .collect();

    let mut palette = Palette::new(options.hue, options.saturation, options.brightness);
    palette.set_number_of_colors(copper_layers.len());

    let geometry = PageGeometry {
        x_min,
        y_min,
        margin: MARGIN,
        bounds,
    };

    let mut pages: Vec<(String, String)> = Vec::new();
    pages.push((
        "Composite".to_string(),
        composite_content(pcb, &geometry, &palette, &copper_layers),
    ));
    for (index, layer) in copper_layers.iter().enumerate() {
        pages.push((
            layer.name.clone(),
            layer_content(pcb, &geometry, &palette, index, layer),
        ));
    }

    Ok(build_pdf(width, height, &pages))
}

struct PageGeometry {
    x_min: f64,
    y_min: f64,
    margin: f64,
    bounds: Bounds,
}

impl PageGeometry {
    fn point(&self, point: &crate::model::FloatPoint) -> (f64, f64) {
        (
            point.x * POINTS_PER_METRE - self.x_min + self.margin,
            point.y * POINTS_PER_METRE - self.y_min + self.margin,
        )
    }

    fn caption_position(&self) -> (f64, f64) {
        let height = (self.bounds.y1 - self.bounds.y0) * POINTS_PER_METRE;
        (self.margin, height + self.margin + TEXT_HEIGHT)
    }
}

fn set_color(content: &mut String, (r, g, b): (f64, f64, f64)) {
    content.push_str(&format!(
        "{} {} {} RG\n{} {} {} rg\n",
        format_cpp_float(r),
        format_cpp_float(g),
        format_cpp_float(b),
        format_cpp_float(r),
        format_cpp_float(g),
        format_cpp_float(b)
    ));
}

fn set_color_black(content: &mut String) {
    set_color(content, (0.0, 0.0, 0.0));
}

fn draw_polygon(content: &mut String, geometry: &PageGeometry, polygon: &FloatPolygon) {
    let mut first = true;
    let mut old: (f64, f64) = (0.0, 0.0);
    for point in &polygon.points {
        let (px, py) = geometry.point(point);
        if first {
            content.push_str(&format!(
                "{} {} m\n",
                format_cpp_float(px),
                format_cpp_float(py)
            ));
            first = false;
        } else if px != old.0 || py != old.1 {
            content.push_str(&format!(
                "{} {} l\n",
                format_cpp_float(px),
                format_cpp_float(py)
            ));
        }
        old = (px, py);
    }
    content.push_str("h\n");
}

fn draw_polygons(content: &mut String, geometry: &PageGeometry, polygons: &FloatPolygons) {
    for poly in &polygons.polys {
        draw_polygon(content, geometry, &poly.poly);
    }
}

fn draw_circle(content: &mut String, geometry: &PageGeometry, via: &Via) {
    let (cx, cy) = geometry.point(&crate::model::FloatPoint { x: via.x, y: via.y });
    let r = via.radius * POINTS_PER_METRE;
    // Four cubic Bezier quarter arcs, as libharu's HPDF_Page_Circle.
    let k = 0.5522847498307936 * r;
    let f = format_cpp_float;
    content.push_str(&format!("{} {} m\n", f(cx + r), f(cy)));
    content.push_str(&format!(
        "{} {} {} {} {} {} c\n",
        f(cx + r),
        f(cy + k),
        f(cx + k),
        f(cy + r),
        f(cx),
        f(cy + r)
    ));
    content.push_str(&format!(
        "{} {} {} {} {} {} c\n",
        f(cx - k),
        f(cy + r),
        f(cx - r),
        f(cy + k),
        f(cx - r),
        f(cy)
    ));
    content.push_str(&format!(
        "{} {} {} {} {} {} c\n",
        f(cx - r),
        f(cy - k),
        f(cx - k),
        f(cy - r),
        f(cx),
        f(cy - r)
    ));
    content.push_str(&format!(
        "{} {} {} {} {} {} c\n",
        f(cx + k),
        f(cy - r),
        f(cx + r),
        f(cy - k),
        f(cx + r),
        f(cy)
    ));
    content.push_str("h\n");
}

fn draw_caption(content: &mut String, geometry: &PageGeometry, text: &str) {
    let (x, y) = geometry.caption_position();
    let escaped = text
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");
    content.push_str(&format!(
        "BT\n/F1 {} Tf\n{} {} Td\n({}) Tj\nET\n",
        format_cpp_float(TEXT_HEIGHT),
        format_cpp_float(x),
        format_cpp_float(y),
        escaped
    ));
}

fn composite_content(
    pcb: &PCB,
    geometry: &PageGeometry,
    palette: &Palette,
    copper_layers: &[&crate::model::Layer],
) -> String {
    let mut content = String::new();
    content.push_str("0 w\n");
    draw_caption(&mut content, geometry, "Composite");

    set_color_black(&mut content);
    draw_polygons(&mut content, geometry, &pcb.board);
    content.push_str("S\n");

    for (index, layer) in copper_layers.iter().enumerate() {
        set_color(&mut content, palette.color(index));
        draw_polygons(&mut content, geometry, &layer.metal);
        content.push_str("S\n");
    }

    set_color_black(&mut content);
    for via in &pcb.via {
        draw_circle(&mut content, geometry, via);
    }
    content.push_str("f\n");
    content
}

fn layer_content(
    pcb: &PCB,
    geometry: &PageGeometry,
    palette: &Palette,
    index: usize,
    layer: &crate::model::Layer,
) -> String {
    let mut content = String::new();
    content.push_str("0 w\n");
    draw_caption(&mut content, geometry, &layer.name);

    set_color_black(&mut content);
    draw_polygons(&mut content, geometry, &pcb.board);
    content.push_str("S\n");

    set_color(&mut content, palette.color(index));
    draw_polygons(&mut content, geometry, &layer.metal);
    content.push_str("f\n");

    // Blind vias are only drawn when they pass through this layer.
    set_color_black(&mut content);
    for via in &pcb.via {
        if via.z0 <= layer.z0 && via.z1 >= layer.z1 {
            draw_circle(&mut content, geometry, via);
        }
    }
    content.push_str("f\n");
    content
}

/// Assemble a PDF 1.4 document with the given pages (caption, content stream).
fn build_pdf(width: f64, height: f64, pages: &[(String, String)]) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();
    let page_count = pages.len();

    // 1: catalog, 2: pages, 3: info, 4: font, then page/content pairs.
    let kids: Vec<String> = (0..page_count)
        .map(|i| format!("{} 0 R", 5 + i * 2))
        .collect();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    objects.push(format!(
        "<< /Type /Pages /Kids [ {} ] /Count {} >>",
        kids.join(" "),
        page_count
    ));
    objects.push("<< /Producer (hyp2mat-rs) /Creator (hyp2mat) >>".to_string());
    objects.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /StandardEncoding >>"
            .to_string(),
    );

    for (index, (caption, content)) in pages.iter().enumerate() {
        let _ = caption;
        let content_id = 6 + index * 2;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [ 0 0 {} {} ] /Contents {} 0 R /Resources << /ProcSet [ /PDF /Text /ImageB /ImageC /ImageI ] /Font << /F1 4 0 R >> >> >>",
            format_cpp_float(width),
            format_cpp_float(height),
            content_id
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            content
        ));
    }

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
    }

    let xref_offset = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
    pdf.push_str("0000000000 65535 f \n");
    for offset in &offsets {
        pdf.push_str(&format!("{:010} 00000 n \n", offset));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R /Info 3 0 R >>\nstartxref\n{}\n%%EOF\n",
        objects.len() + 1,
        xref_offset
    ));
    pdf.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FloatPoint, FloatPoly, FloatPolygon, FloatPolygons, Layer};

    fn sample_pcb() -> PCB {
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
        let metal = FloatPolygons {
            polys: vec![FloatPoly {
                poly: FloatPolygon {
                    points: vec![
                        FloatPoint { x: 0.004, y: 0.004 },
                        FloatPoint { x: 0.004, y: 0.006 },
                        FloatPoint { x: 0.006, y: 0.006 },
                        FloatPoint { x: 0.006, y: 0.004 },
                    ],
                },
                is_hole: false,
                nesting_level: 0,
            }],
        };
        let layer = |name: &str, kind: &str, z0: f64, z1: f64| Layer {
            name: name.into(),
            layer_type: kind.into(),
            thickness: z1 - z0,
            z0,
            z1,
            epsilon_r: 4.4,
            loss_tangent: 0.02,
            bulk_resistivity: 1.724e-8,
            metal: metal.clone(),
        };
        PCB {
            board,
            stackup: vec![
                layer("Top", "signal", 1e-3, 1e-3),
                layer("DL01", "dielectric", 0.0, 1e-3),
                layer("Bottom", "signal", 0.0, 0.0),
            ],
            via: vec![Via {
                x: 0.01,
                y: 0.005,
                z0: 0.0,
                z1: 1e-3,
                radius: 2e-4,
            }],
            via_plating_thickness: 0.0,
            device: Vec::new(),
            pin: Vec::new(),
        }
    }

    fn parse_pdf(pdf: &[u8]) -> Vec<(usize, String)> {
        let text = String::from_utf8(pdf.to_vec()).expect("ascii pdf");
        let mut objects = Vec::new();
        let mut index = 0;
        while let Some(found) = text[index..].find(" 0 obj\n") {
            let start = index + found + " 0 obj\n".len();
            let number_start = text[..index + found]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0);
            let number: usize = text[number_start..index + found].trim().parse().unwrap();
            let end = text[start..].find("\nendobj").expect("object end") + start;
            objects.push((number, text[start..end].to_string()));
            index = end;
        }
        objects
    }

    #[test]
    fn palette_matches_native_hsv_conversion() {
        let mut palette = Palette::new(0.0, 0.6, 0.9);
        palette.set_number_of_colors(2);
        let (r, g, b) = palette.color(0);
        assert!((r - 0.9).abs() < 1e-12);
        assert!((g - 0.36).abs() < 1e-12);
        assert!((b - 0.36).abs() < 1e-12);
        // Second colour: hue = 0.5 -> cyan-ish.
        let (r, g, b) = palette.color(1);
        assert!((r - 0.36).abs() < 1e-12);
        assert!((g - 0.9).abs() < 1e-12);
        assert!((b - 0.9).abs() < 1e-12);
    }

    #[test]
    fn pdf_has_one_composite_and_one_page_per_copper_layer() {
        let pcb = sample_pcb();
        let pdf = write_pdf(&pcb, &PdfOptions::default()).unwrap();
        let text = String::from_utf8(pdf.clone()).unwrap();
        assert!(text.starts_with("%PDF-1.4\n"));
        assert!(text.ends_with("%%EOF\n"));
        assert!(text.contains("/Count 3"));
        let objects = parse_pdf(&pdf);
        assert_eq!(objects.len(), 4 + 3 * 2);
        // MediaBox: 20 mm x 10 mm = 56.69 x 28.35 points plus margins.
        assert!(text.contains("/MediaBox [ 0 0 76.6929 58.3465 ]"), "{text}");
        assert!(text.contains("(Composite) Tj"));
        assert!(text.contains("(Bottom) Tj"));
        assert!(text.contains("(Top) Tj"));
    }

    #[test]
    fn pdf_geometry_uses_points_and_margins() {
        let pcb = sample_pcb();
        let pdf = write_pdf(&pcb, &PdfOptions::default()).unwrap();
        let text = String::from_utf8(pdf).unwrap();
        // Board corner (0, 0) maps to (10, 10) points; (0.02, 0.01) maps to
        // (66.69, 38.35).
        assert!(text.contains("10 10 m\n"), "{text}");
        assert!(text.contains("66.6929 10 l\n"), "{text}");
        assert!(text.contains("66.6929 38.3465 l\n"), "{text}");
    }

    #[test]
    fn pdf_xref_offsets_point_at_objects() {
        let pcb = sample_pcb();
        let pdf = write_pdf(&pcb, &PdfOptions::default()).unwrap();
        let text = String::from_utf8(pdf).unwrap();
        let xref = text.find("\nxref\n").expect("xref") + 1;
        let startxref = text.rfind("startxref\n").expect("startxref");
        assert_eq!(
            text[startxref + "startxref\n".len()..]
                .lines()
                .next()
                .unwrap(),
            xref.to_string()
        );
        let mut lines = text[xref..].lines();
        lines.next();
        let header = lines.next().unwrap();
        assert_eq!(header, "0 11");
        for (index, line) in lines.take(10).enumerate() {
            if index == 0 {
                assert_eq!(line, "0000000000 65535 f ");
                continue;
            }
            let offset: usize = line[..10].parse().unwrap();
            assert!(
                text[offset..].starts_with(&format!("{index} 0 obj")),
                "xref entry {index} points at {:?}",
                &text[offset..offset + 12]
            );
        }
    }

    #[test]
    fn pdf_caption_is_escaped() {
        let mut pcb = sample_pcb();
        pcb.stackup[0].name = "A(1)".to_string();
        let pdf = write_pdf(&pcb, &PdfOptions::default()).unwrap();
        let text = String::from_utf8(pdf).unwrap();
        assert!(text.contains("(A\\(1\\)) Tj"));
    }

    #[test]
    fn pdf_empty_board_is_valid() {
        let mut pcb = PCB::default();
        pcb.stackup.clear();
        let pdf = write_pdf(&pcb, &PdfOptions::default()).unwrap();
        let text = String::from_utf8(pdf).unwrap();
        assert!(text.contains("/Count 1"));
        // No geometry: width/height are the margins only.
        assert!(text.contains("/MediaBox [ 0 0 20 30 ]"));
    }
}
