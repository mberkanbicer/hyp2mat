//! Canonical CSXCAD model dump for the RM-109 differential check.
//!
//! Reads every CSXCAD XML document given on the command line with the Rust
//! `csxcad-core` parser and prints the same summary as the native
//! `tests/native/native_model_dump.cpp`, so `run_native_read_check.sh` can diff
//! the two. Nothing here changes the library; it only exercises the public
//! reader.
//!
//! ```text
//! cargo run --example model_dump -- file.xml
//! ```

use std::path::Path;
use std::process::ExitCode;

use csxcad_core::material::MaterialField;
use csxcad_core::model::Structure;

fn fixed(label: &str, value: f64) {
    println!("{label} {value:.12}");
}

fn material_value(structure: &Structure, index: usize, field: MaterialField) -> f64 {
    structure
        .property_by_index(index)
        .and_then(|property| property.material())
        .and_then(|material| material.value(field, 0, None).ok().flatten())
        .unwrap_or(f64::NAN)
}

fn dump(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned());
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            println!("file {name}");
            println!("ERROR {error}");
            return false;
        }
    };
    let mut structure = match Structure::from_xml_str(&text) {
        Ok(structure) => structure,
        Err(error) => {
            println!("file {name}");
            println!("ERROR {error}");
            return false;
        }
    };

    println!("file {name}");
    println!("properties {}", structure.property_count());
    println!("primitives {}", structure.primitive_count());

    let (lines_x, lines_y, lines_z, delta_unit) = {
        let grid = structure.grid_mut();
        (
            grid.line_count(0),
            grid.line_count(1),
            grid.line_count(2),
            grid.delta_unit(),
        )
    };
    println!("grid {lines_x} {lines_y} {lines_z}");
    fixed("delta_unit", delta_unit);

    let background = structure.background_material();
    fixed(
        "background_epsilon",
        background.value(MaterialField::Epsilon),
    );
    fixed("background_mue", background.value(MaterialField::Mue));
    fixed("background_kappa", background.value(MaterialField::Kappa));
    fixed("background_sigma", background.value(MaterialField::Sigma));

    for index in 0..structure.property_count() {
        let Some(property) = structure.property_by_index(index) else {
            println!("property <missing> <missing>");
            continue;
        };
        println!(
            "property {} {}",
            property.property_type().xml_string().unwrap_or("Any"),
            property.name()
        );
        if property.material().is_some() {
            fixed(
                "property_epsilon",
                material_value(&structure, index, MaterialField::Epsilon),
            );
            fixed(
                "property_mue",
                material_value(&structure, index, MaterialField::Mue),
            );
            fixed(
                "property_kappa",
                material_value(&structure, index, MaterialField::Kappa),
            );
            fixed(
                "property_sigma",
                material_value(&structure, index, MaterialField::Sigma),
            );
        }
    }

    for primitive in structure.all_primitives(false, csxcad_core::model::PropertyType::Any) {
        let owner = primitive
            .property_id()
            .and_then(|id| structure.property(id))
            .map(|property| property.name())
            .unwrap_or("<orphan>");
        println!(
            "primitive {} {} {}",
            primitive.primitive_type().type_name(),
            primitive.priority(),
            owner
        );
    }
    true
}

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: model_dump <file.xml>...");
        return ExitCode::from(2);
    }
    let mut status = ExitCode::SUCCESS;
    for path in &paths {
        if !dump(path) {
            status = ExitCode::FAILURE;
        }
    }
    status
}
