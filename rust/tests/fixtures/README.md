# HyperLynx `.hyp` fixture corpus

This directory holds the input corpus used by the hyp2mat-rs tests. It combines
the three real HyperLynx files shipped with the native project with synthesized
files that exercise the rest of the documented grammar.

## Provenance

| File | Origin |
| --- | --- |
| `real_notchfilter.hyp` | copy of `hyp2mat/eagle/notchfilter/notchfilter.HYP` (EAGLE 6.3 export, unchanged) |
| `real_msl.hyp` | copy of `hyp2mat/eagle/tutorial/msl.HYP` (EAGLE 6.3 export, unchanged) |
| `real_hairpinfilter.hyp` | copy of `hyp2mat/eagle/hairpinfilter/hairpinfilter.HYP` (EAGLE 6.3 export, unchanged) |
| `synthetic_*.hyp` | written for this port from the Bison grammar in `hyp2mat/lib/parse.yy` and the scanner in `hyp2mat/lib/scan.ll` |

The native repository contains **no** lowercase `.hyp` files; the audit note in
`docs/migration/audits/hyp2mat.md` therefore reported "no fixture". These three
`.HYP` files are the real inputs the original project ships for its tutorial and
EAGLE export workflows, so they are used verbatim here.

The synthesized files only use record types, keywords, and value shapes that
exist in the grammar. They were validated by running the native `hyp2mat`
0.0.18 binary (built from the pinned sources with autotools in WSL) over every
file: all nine parse without a single syntax error, and the native model dump
(`hyp2mat -d`) is byte-identical to the Rust model dump (see
`tests/reference/dump/`). No syntax was invented.

**Unit note:** the HyperLynx dialect defines `{UNITS=METRIC ...}` as
**centimetres** (`hyp2mat/lib/exec_board.cc`: `unit = 0.01`). The synthetic
files therefore use centimetre numbers (e.g. `X1=3.000` is 3 cm). The real
EAGLE files use inches.

## Coverage

| File | Layers | Board outline | Planes / pours | Traces | Vias | Pads | Arcs | Devices / pins |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `real_notchfilter.hyp` | signal + dielectric + signal | 4 segments | `T=POUR` on both copper layers | – | – | rectangular, 180° rotation | – | 4 devices, 4 pins |
| `real_msl.hyp` | 2 dielectrics + 2 signals | 4 segments | `T=POUR` + `POLYVOID` hole | – | – | rectangular, 180° rotation | – | 5 devices, 6 pins |
| `real_hairpinfilter.hyp` | signal + dielectric + signal | 4 segments | 6 `T=POUR` polygons | – | – | rectangular | – | 7 devices, 7 pins |
| `synthetic_plane_clearance.hyp` | 4 metal layers, 3 dielectrics | 4 segments + rectangular cutout | `T=PLANE` with clearance both planes | `SEG` + `ARC` | 3 padstack vias | oval / rectangular / oblong, 45° rotation | board cutout | `SUPPLIES`, `NET_CLASS`, attributes |
| `synthetic_polygon_void.hyp` | 2 signals + dielectric | 4 segments + cutout | `T=POUR` + `POLYVOID`, `T=COPPER` | `POLYLINE` | – | rectangular | – | 1 device/pin |
| `synthetic_metric_weight.hyp` | 2 signals + dielectric | 8 segments + 4 rounded `PERIMETER_ARC` corners | – | `SEG` + `ARC` | v1.0 `VIA` with `S1/S2` pads | deprecated `PAD` | yes | `USEG` |
| `synthetic_english_rotated.hyp` | 2 signals + dielectric | 4 segments | – | `SEG` with `W=` | – | 90°, 180° rotations, `MDEF`/`ADEF` | – | 3 pins |
| `synthetic_flood.hyp` | 2 signals + plane + 2 dielectrics | 4 segments | `T=POUR` on a plane layer | `SEG` | – | – | – | – |
| `synthetic_minimal.hyp` | 2 signals + dielectric | none (bounding box is derived) | `T=COPPER` | – | – | – | – | – |

## Integrity

`manifest.sha256` lists the SHA-256 of every `.hyp` file. The
`fixtures::manifest_matches_files` test verifies the manifest, and
`fixtures::dump_matches_native_reference` verifies the parser model against the
native dump.

Reference outputs produced by the native binary for this corpus live in
`../reference/`:

* `*.m` – native CSXCAD MATLAB script (`-f csxcad`);
* `*.qucs` – native QucsStudio layout;
* `*.summary.txt` – native `--verbose` stderr;
* `dump/*.dump.txt` – native `-d` model dump (`hyp_debug.txt`);
* `raw/*.raw.m` – native `--raw` output;
* `real_notchfilter.pdf` – native PDF;
* variant outputs (`real_notchfilter_pcboutline_metal3d.m`,
  `real_notchfilter_crop.m`, `synthetic_flood_flooded.m`).

The parser model dump and the `--raw` polygon output are byte-identical to the
native binary for every fixture. The joined CSXCAD/QucsStudio geometry is
identical for seven fixtures; `synthetic_metric_weight` and
`synthetic_plane_clearance` differ by at most one 10 µm grid step on a single
intersection vertex of a copper arc (Clipper2 and Clipper1 round shallow
intersections differently). `tests/fixtures.rs` asserts exactly that bound.
