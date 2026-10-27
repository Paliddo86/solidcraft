#!/usr/bin/env sh
# KiCad oracle: run KiCad's public command-line exporter on a board, so reference geometry comes
# from a separate program. KiCad is never linked, vendored or shipped with SolidCraft; see README.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

usage() {
  echo "usage: $0 <board.kicad_pcb> [--out DIR]" >&2
  exit 64
}

board=
out=
while [ $# -gt 0 ]; do
  case "$1" in
    --out) [ $# -ge 2 ] || usage; out=$2; shift 2 ;;
    -h | --help) usage ;;
    -*) echo "$0: unknown option $1" >&2; usage ;;
    *)
      [ -z "$board" ] || usage
      board=$1
      shift
      ;;
  esac
done
[ -n "$board" ] || usage
[ -f "$board" ] || { echo "$0: $board: not a file" >&2; exit 66; }

kicad=${KICAD_CLI:-}
if [ -z "$kicad" ]; then
  if command -v kicad-cli >/dev/null 2>&1; then
    kicad=kicad-cli
  elif [ -x "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli" ]; then
    kicad="/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli"
  fi
fi
if [ -z "$kicad" ]; then
  echo "$0: kicad-cli not found." >&2
  echo "Install KiCad from https://www.kicad.org/download/ or set KICAD_CLI." >&2
  echo "The oracle runs KiCad as a separate program; it is never part of SolidCraft's build." >&2
  exit 69
fi

stem=$(basename "$board")
stem=${stem%.*}
[ -n "$out" ] || out="$here/out/$stem"
mkdir -p "$out"

echo "kicad-cli: $("$kicad" version | head -1)"
echo "board: $board"
echo "out: $out"

# 3D reference: the substrate, holes and component models as STEP.
"$kicad" pcb export step --output "$out/board.step" "$board"
echo "wrote $out/board.step"

# 2D reference: the board's layers as SVG.
if "$kicad" pcb export svg --output "$out/board.svg" "$board" 2>"$out/svg.log"; then
  echo "wrote $out/board.svg"
else
  echo "note: kicad-cli pcb export svg did not run ($(cat "$out/svg.log")); the STEP reference is there" >&2
fi
