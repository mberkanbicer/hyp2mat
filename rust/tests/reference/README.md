# Native hyp2mat reference outputs

Every file in this directory was produced by the **native** hyp2mat 0.0.18
(`hyp2mat/configure.ac`: `AC_INIT([hyp2mat], [0.0.18])`) built from the pinned
sources in this repository. No file here may be edited by hand; the Rust tests
in `tests/fixtures.rs`, `tests/cli.rs` and `tests/fixtures.rs` compare against
them.

## Provenance

| Item | Value |
| --- | --- |
| Native source | `hyp2mat/` at the pinned submodule revision (unmodified) |
| Native version | 0.0.18 (`AC_INIT([hyp2mat], [0.0.18])`) |
| Build | autotools, out of tree: `./bootstrap.sh && mkdir build && cd build && ../configure && make -j` |
| Host | Ubuntu 24.04 under WSL2, GCC 15.2 |
| Fixtures | `tests/fixtures/*.hyp` (`real_*` from the native EAGLE examples, `synthetic_*` derived from the parser grammar) |

The native tree is copied to a scratch directory before building so the
repository's C++ sources stay untouched, the same rule the CSXCAD fixture
generator follows (`../../CSXCAD/rust/tests/native_gen/README.md`).

## Files

| Pattern | Produced by |
| --- | --- |
| `<fixture>.m` | `hyp2mat -f csxcad -o <fixture>.m <fixture>.hyp` (default output format is `pdf`) |
| `<fixture>.qucs` | `hyp2mat -f qucs -o <fixture>.qucs <fixture>.hyp` |
| `<fixture>.qucs.stderr.txt` | the stderr of the QucsStudio run (device/port warnings, native ordering) |
| `<fixture>.summary.txt` | `hyp2mat -v -f csxcad ...` board/layer summary; native `CSBoard::PrintSummary` writes to `std::cerr` (`lib/pcb.cc:319`), not stdout |
| `real_notchfilter.pdf` | `hyp2mat -f pdf -o real_notchfilter.pdf real_notchfilter.hyp` |
| `real_notchfilter_crop.m`, `real_notchfilter_pcboutline_metal3d.m`, `synthetic_flood_flooded.m` | variant runs with `-x/-X/-y/-Y`, `--pcb-outline --metal-3d` and `--flood plane_layers` |

The `--raw` polygon dumps and the `-d` model dumps are not committed verbatim:
the fixture tests invoke the Rust binary with `-r`/`-d` and compare against the
native text captured in `tests/fixtures.rs` expectations and the `.m` polygon
blocks.

## Regenerating

The native binary is not installed on the development host. To regenerate:

```bash
# scratch copy of the pinned sources (keeps the repository untouched)
mkdir -p ~/hyp2mat-native && (cd hyp2mat && tar --exclude=.git -cf - .) | (cd ~/hyp2mat-native && tar -xf -)
cd ~/hyp2mat-native && ./bootstrap.sh && mkdir build && cd build && ../configure && make -j"$(nproc)"

# then, from the repository root
for f in hyp2mat/rust/tests/fixtures/*.hyp; do
  name=$(basename "$f" .hyp)
  ~/hyp2mat-native/build/hyp2mat -f csxcad -o "hyp2mat/rust/tests/reference/$name.m" "$f"
  ~/hyp2mat-native/build/hyp2mat -f qucs -o "hyp2mat/rust/tests/reference/$name.qucs" "$f" \
      2> "hyp2mat/rust/tests/reference/$name.qucs.stderr.txt"
  ~/hyp2mat-native/build/hyp2mat -v -f csxcad -o /dev/null "$f" \
      2> "hyp2mat/rust/tests/reference/$name.summary.txt"
done
```

`-o /dev/null` for the summary run discards the script; the committed
`.summary.txt` files contain the native `-v` stderr text.

## Accepted differences

The Rust port reproduces the native model dump, `--raw` polygons and `-v`
stderr **byte for byte**. Two fixtures with copper arcs
(`synthetic_metric_weight`, `synthetic_plane_clearance`) differ by at most one
10 µm grid step on a single polygon-intersection vertex because the Rust port
uses Clipper2 while the native code uses Clipper 1; the bound is asserted
explicitly in `tests/fixtures.rs` (`geometry_tolerance`). All other fixtures are
geometrically exact.
