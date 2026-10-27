# KiCad oracle

Reference boards and exports for the KiCad PCB import (`crates/io/src/kicad_in.rs`, issue #5091).
KiCad is run as a separate program through its public command line (`kicad-cli`), exactly as
[`tools/freecad-oracle/`](../freecad-oracle/) runs FreeCAD: SolidCraft does not use, link, vendor
or ship any code, library or binary from KiCad, and nothing in this directory is part of the
build. KiCad is GPL-3.0-or-later: its sources are never read, and only its documented file
formats and its public command line are used.

## What it does

`run.sh <board.kicad_pcb> [--out DIR]`:

- prints `kicad-cli version`;
- exports the board's 3D model as STEP (`kicad-cli pcb export step`), a reference a SolidCraft
  import can be compared with (bounding box, volume, component placement);
- exports the board's layers as SVG (`kicad-cli pcb export svg`), a picture of the outline,
  copper, mask and silkscreen the import should reproduce;
- writes both under `out/<board>/` (git-ignored).

## Running it

```sh
tools/kicad-oracle/run.sh tools/kicad-oracle/boards/sample.kicad_pcb
tools/kicad-oracle/run.sh board.kicad_pcb --out /tmp/kicad-oracle
```

`kicad-cli` is found through `$KICAD_CLI`, then `kicad-cli` on `PATH`, then the macOS app bundle
(`/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli`). Install KiCad from
<https://www.kicad.org/download/> (or your distribution's package) to run it. Nothing here
downloads or bundles KiCad.

## Clean-room rules

- Never read KiCad's source code (GPL-3.0-or-later): the import is written from the public
  s-expression format documentation and from boards KiCad itself writes.
- The oracle only runs KiCad as a separate program and reads its output; no KiCad code, icons or
  artwork is copied into SolidCraft.
- Reference values are for our own fixtures only. Do not commit another project's board files
  without their licence and an `ATTRIBUTION.md` row.

## Status

The board oracle works today (`kicad-cli pcb export step|svg`). A schematic oracle
(`kicad-cli sch export netlist|svg`) will be added when the schematic importer lands.
