//! Shared helpers for the hyp2mat-rs integration tests.
//!
//! Contains a small SHA-256 implementation (the crate has no hashing
//! dependency), temporary directory handling, and helpers to parse and compare
//! the native CSXCAD/QucsStudio output formats.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// SHA-256
// ---------------------------------------------------------------------------

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 of `data`, lowercase hexadecimal (64 characters).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    state.iter().map(|word| format!("{word:08x}")).collect()
}

// ---------------------------------------------------------------------------
// Paths and temporary directories
// ---------------------------------------------------------------------------

pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn reference_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reference")
}

pub fn fixture_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixtures_dir())
        .expect("fixtures directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("hyp") {
                Some(path.file_stem().unwrap().to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary directory that is removed when dropped.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "hyp2mat-rs-test-{}-{}-{}",
            std::process::id(),
            label,
            id
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        TempDir { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Path of the compiled `hyp2mat-rs` binary.
pub fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_hyp2mat")
}

pub struct CliOutput {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

impl CliOutput {
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

/// Run the CLI with arguments and optional standard input.
pub fn run_cli(args: &[&str], stdin: Option<&[u8]>) -> CliOutput {
    run_cli_in(None, args, stdin)
}

/// Run the CLI in a specific working directory (used for `-d` dumps).
pub fn run_cli_in(directory: Option<&Path>, args: &[&str], stdin: Option<&[u8]>) -> CliOutput {
    use std::io::Write;
    use std::process::Stdio;

    let mut command = Command::new(binary());
    command.args(args);
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }
    let mut child = command.spawn().expect("spawn hyp2mat-rs");
    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(input)
            .expect("write stdin");
    }
    let output = child.wait_with_output().expect("wait for hyp2mat-rs");
    CliOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

// ---------------------------------------------------------------------------
// CSXCAD MATLAB script comparison
// ---------------------------------------------------------------------------

/// Polygon vertices (metres) in the order they appear in a native CSXCAD
/// script, one vector per `pgon` block.
pub fn parse_script_polygons(script: &str) -> Vec<Vec<[f64; 2]>> {
    let mut polygons = Vec::new();
    let mut current: Option<Vec<[f64; 2]>> = None;
    for line in script.lines() {
        let line = line.trim();
        if line == "pgon = [];" {
            if let Some(polygon) = current.take() {
                polygons.push(polygon);
            }
            current = Some(Vec::new());
        } else if line.starts_with("pgon(:, end+1) = [") {
            if current.is_none() {
                current = Some(Vec::new());
            }
            let inner = line
                .trim_start_matches("pgon(:, end+1) = [")
                .trim_end_matches("];");
            let mut parts = inner.split(';');
            let x: f64 = parts.next().expect("x").parse().expect("x number");
            let y: f64 = parts.next().expect("y").parse().expect("y number");
            current.as_mut().unwrap().push([x, y]);
        } else if line.starts_with("CSX = AddPolygon") || line.starts_with("CSX = AddLinPoly") {
            if let Some(polygon) = current.take() {
                if !polygon.is_empty() {
                    polygons.push(polygon);
                }
            }
        }
    }
    if let Some(polygon) = current {
        if !polygon.is_empty() {
            polygons.push(polygon);
        }
    }
    polygons
}

/// Canonical form of a polygon: vertices rounded to the 10 µm grid, rotated to
/// the lexicographically smallest sequence, in the direction that sorts first.
pub fn canonical_polygon(polygon: &[[f64; 2]]) -> Vec<(i64, i64)> {
    let grid = |v: f64| (v / 1e-5).round() as i64;
    let points: Vec<(i64, i64)> = polygon.iter().map(|p| (grid(p[0]), grid(p[1]))).collect();
    let n = points.len();
    if n == 0 {
        return points;
    }
    let mut best: Option<Vec<(i64, i64)>> = None;
    for candidate in [points.clone(), points.iter().rev().copied().collect()] {
        for shift in 0..n {
            let rotated: Vec<(i64, i64)> = candidate[shift..]
                .iter()
                .chain(candidate[..shift].iter())
                .copied()
                .collect();
            if best.as_ref().is_none_or(|current| &rotated < current) {
                best = Some(rotated);
            }
        }
    }
    best.unwrap()
}

pub fn polygon_area(polygon: &[(f64, f64)]) -> f64 {
    let n = polygon.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i].0 * polygon[j].1 - polygon[j].0 * polygon[i].1;
    }
    area / 2.0
}

/// Largest distance between two equally sized vertex cycles, over cyclic
/// rotations and both directions.
pub fn cyclic_distance(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    if n == 0 {
        return 0.0;
    }
    let mut best = f64::INFINITY;
    for direction in [b.to_vec(), b.iter().rev().copied().collect()] {
        for shift in 0..n {
            let mut worst: f64 = 0.0;
            for i in 0..n {
                let p = a[i];
                let q = direction[(i + shift) % n];
                let dx = p[0] - q[0];
                let dy = p[1] - q[1];
                worst = worst.max((dx * dx + dy * dy).sqrt());
            }
            best = best.min(worst);
        }
    }
    best
}

pub fn to_float_pairs(polygon: &[[f64; 2]]) -> Vec<(f64, f64)> {
    polygon.iter().map(|p| (p[0], p[1])).collect()
}

// ---------------------------------------------------------------------------
// QucsStudio layout comparison
// ---------------------------------------------------------------------------

/// Absolute-coordinate polygons listed in a QucsStudio layout (`Polygon` and
/// `Rectangle` items of the exported layer).
pub fn parse_qucs_items(text: &str) -> Vec<Vec<[f64; 2]>> {
    let mut polygons = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "Polygon" => {
                // Polygon metalN cx cy |dx|dy|dx|dy...
                if parts.len() < 4 {
                    continue;
                }
                let cx: f64 = parts[2].parse().unwrap();
                let cy: f64 = parts[3].parse().unwrap();
                let mut polygon = Vec::new();
                let coords: Vec<&str> = parts[4..]
                    .iter()
                    .flat_map(|segment| segment.split('|'))
                    .filter(|value| !value.is_empty())
                    .collect();
                for pair in coords.chunks(2) {
                    if pair.len() != 2 {
                        break;
                    }
                    let dx: f64 = pair[0].parse().unwrap();
                    let dy: f64 = pair[1].parse().unwrap();
                    polygon.push([cx + dx, cy + dy]);
                }
                polygons.push(polygon);
            }
            "Rectangle" => {
                // Rectangle metalN cx cy 0|0|dx|dy
                let cx: f64 = parts[2].parse().unwrap();
                let cy: f64 = parts[3].parse().unwrap();
                let coords: Vec<&str> = parts[4].split('|').collect();
                let dx: f64 = coords[2].parse().unwrap();
                let dy: f64 = coords[3].parse().unwrap();
                polygons.push(vec![
                    [cx, cy],
                    [cx, cy + dy],
                    [cx + dx, cy + dy],
                    [cx + dx, cy],
                ]);
            }
            _ => {}
        }
    }
    polygons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let long = vec![b'a'; 1_000_000];
        assert_eq!(
            sha256_hex(&long),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn canonical_polygon_is_rotation_and_direction_invariant() {
        let a = [[0.0, 0.0], [0.001, 0.0], [0.001, 0.001], [0.0, 0.001]];
        let mut b = a;
        b.reverse();
        b.rotate_left(2);
        assert_eq!(canonical_polygon(&a), canonical_polygon(&b));
    }
}
