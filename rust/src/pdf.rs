//! PDF output writer — port of `hyp2mat/lib/pdf.cc` (libhpdf).
//! Generates a minimal PDF document with PCB layers as vector graphics.

use crate::model::PCB;

/// Write a PCB as a PDF document.
/// Returns the PDF content as a String.
pub fn write_pdf(pcb: &PCB) -> Result<String, String> {
    let mut pdf = String::new();

    // Compute board bounds
    let bounds = compute_bounds(pcb);

    // PDF header
    pdf.push_str("%PDF-1.4\n");

    // Object 1: Catalog
    let obj1_pos = pdf.len();
    pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Object 2: Pages
    let obj2_pos = pdf.len();
    pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Object 3: Page
    let obj3_pos = pdf.len();
    pdf.push_str(&format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [{:.2} {:2} {:.2} {:.2}] /Resources << /ProcSet [/PDF /Text /ImageB /ImageC /ImageI] /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n",
        bounds.x0, bounds.y0, bounds.x1, bounds.y1
    ));

    // Object 4: Font
    let obj4_pos = pdf.len();
    pdf.push_str("4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

    // Object 5: Contents stream
    let mut content = String::new();
    content.push_str("BT\n/F1 12 Tf\n72 720 Td\n(PCB Export from hyp2mat) Tj\nET\n");

    // Draw board outline
    if !pcb.board.polys.is_empty() {
        content.push_str("0.5 0.5 0.5 RG\n"); // gray stroke
        content.push_str("0.9 0.9 0.9 rg\n"); // light gray fill
        for poly in &pcb.board.polys {
            if poly.poly.points.is_empty() {
                continue;
            }
            content.push_str(&format!(
                "{} {} m\n",
                poly.poly.points[0].x, poly.poly.points[0].y
            ));
            for pt in &poly.poly.points[1..] {
                content.push_str(&format!("{} {} l\n", pt.x, pt.y));
            }
            content.push_str("h f S\n");
        }
    }

    // Draw each layer's metal
    let layer_colors: &[(f64, f64, f64)] = &[
        (0.8, 0.2, 0.2), // red
        (0.2, 0.8, 0.2), // green
        (0.2, 0.2, 0.8), // blue
        (0.8, 0.8, 0.2), // yellow
        (0.8, 0.2, 0.8), // magenta
        (0.2, 0.8, 0.8), // cyan
    ];
    for (i, layer) in pcb.stackup.iter().enumerate() {
        let (r, g, b) = layer_colors[i % layer_colors.len()];
        content.push_str(&format!("{} {} {} rg\n", r, g, b));
        for poly in &layer.metal.polys {
            if poly.poly.points.is_empty() {
                continue;
            }
            content.push_str(&format!(
                "{} {} m\n",
                poly.poly.points[0].x, poly.poly.points[0].y
            ));
            for pt in &poly.poly.points[1..] {
                content.push_str(&format!("{} {} l\n", pt.x, pt.y));
            }
            content.push_str("h f\n");
        }
    }

    // Draw vias as circles
    content.push_str("0 0 0 rg\n"); // black fill
    for via in &pcb.via {
        // Approximate circle with a filled rectangle
        content.push_str(&format!(
            "{} {} {} {} re f\n",
            via.x - via.radius,
            via.y - via.radius,
            via.radius * 2.0,
            via.radius * 2.0
        ));
    }

    // Draw pins
    content.push_str("1 0 0 rg\n"); // red fill
    for pin in &pcb.pin {
        content.push_str(&format!(
            "{} {} {} {} re f\n",
            pin.x - 0.5,
            pin.y - 0.5,
            1.0,
            1.0
        ));
    }

    let content_bytes = content.as_bytes();
    let obj5_pos = pdf.len();
    pdf.push_str(&format!(
        "5 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        content_bytes.len(),
        content
    ));

    // Xref table
    let xref_pos = pdf.len();
    pdf.push_str("xref\n");
    pdf.push_str("0 6\n");
    pdf.push_str("0000000000 65535 f \n");
    pdf.push_str(&format!("{:010} 00000 n \n", obj1_pos));
    pdf.push_str(&format!("{:010} 00000 n \n", obj2_pos));
    pdf.push_str(&format!("{:010} 00000 n \n", obj3_pos));
    pdf.push_str(&format!("{:010} 00000 n \n", obj4_pos));
    pdf.push_str(&format!("{:010} 00000 n \n", obj5_pos));

    // Trailer
    pdf.push_str("\ntrailer\n<< /Size 6 /Root 1 0 R >>\n");
    pdf.push_str(&format!("startxref\n{}\n%%EOF\n", xref_pos));

    Ok(pdf)
}

/// Compute the bounding box of the entire PCB.
fn compute_bounds(pcb: &PCB) -> crate::model::Bounds {
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;

    let mut update = |x: f64, y: f64| {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    };

    for poly in &pcb.board.polys {
        for pt in &poly.poly.points {
            update(pt.x, pt.y);
        }
    }
    for layer in &pcb.stackup {
        for poly in &layer.metal.polys {
            for pt in &poly.poly.points {
                update(pt.x, pt.y);
            }
        }
    }
    for via in &pcb.via {
        update(via.x, via.y);
    }
    for pin in &pcb.pin {
        update(pin.x, pin.y);
    }

    if x0.is_infinite() {
        x0 = 0.0;
        y0 = 0.0;
        x1 = 100.0;
        y1 = 100.0;
    }

    let z0 = pcb.stackup.first().map_or(0.0, |l| l.z0);
    let z1 = pcb.stackup.last().map_or(0.0, |l| l.z1);

    crate::model::Bounds {
        x0,
        y0,
        x1,
        y1,
        z0,
        z1,
    }
}
