#!/usr/bin/env bash
# RM-109: prove that the **native** CSXCAD reads the CSXCAD XML that the Rust
# hyp2mat writes, and that both cores observe the same model.
#
# For every `.hyp` fixture the Rust CLI writes CSXCAD XML, then
#   * the native tool (`native_model_dump.cpp`, linked against the pinned native
#     libCSXCAD built by CSXCAD/rust/tests/native_gen/run_native_generator.sh)
#   * the Rust dump (`cargo run --example model_dump`)
# print the same canonical model summary; the script diffs them.
#
# Usage: hyp2mat/rust/tests/native/run_native_read_check.sh
#   RUSTMIG_NATIVE_DIR  native build root (default: $HOME/rustmig-native)
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK="${RUSTMIG_NATIVE_DIR:-$HOME/rustmig-native}"
FPARSER_PREFIX="${FPARSER_PREFIX:-$HOME/opt/openEMS}"

if [ ! -f "$WORK/csxcad-build/src/libCSXCAD.so" ]; then
    echo "native CSXCAD not found at $WORK/csxcad-build/src" >&2
    echo "run CSXCAD/rust/tests/native_gen/run_native_generator.sh first" >&2
    exit 2
fi

export PATH="$HOME/.cargo/bin:$PATH"
OUT="${TMPDIR:-/tmp}/hyp2mat-native-read"
rm -rf "$OUT"
mkdir -p "$OUT"

echo "== building the Rust CLI and the model dump"
( cd "$CRATE_ROOT" && cargo build --quiet --bin hyp2mat ) || exit 1
BIN="$CRATE_ROOT/../../target/debug/hyp2mat"
if [ ! -x "$BIN" ]; then
    BIN="$(find "$CRATE_ROOT/../../target" -maxdepth 2 -name hyp2mat -type f -perm -u+x | head -1)"
fi
if [ ! -x "$BIN" ]; then
    echo "hyp2mat binary not found" >&2
    exit 1
fi

echo "== writing CSXCAD XML for every fixture"
for fixture in "$CRATE_ROOT"/tests/fixtures/*.hyp; do
    name="$(basename "$fixture" .hyp)"
    "$BIN" -f csxcad --csxcad-xml -o "$OUT/$name.xml" "$fixture" >/dev/null 2>&1 || {
        echo "conversion failed for $name" >&2
        exit 1
    }
done

echo "== building the native model dump"
g++ -std=c++17 -O2 -o "$OUT/native_model_dump" "$SCRIPT_DIR/native_model_dump.cpp" \
    -I"$WORK/csxcad-install/include/CSXCAD" \
    -I"$FPARSER_PREFIX/include" \
    -L"$WORK/csxcad-build/src" \
    -Wl,-rpath,"$WORK/csxcad-build/src" \
    -lCSXCAD || exit 1

XMLS=("$OUT"/*.xml)

echo "== native read"
"$OUT/native_model_dump" "${XMLS[@]}" > "$OUT/native.dump" || {
    echo "native CSXCAD could not read the Rust XML" >&2
    exit 1
}

echo "== rust read"
( cd "$CRATE_ROOT" && cargo run --quiet --example model_dump -- "${XMLS[@]}" ) > "$OUT/rust.dump" || exit 1

echo "== diff"
if diff -u "$OUT/native.dump" "$OUT/rust.dump" > "$OUT/diff.txt"; then
    echo "native and Rust model dumps are identical ($(wc -l < "$OUT/native.dump") lines, ${#XMLS[@]} documents)"
    echo "logs: $OUT"
    exit 0
fi
sed -n '1,80p' "$OUT/diff.txt"
echo "model dumps differ; logs: $OUT" >&2
exit 1
