# Electrical and electronic design: a clean-room research note

## About this research

This note is the groundwork requested by issue #1624: what a schematic-capture and
printed-circuit-board (PCB) workspace has to do, before any of it is built in SolidCraft. It is
*research*, not a design document and not code.

It was written in October 2026 by SolidCraft contributors from public documentation, published
standards, the public documentation of open-source tools, and their own experience as users.
No commercial CAD product was driven, scripted, automated or reverse-engineered, and no
commercial product's files, measurements or command lists were used. Items that could not be
confirmed from a public source are marked **(unverified)**.

The rules this note follows are the project's: never read, copy or link GPL/LGPL/AGPL source
(KiCad, LibrePCB, Horizon EDA, QElectroTech, WireViz and Fritzing are all copyleft); describe
published documentation in our own words; and keep every number in millimetres (radians for
angles), Z up.

## Summary

- **Schematic capture is a netlist editor with a picture.** Symbols are placed on a sheet,
  their pins are joined by wires and labels, and the result is a list of nets: sets of pins
  that are electrically one node. Everything else (drawing, printing, BOM) is derived.
- **The board is a 2.5D object**: a substrate with an outline and cutouts, a stack of copper
  and dielectric layers, drilled holes, and footprints placed by reference. The netlist is the
  contract between schematic and board; layout turns each net into copper.
- **The hard parts are connectivity, rules and exchange.** Deciding what counts as one net,
  checking clearances and widths (DRC), and moving data between ECAD, MCAD and fabrication
  without losing meaning are where tools differ most.
- **Manufacturing output** is still Gerber plus Excellon drill in most shops; IPC-2581 and
  ODB++ carry more (stackup, nets, test points); **MCAD exchange** uses IDF/IDX and STEP AP242.
- A first SolidCraft version should do the schematic side first (symbols, wires, labels,
  netlist), then a board model, then import/export; a full autorouter and simulation are out of
  scope for the first milestone.

## Fundamentals

### Schematic capture

A schematic is a drawing whose meaning is electrical, not geometric. Its objects are:

- **Symbols** (also called parts or components): a named shape with **pins**. Each pin has a
  number (the package's pin number, e.g. `1`…`8`) and often a name (e.g. `VCC`, `RESET`). A
  symbol also has a **reference designator** (`R1`, `U3`) and a **value** (`10k`, `NE555`).
- **Wires**: polylines that join pin ends. A **junction** marks a connection where wires cross
  or meet; without one, a crossing is not a connection.
- **Net labels**: text that names the net at a point. Two separate wires with the same label are
  one net, which is how a schematic avoids drawing every connection.
- **Power symbols**: ground and supply symbols that are labels with a fixed name and a shape.
- **Sheets and hierarchy**: a large design is split into sheets; a **sheet symbol** (or
  hierarchical pin) connects a net inside a sheet to the sheet above. Buses group related nets.
- **Buses and bus entries**: a bus is a bundle of named nets; a bus entry connects one member
  to a wire. Some tools only draw buses and require the members to be labelled explicitly
  **(unverified)**.
- **Text, title blocks and drawing frames**: documentation, not connectivity.

The essential operation is **connectivity**: two pin ends, or a pin end and a wire end, that
share a point (within a small tolerance) are connected. Labels merge nets by name, not by
position. A good capture tool shows the result as a **ratsnest** (straight “airwires” between
pins that still need copper on the board) and as a **netlist**.

### Netlist and connectivity

A **net** is a set of pins that are one electrical node. A netlist lists, for each net, its
name and its pins (`R1.1`, `U3.7`, …). It also lists the components and their values, which is
what a **bill of materials (BOM)** is built from.

- **Net names** are either given by a label or power symbol, or generated (`N$12`,
  `Net-(R1-Pad2)`). Tools differ in how they name unnamed nets and in whether a label on a wire
  renames the whole net or only the wire **(unverified for every tool)**.
- **Power and ground** are ordinary nets with conventional names (`GND`, `+3V3`, `VCC`). Global
  labels and power symbols make a net visible across sheets.
- **No-connect flags** mark a pin that is deliberately unconnected, so a check does not report
  it. Unconnected pins are otherwise a common warning.
- **Net classes** (power, high-speed, differential) carry rules such as width and clearance;
  **differential pairs** are two nets routed together with a controlled gap.

The netlist is the **contract** between schematic and board: a board tool imports it, checks
that every pin is placed, and reports pins that are missing or extra. Renaming or re-annotating
components must keep the two sides in step (**forward annotation** schematic → board,
**back annotation** board → schematic).

### The board

A PCB is a stack of layers:

- **Substrate**: the insulating base (FR-4 and others), with a **thickness** (commonly 1.6 mm;
  many others exist) and a **stackup** of copper and dielectric layers. The **outline** is a
  closed contour on a mechanical layer; **cutouts** are closed contours inside it. **Drilled
  holes** are round or slotted, plated or not.
- **Copper layers**: top and bottom, plus inner layers in a multilayer board. Each has a name
  and a position in the stackup; copper thickness is often given in ounces per square foot or
  micrometres **(unverified as a single convention)**.
- **Tracks (traces)**: copper lines of a given width on one layer, ending at pads, vias or other
  tracks of the same net. A **via** is a plated hole joining copper on two or more layers;
  **blind/buried vias** reach only some layers.
- **Pads**: the copper where a component pin is soldered. **Through-hole (THT)** pads are
  drilled; **surface-mount (SMD)** pads are not. A pad has a shape (round, rectangular, oval,
  custom) and, for THT, a **drill** and an **annular ring** (copper around the hole).
- **Footprints** (also called land patterns or decals): the pad layout of one package, with a
  **courtyard** (keep-out around the part), silkscreen and often a 3D model. A footprint is
  placed at a position and rotation and carries a reference and value.
- **Zones / copper pours**: filled copper areas, often connected to a net (e.g. ground), with a
  clearance to other nets and a fill strategy.
- **Silkscreen**: printed text and outlines, on top and bottom. **Solder mask** openings expose
  pads; **paste mask** openings control solder paste.
- **Board-edge clearance** and **keep-outs** keep copper and parts away from the edge or from
  defined areas.

The board is therefore **2.5D**, not a solid model: it has a thickness and a stackup, but its
geometry is mostly planar per layer. A mechanical CAD system usually imports a simplified solid
(outline + thickness + component envelopes) rather than every copper detail.

### Design rules (DRC)

A **design rule check (DRC)** compares the board with a set of rules and reports violations.
The common rules are:

- **Clearance**: the smallest gap between copper of different nets (and between copper and the
  board edge or a hole). It often depends on the net class, the voltage and the environment.
- **Track width**: a minimum per net class, chosen from the current the track carries and the
  allowed temperature rise.
- **Via and drill**: smallest drill, annular ring, hole-to-hole spacing, and via-to-copper
  clearance.
- **Pads**: a pad's size against its drill (annular ring) and its solder-mask opening.
- **Board edge**: copper, holes and courtyards at the outline.
- **Silkscreen**: text and lines over pads (a warning, not an electrical error, in most tools
  **(unverified)**).
- **Unconnected pins** and nets with one pin: a warning that catches a forgotten wire.

Public standards that inform the numbers: **IPC-2221** (generic PCB design), **IPC-2152**
(current-carrying capacity of tracks), **IPC-7351** (footprint/land-pattern generation) and
**IPC-A-610** (acceptability of assemblies). They are paid documents; our tests must use
analytic values and public summaries, not copied tables **(unverified for every table)**.

Rules are where a tool earns trust: the same board can pass one tool's DRC and fail another's
on a different default or a different interpretation of a sliver. A useful DRC **says which
rule, which objects, how much and where**, and offers a way to waive one violation with a
reason.

### Manufacturing outputs

The fabrication house needs:

- **Gerber** (RS-274X, and the newer X2 with attributes): one file per copper/mask/silkscreen
  layer, plus the outline. It describes shapes (apertures) and draws (flashes, lines, arcs,
  regions).
- **Excellon drill** files: hole positions, diameters and plating, plus a tool table. Slots are
  expressed as routed paths (format details vary).
- **Stackup and materials**: often a drawing or a separate document; IPC-2581 and ODB++ can
  carry it in the data.
- **Pick-and-place / centroid**: component references, positions, rotations and sides for the
  assembly machine.
- **BOM**: references, values, footprints and quantities, with supplier parts when known.
- **Panelisation**: several boards in one panel with rails, fiducials and tooling holes. It is a
  fabrication decision, not part of the logical board.

**IPC-2581** (XML) and **ODB++** (a directory format) are richer than Gerber+Excellon: nets,
stackup, test points and assembly data travel together. Adoption varies by shop **(unverified
for the current split)**. Gerber+Excellon remains the lowest common denominator.

### ECAD-MCAD exchange

A board lives inside a mechanical product, so the two worlds exchange:

- **IDF** (and its successor **IDX**): a text exchange of board outline, holes, keep-outs and
  component envelopes (place, height, shape), with a way to propose and accept changes.
- **STEP AP242**: a 3D exchange that can carry the board solid, component models and product
  structure; it is what a mechanical CAD system usually imports for a visual check.
- **Placement and height data**: a simple table (reference, x, y, rotation, height) is often
  enough for an enclosure. It is also what a **3D board in MCAD** needs to place components.

The recurring failures are lost holes, mirrored footprints, wrong rotations and components
that float above the board because the model's origin is not the footprint's origin. A good
exchange **round-trips**: export, import into the other tool, export again, and compare.

### Accuracy

Electrical design is less about continuous geometry than about **discrete decisions** (is this
one net? does this clearance pass?) and about **data that must not be lost**. Accuracy therefore
means:

- **Connectivity is exact**: a point that two objects share within the tool's tolerance is a
  connection; the tolerance must be documented and small (micrometres to tens of micrometres,
  not millimetres).
- **Rules are explicit**: every violation names the rule, the objects, the measured value and
  the limit, so a user can decide.
- **Exchange is lossless or refuses**: an importer that cannot represent something says so
  rather than dropping it silently (our import already reports warnings per skipped item).
- **Verification** uses independent data: KiCad (run as a separate program) exports a board to
  STEP/SVG as a reference; analytic values cover simple shapes; IPC standards give the limits.

A result is only as trustworthy as the **report** around it: what was read, what was skipped,
what was assumed, and what could not be checked.

## Programs

This section describes what tools offer from their **public documentation and user reports**,
not from their code or from driving them for this project.

- **Fusion Electronics** (the successor to EAGLE, inside Autodesk Fusion): schematic capture,
  PCB layout, libraries, and a 3D board that lives in the same document as the mechanical
  design. Users report one environment for electronics and enclosure and an integrated 3D
  check; they also report friction around the EAGLE-to-Fusion transition, library management and
  performance on large boards **(unverified)**.
- **Altium Designer**: a long-standing professional PCB tool with strong interactive routing,
  rules and MCAD collaboration. Users report power and cost/complexity in the same breath
  **(unverified)**.
- **Cadence OrCAD X / Allegro X** and **Siemens Xpedition / PADS**: professional layout and
  high-speed flows; strong in large organizations, heavy to learn **(unverified)**.
- **KiCad**: free and open source (GPL-3.0-or-later), with schematic capture, PCB layout,
  libraries, 3D viewer and a command-line exporter. Users report no cost, active development and
  good enough for many professional boards; they also report library/project workflow friction
  and some rough edges in large designs **(unverified)**. Its **file formats are public**, which
  is why our first import targets `.kicad_pcb`.
- **LibrePCB**, **Horizon EDA**, **QElectroTech**, **WireViz**, **Fritzing**: other open-source
  options with different scopes (from schematic-only to full boards). All copyleft; usable as
  *programs* and as format references, never as source to read **(unverified for each)**.
- **EAGLE** (legacy): still used; its XML format is public and widely documented.

What they share: the same conceptual model (symbols → netlist → board), the same output
(Gerber/Excellon), and the same pain points (libraries, exchange, rules).

## Complaints and requests

Recurring themes in public user discussions **(unverified in detail)**:

- **Integration with mechanical CAD**: keep the board and the enclosure in step, and see
  component heights in 3D.
- **Library management**: finding, versioning and trusting symbols and footprints; avoiding
  "the footprint is wrong" failures.
- **Import/export fidelity**: losing nets, stackup or 3D models when moving between tools.
- **Rules that explain themselves**: which rule failed, by how much, and how to waive it.
- **Performance and stability** on large boards and long sessions.
- **Learning curve and cost** for professional tools; **rough edges** for free ones.

## What users need

A typical small-board workflow:

1. **Capture**: place symbols, wire pins, label nets, set references and values; get a netlist.
2. **Assign footprints**: map each symbol to a package; check pin numbers match.
3. **Place**: put footprints on the board, set the outline, keep-outs and mounting holes.
4. **Route**: draw tracks and vias per net, pour copper, check the ratsnest shrinks to zero.
5. **Check**: run DRC, fix violations, check the board against the enclosure.
6. **Produce**: Gerber, drill, pick-and-place, BOM, assembly drawings; send to fabrication.
7. **Iterate**: change the schematic, forward-annotate the board, re-route what changed.

Inputs: symbols/footprints (libraries), a board outline (often from the mechanical design),
rules (net classes, clearances), and the netlist. Outputs: the board, the fabrication package,
the BOM, and a 3D model for MCAD.

## Glossary

- **Annular ring** — the copper width around a drilled hole; too small and the drill breaks out.
- **BOM (bill of materials)** — the list of parts, references, values and quantities.
- **Clearance** — the minimum gap between copper of different nets.
- **Courtyard** — the keep-out area around a footprint, larger than the part.
- **DRC (design rule check)** — a check of the board against clearance, width and hole rules.
- **Excellon** — a drill-file format (positions, diameters, plating).
- **Footprint (land pattern, decal)** — the pads and outline of one package on the board.
- **Gerber (RS-274X/X2)** — a layer description format for fabrication.
- **IDF/IDX** — ECAD–MCAD exchange of outline, holes, keep-outs and component envelopes.
- **IPC-2221 / IPC-2152 / IPC-7351** — public standards for board design, track current and
  footprints (paid documents; summaries used here).
- **Net** — a set of pins that are one electrical node.
- **Netlist** — the list of nets and their pins; the contract between schematic and board.
- **Pad** — the copper where a pin is soldered (THT: drilled; SMD: not).
- **PCB** — printed circuit board.
- **Pick-and-place (centroid)** — the file that tells an assembly machine where each part goes.
- **Ratsnest (airwires)** — straight lines between pins of a net that still need copper.
- **Stackup** — the order and thickness of copper and dielectric layers.
- **Symbol** — the schematic shape of a part, with pins, reference and value.
- **Track (trace)** — a copper line on one layer.
- **Via** — a plated hole joining copper on two or more layers.
- **Zone (copper pour)** — a filled copper area, often connected to a net.

## Open questions

- Which connectivity tolerance should the schematic use, and should it be configurable?
- Should a schematic be a separate document (as here) or a mode of the same document?
- Which exchange format first: KiCad (import), IDF/IDX, STEP AP242, IPC-2581?
- How much of the board stackup and rules belongs in the first milestone?
- Which IPC limits can we encode from public summaries without copying paid tables?

## Sources

- Issue #1624 (research request) and #5091/#5092 (KiCad PCB import) in this repository.
- Public KiCad file-format documentation (s-expression files) and the `kicad-cli` manual page.
- Public standards: IPC-2221, IPC-2152, IPC-7351, IPC-2581, IPC-A-610, Gerber RS-274X/X2,
  Excellon, IDF/IDX, STEP AP242 (titles and scope; paid documents not copied).
- Public product documentation and user discussions of the tools named under **Programs**
  (used for scope and reported experience only).
- Tools in this repository: `tools/kicad-oracle/` (KiCad as a separate program),
  `crates/io/src/kicad_in.rs` (the import) and `crates/ecad/` (the schematic model).

