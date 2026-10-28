# SolidCraft — Piano UI Electronics (ribbon + component manager)

> Stato: **M2 (libreria + Component Manager) completato** · Prossimo passo: import KiCad/STEP nella libreria (M2b), editor e board 3D · Ultimo aggiornamento: 2026-10-28
> Branch `ecad`, allineato a `upstream/main` (v0.25.1, merge `7dc9b850`) e con il PR KiCad #5092 già incluso (`3b07c355`).

Scopo: portare il workspace Electronics da "canvas con toolbar minima" a un
ambiente di progettazione paragonabile a Fusion 360 Electronics: **ribbon
contestuale** (Schematic / PCB / Library), **component manager** con simboli,
footprint e modelli 3D, editing guidato dalla libreria. La toolbar del pannello
centrale resta come modalità compatta finché il ribbon non copre tutti i tool.

## Riferimenti (comportamento, non codice)

Fusion 360 Electronics (Autodesk Fusion):
- "Electronics workspace tool names and descriptions" — elenco ufficiale dei
  tool del workspace, con descrizioni (Place/Route/Inspect/Library).
- "Schematic Editor commands" e "Manual routing in electronics design".
- "Design rules" — `https://help.autodesk.com/cloudhelp/ENU/Fusion-ECAD/files/ECD-DESIGN-RULES.htm`
  (clearance, copper width, via drill, DRC, priorità delle regole).
- "Routing multilayer boards" — `.../ECD-ROUTE-MULTILAYER-BOARD-CPT.htm`
  (ROUTE, VIA, CHANGE LAYER, RATSNEST, POLYGON, Layer Stack Manager).
- "Tutorial: Manage electronic component libraries" — `.../ECD-TUT-ECAD-MANAGE-LIBRARIES-TOP-LEVEL.htm`
  (Library Manager, colonne In Design / In Use / Available, toggle In Use).
- "Electronic library - Component attributes" e "Electronics Library Manager".
- "Electronic Component 3D Mapping in Fusion Libraries Editor" e
  "Mapping 3D Modules With the Fusion Package Editor" (blog Autodesk):
  il modello 3D si aggancia al **package/footprint**, non al simbolo.
- SnapEDA / Ultra Librarian: formato di scambio symbol + footprint + 3D model.

Riferimento secondario: KiCad (Library Manager, Symbol Editor, Footprint
Editor, 3D models) — solo comportamento e formati aperti, nessun codice o
asset copiato. L'import KiCad già portato (#5092) resta la sorgente principale
per popolare la libreria.

## Principi SolidCraft

- **Ogni tool è un comando** (`CommandSpec` con input/output, `.at(tab, panel)`,
  icona, label): il ribbon, la palette comandi, il canale di controllo e gli
  agenti usano lo stesso percorso. Nessun dialogo da chiamate programmatiche.
- **Un solo container UI** e una sola camera per canvas; il ribbon è il posto
  dei comandi, il pannello centrale resta il canvas.
- **Localizzazione**: ogni nuova label va in `assets/locales/pending/` (prima
  `en` + `it`), mai direttamente nei cataloghi condivisi.
- **Clean-room**: si studia il comportamento, non si copiano icone, testi o
  template; icone originali in `assets/icons/` con riga in `ATTRIBUTION.md`.
- **Test**: ogni tool nuovo ha un test nel file di test dell'area; il ribbon
  deve passare `every_toolbar_tab_and_panel_is_in_the_english_catalog`.

## 1. Ribbon contestuale

Quando il workspace Electronics è attivo, il ribbon superiore mostra i tab
**SCHEMATIC** e **PCB** al posto dei tab Design. Il tab dipende dal documento
attivo (Schematico o PCB); cambiando tab si cambia anche il canvas.

### SCHEMATIC

| Pannello | Comandi |
|---|---|
| DESIGN | Select, Move, Rotate, Mirror, Delete, Properties |
| PLACE | Component, Net, GND, Supply, Port, Frame, Text |
| CONNECT | Wire, Bus, Junction, No Connect, Net Label |
| INSPECT | ERC, Netlist, Nets, Measure |
| LIBRARY | Library Manager, Place from Library, Symbol Editor |

### PCB

| Pannello | Comandi |
|---|---|
| DESIGN | Select, Move, Rotate, Mirror, Delete, Properties |
| PLACE | Component, Pad, Via, Hole, Polygon, Text, Dimension |
| ROUTE | Route Manual, Route Multi, Differential Pair, Airwire, Via, Layer, Ratsnest |
| BOARD | Board Outline, Layer Stack, Design Rules, Copper Pour |
| INSPECT | DRC, Errors, Nets, Measure |
| LIBRARY | Library Manager, Place from Library, Footprint Editor, 3D Model |

### Mapping ai comandi SolidCraft

- `schematic.select`, `schematic.move`, `schematic.rotate`, `schematic.mirror`,
  `schematic.delete`, `schematic.properties`
- `schematic.component.place` (device della libreria), `schematic.net`,
  `schematic.gnd`, `schematic.supply`, `schematic.port`, `schematic.frame`,
  `schematic.text`, `schematic.wire`, `schematic.bus`, `schematic.junction`,
  `schematic.no_connect`, `schematic.label`
- `schematic.erc`, `schematic.netlist`, `schematic.nets`, `schematic.measure`
- `pcb.select`, `pcb.move`, `pcb.rotate`, `pcb.mirror`, `pcb.delete`, `pcb.properties`
- `pcb.component.place`, `pcb.pad`, `pcb.via`, `pcb.hole`, `pcb.polygon`,
  `pcb.text`, `pcb.dimension`
- `pcb.route.manual`, `pcb.route.multi`, `pcb.route.diff_pair`,
  `pcb.route.airwire`, `pcb.layer`, `pcb.ratsnest`
- `pcb.outline`, `pcb.stack`, `pcb.rules`, `pcb.pour`
- `pcb.drc`, `pcb.errors`, `pcb.nets`, `pcb.measure`
- Libreria: `library.list`, `library.manager`, `library.place`,
  `library.symbol_editor`, `library.footprint_editor`, `library.model3d`

I comandi che **non** hanno ancora parametri sufficienti (es. Place Component)
aprono un **tool di piazzamento** con il device selezionato nel component
manager; nessun comando lancia un dialogo quando è chiamato da un agente.

## 2. Component manager (libreria)

### Modello dati (`crates/ecad/src/library.rs`)

```rust
pub struct Library {
    pub id: String, pub name: String, pub version: String,
    pub symbols: Vec<SymbolDef>, pub footprints: Vec<FootprintDef>,
    pub devices: Vec<Device>, pub models: Vec<Model3D>,
}
pub struct Device {
    pub id: String, pub name: String, pub symbol: String,
    pub footprint: String, pub model: Option<String>,
    pub value: String, pub category: String,
    pub attributes: BTreeMap<String, String>,
}
pub struct Model3D {
    pub id: String, pub path: String, pub format: ModelFormat,
    pub unit: Unit, pub transform: [f64; 16],
}
```

- `SymbolDef`: pin + primitive (riusa `symbol_primitives`) + attributi.
- `FootprintDef`: pad, outline, courtyard, `model: Option<String>`.
- `Device`: l'oggetto che l'utente piazza; lega simbolo + footprint + modello.
- Il **modello 3D sta sul footprint/package**, non sul simbolo (come Fusion).
- Unità mm; trasformazione rigida (rotazione + traslazione) sul modello.

### Persistenza e formati

- Libreria built-in: `assets/ecad/library/*.json` (dati originali SolidCraft).
- Libreria utente: `.sclib` (JSON); una libreria per design è incorporata nel
  design (`ecad.library`).
- Import: KiCad `.kicad_sym`, `.kicad_mod`, footprint da `.kicad_pcb`; 3D da
  STEP (primario), STL/OBJ/GLB (fallback). L'import KiCad #5092 viene riusato
  e mappato su `SymbolDef`/`FootprintDef`.
- Export: `.sclib` JSON, STEP del modello; in futuro KiCad.

### Comandi

- `library.list {kind?: symbol|footprint|device|model}` → elenco + stato
  In Design / In Use / Available (come il Library Manager di Fusion).
- `library.enable {id, on}` → toggle "In Use".
- `library.import {path, kind}` → KiCad/STEP.
- `library.export {path}`.
- `library.add_device {name, symbol, footprint, model?, category?, value?}`.
- `library.replace {component, device}` → sostituisce il device del componente.
- `library.update {component}` → riallinea simbolo/footprint/modello.
- `component.place {device, at, rotation?}` → crea simbolo + footprint.
- `component.delete {id}`, `component.move {id, at, rotation?}`.
- `component.attach_model {footprint, path, transform?}`.

I comandi sono doc-only finché non hanno schema; il component manager li usa
via `app.run`.

### UI

- **Component Manager** in un dock a destra (o finestra mobile, come il
  Browser): albero `Libreria → Device → Simbolo/Footprint/3D`, ricerca,
  filtro categoria, toggle In Use, anteprima (canvas simbolo, canvas
  footprint, thumbnail 3D).
- Pulsanti: **Place** (attiva `Tool::Component(device)`), **Edit** (editor
  simbolo/footprint/3D), **Replace** sul componente selezionato.
- **Property panel**: reference, value, device, footprint, modello 3D,
  net class.
- **Library Manager**: colonne In Design / In Use / Available, come Fusion;
  "Add library" apre un file, "Import" KiCad.
- **Placement**: Place imposta il tool; il click sul canvas esegue
  `component.place` con le coordinate mondo; dopo il piazzamento il componente
  è selezionato e il tool torna Select.
- **Board 3D**: il modello del footprint viene disegnato nel viewport 3D
  (render esistente) e usato nel co-design MCAD (Fase 6).

## 3. Editing e interazione

- **Selezione**: singola/multipla (Shift), box select; evidenziazione per
  simbolo/footprint/rete; `Esc` annulla.
- **Move/Rotate/Mirror**: tool dedicati + drag; rotazione 90°/45°/libera;
  mirror per layer (F.Cu ↔ B.Cu).
- **Undo/Redo ECAD**: stack dedicato (le operazioni ECAD non entrano nella
  timeline MCAD); ogni comando espone undo/redo o un diff.
- **Snap**: griglia 0.001 mm (netlist), 0.1 mm per il layout, snap ai pad,
  ai pin, ai centri via; ortho per wire/track.
- **Layer**: selettore layer attivo, visibilità, layer stack manager; route
  multi-layer con via automatica nello stesso punto.
- **Net**: netlist condivisa schematico↔PCB; ratsnest; highlight della rete;
  net class; DRC/ERC con lista errori cliccabile (zoom sull'errore).
- **Testo/quote**: label, valore, reference, dimensioni.
- **Misure**: distanza, lunghezza traccia, area rame.

## 4. Milestone

- **M1 — Ribbon**: tab SCHEMATIC/PCB contestuali, pannelli e comandi sopra,
  icone riusate dove possibile, label in pending (en+it), test del catalogo
  tab/panel. La toolbar compatta del pannello resta come fallback.
- **M2 — Libreria + Component Manager**: modello `library`, import KiCad/STEP,
  pannello con ricerca/anteprima/Place/Replace, `component.place`; test del
  modello e dei comandi.
- **M3 — Editor**: Symbol Editor (pin, primitive), Footprint Editor (pad,
  outline, courtyard), 3D Model attach; libreria built-in con i device
  standard; board 3D con i modelli.
- **M4 — Verifica**: ERC, DRC, net class, ratsnest, design rules; pannello
  errori; test con board di esempio.
- **M5 — Sync e output**: sincronizzazione schematico↔PCB (annotate,
  forward/backward), BOM, Gerber/drill, export KiCad.
- **M6 — Eagle**: quando il workspace è abbastanza funzionante, importare
  file Eagle `.sch`/`.brd` (XML) per schematico e board, mappandoli su
  `Schematic`/`Pcb` e sulla libreria (device, symbol, package); test di
  compatibilità con file Eagle di esempio. Dopo M4.

## 5. Decisioni aperte

- **Ribbon vs pannello**: il ribbon superiore è la destinazione; per i tool
  parametrici (Place Component) serve il component manager, quindi M1 espone
  i tool già parametrici e lascia la palette del pannello per i simboli
  finché M2 non arriva.
- **Formato libreria**: JSON `.sclib` + built-in; import KiCad; niente
  librerie commerciali copiate.
- **3D**: STEP primario (reader esistente), STL/OBJ/GLB fallback; mm.
- **Undo**: stack ECAD separato dalla timeline MCAD.
- **Layer stack**: 2 layer ora; il modello `Layer` esistente va esteso a
  stack (F.Cu, In1..InN, B.Cu, mask, silk, edge).

## 6. Prossimo passo immediato

1. Chiudere il merge `upstream/main` (build + test) e committare.
2. M1: `workspace::visible_tabs` → SCHEMATIC/PCB quando Electronics è attivo;
   `SECTIONS`/`panels` per i due tab; `.at(...)` sui comandi; label en+it.
3. M2: `crates/ecad/src/library.rs` + comandi `library.*` + Component Manager.
