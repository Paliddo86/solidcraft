# SolidCraft — Piano ECAD (PCB / Electronics)

> Stato: **Fase 7 (UI workspace) MVP: toolbar + editing completati** · Prossimo passo: pan/zoom via `navigation::handle`, DRC e ratsnest · Ultimo aggiornamento: 2026-10-26
> Base: branch `ecad` allineato a `upstream/main` (1cd14e24); PR #5092 portato con merge.

Scopo: integrare un workspace di progettazione elettronica stile Fusion360
"Electronics" (schematico + PCB + librerie + output di produzione), partendo
dall'import KiCad già avviato upstream (#5091 / #5092).

## Come usare e aggiornare questo file (protocollo per agenti)
1. All'inizio di ogni sessione leggi: questo file, `AGENTS.md`, `ROADMAP.md`,
   `docs/architecture.md` e lo stato git (`git status`, `git log --oneline -5`).
2. Lavora su UNA voce alla volta. Spunta `[x]` solo quando è fatta E i test
   passano (`cargo xtask ci` per le crate toccate; minimo `cargo test -p <crate>`).
3. Prima di iniziare una voce controlla upstream (issue/PR) per lavoro già
   esistente o claim attivi (#5091/#5092 per l'import KiCad).
4. Sposta il marcatore `▶ IN CORSO` sulla voce attuale; non spuntare voci future.
5. Al termine di ogni voce aggiorna "Stato" e "Prossimo passo" in alto, e
   aggiungi una riga al "Log" (data, cosa è cambiato, prossimo passo).
6. Non cancellare o riordinare le voci completate; aggiungi nuove voci in coda
   alla loro fase. Ogni arco di lavoro = un commit/PR (vedi CONTRIBUTING).

## Regole non negoziabili (richiamo)
- Clean-room: niente lettura/copia di codice GPL (KiCad, LibrePCB, Horizon EDA,
  QElectroTech, WireViz, Fritzing sono GPL). Solo specifiche pubbliche dei formati.
- Niente prodotti CAD commerciali: non pilotare, scriptare, automatizzare o
  estrarre dati da Fusion360/SolidWorks/Onshape; **non fare reverse engineering**
  dei loro binari (anche solo per "studiare il comportamento"): la ricerca usa
  solo documentazione pubblica + esperienza personale d'uso (#1624).
- "Everything is a command" (`CommandSpec`, id `pcb.*`/`schematic.*`).
- Layering `cargo xtask layers`; UI thin; icone solo dai master originali
  (leggi `assets/AGENTS.md` prima di ogni icona).

## Build e anteprima veloce (dev)
La macchina di sviluppo può essere lenta: preferire questi percorsi.
- **Feedback veloce (senza codegen)**: `cargo check -p <crate>`.
- **Profilo consigliato dal repo**: `--profile ci` (line tables, niente
  debug-assertions; deps a opt-level 2). È il profilo dei test CI.
- **Compilazione più rapida in assoluto** (verificato: profilo "unoptimized"):
  ```sh
  cargo build --profile ci \
    --config 'profile.ci.opt-level=0' \
    --config 'profile.ci.package."*".opt-level=0' \
    -p solidcraft-cli
  ```
  Compila a opt-level 0 (build molto più veloce, runtime più lento): adatto
  all'anteprima, non ai test.
- **Anteprima headless (senza GUI)**: costruire `solidcraft-cli`, non l'app
  desktop (la CLI non trascina egui/eframe). Uno script JSON con `doc.open`,
  poi:
  ```sh
  solidcraft-cli snapshot preview.json --out /tmp/board.png --view iso
  ```
  (con `SOLIDCRAFT_GPU=cpu` se non c'è un adapter GPU).
- **Evitare ricompilazioni inutili**: `-p` diversi possono risolvere feature
  diverse e ricompilare i deps pesanti (es. `cadmpeg-ir`). Usare una sola
  invocazione con più `-p` quando possibile.
- `sccache` è usato in CI ma non è indispensabile in locale (cache tra build
  pulite).

## Riferimenti upstream
- #1624 Research Request: Electrical (linee guida ricerca + regole legali)
- #5091 / #5092 KiCad PCB import (in progress, ~5.500 righe, 20 file)
- #2825 roadmap: PCB "deferred"

## Fase 0 — Groundwork & ricerca clean-room (nessun codice)
- [x] 0.1 Leggere integralmente #1624 e #5091; riassumerle qui.
- [x] 0.2 Scrivere `docs/ecad-research.md` secondo il layout di #1624
       (fundamentals, accuracy, programs, glossary, sources).
- [x] 0.3 Creare `tools/kicad-oracle/` (KiCad come processo esterno, stile
       freecad-oracle) per generare `.kicad_pcb`/`.kicad_sch` di riferimento.

## Fase 1 — Import KiCad: riprendere il lavoro upstream (#5092)
- [x] 1.1 Recuperare il codice di #5092 (patch pubblica o fetch del fork
       ahagelberg/solidcraft) e studiarlo; nessun claim in conflitto.
- [x] 1.2 Portare parser `.kicad_pcb` (s-expression), outline/cutout/drill,
       copper e footprint/componenti nel fork (MIT/Apache, stesso progetto).
- [x] 1.3 Registrare il formato in `crates/io/src/formats.rs` + comando
       `doc.open` con opzioni (thickness, copper, silkscreen, color, components).
- [x] 1.4 Portare/adattare i test (46 io + 4 engine) e farli passare.
       ▶ io: 46/46 OK · engine: 4/4 OK

## Fase 2 — Modello dati ECAD nativo + crate
- [x] 2.0 Creare `crates/solidcraft-ecad` (L3) con il modello **schematico**
       (`Symbol`, `Pin`, `Wire`, `Label`, `Net`, libreria simboli) e la netlist.
- [x] 2.1 Modello **board** (board/stackup/layer/track/pad/via/footprint/net/zone)
       nello stesso crate, distinto dall'import MCAD (`crates/ecad/src/pcb.rs`).
- [x] 2.2 Vincoli di sicurezza: `get()` non `[i]`, check finite, cap su loop,
       `validate()` sul file letto.
- [x] 2.3 Test unitari del modello (5 test in `solidcraft-ecad`).

## Fase 3 — Board authoring MVP (core workspace PCB)
- [x] 3.1 `pcb.create` (documento board) + outline da dimensioni.
- [x] 3.2 Stackup: layer rame/silk/mechanical + spessore (prepreg/dielettrico: dopo).
- [x] 3.3 Footprint/pad (SMD, THT) e libreria componenti (r_0603, c_0603,
       led_0603, ic8_dip, header_2, mounting_hole).
- [x] 3.4 Vias, tracks (routing), copper zones (testo silkscreen: ancora da fare).
- [x] 3.5 Nets e riepilogo connettività (ratsnest/airwires: dopo).
- [ ] 3.6 DRC: clearance, larghezza traccia, drill, annular ring.
- [x] 3.7 Simboli schematici **standard** (resistenza IEC, condensatore a piastre,
       induttanza, diodo, LED, IC a 8 pin, ground, VCC, connettore) disegnati da
       `symbol_primitives` in SVG e nella UI.

## Fase 4 — Schematic capture
- [x] 4.1 Documento schematico, simboli, wire, net label (`schematic.create`,
       `schematic.symbol.place`, `schematic.wire`, `schematic.label`,
       `schematic.info`, `schematic.save`, `schematic.open`,
       `schematic.export_svg`; CLI `schematic` con `--svg`).
- [x] 4.2 Netlist da schematico (unione dei punti su griglia 0.001 mm).
- [ ] 4.3 Sincronizzazione schematico ↔ PCB (forward/back annotation).

## Fase 5 — Export
- [ ] 5.1 Gerber RS-274X/X2 + Excellon drill.
- [ ] 5.2 IPC-2581 e ODB++.
- [ ] 5.3 STEP AP242 per co-design MCAD; BOM e pick-and-place.

## Fase 6 — Integrazione MCAD
- [ ] 6.1 Board 3D nel design (substrato, rame, componenti STEP).
- [ ] 6.2 Co-design enclosure↔board: keep-in/keep-out, clearance.

## Fase 7 — UI workspace
- [x] 7.0 Finestra **Schematic** read-only nell'app (egui painter, si apre da
       sola quando esiste uno schema; `crates/ui-egui/src/schematic_panel.rs`).
- [x] 7.4 Finestra **PCB** read-only nell'app (outline, zone, track, pad, via;
       `crates/ui-egui/src/pcb_panel.rs`).
- [x] 7.5 Workspace **Electronics** come Design/Render: voce nel selettore,
       central panel con tab **Schematic/PCB** e i due canvas
       (`crates/ui-egui/src/electronics_workspace.rs`); il selettore passa tra
       Design/Render/Electronics e il canale di controllo accetta `electronics`.
- [x] 7.6a Toolbar **Electronics** dedicata nel workspace: tool Select, palette
       simboli (Resistor, Capacitor, IC, Ground, VCC, Connector) e footprint
       (R 0603, C 0603, DIP-8, Header, Hole), Wire/Net Label/Track/Via/Copper
       Zone, Finish e Delete; ogni pulsante esegue i comandi `schematic.*`/`pcb.*`.
- [ ] 7.6b Pan/zoom del canvas via `navigation::handle` (orbit off).
- [x] 7.7a Editing nella UI: click-to-place, selezione (hit test), trascinamento
       di simboli/footprint (`*.symbol.move`/`*.footprint.move`), routing di
       wire/track a più punti, label, via, zona rettangolare, delete.
- [ ] 7.7b DRC e ratsnest nel workspace.
- [ ] 7.1 Tab "Electronics" con `ui_kit::Container`, theme tokens.
- [ ] 7.2 Canvas 2D board/schematic via `navigation::handle` (orbit off).
- [ ] 7.3 Pannelli/toolbar via `app.run(id, params)` / `app.start(id)`.

## Log
- 2026-10-26 — Piano creato; repo allineato a upstream/main (1cd14e24); nessun
  lavoro iniziato.
- 2026-10-26 — Recuperato `refs/pull/5092/head` e portato con merge in `ecad`
  (20 file, +5527). `cargo check -p solidcraft-io --lib` OK (6m20s).
- 2026-10-26 — `cargo test -p solidcraft-io --lib kicad`: **46 passed, 0 failed**.
- 2026-10-26 — Metodo veloce (opt-level 0 via `--config`): i 4 test
  `cargo test -p solidcraft-engine --lib kicad_import` passano, build in
  **8m25s** (il profilo lento era a 26+ min sui soli deps). **Fase 1 completata**.
- 2026-10-26 — Fase 4 MVP: nuova crate `solidcraft-ecad` (L3) con modello
  schematico e netlist; comandi `schematic.*`; CLI `schematic`. Build CLI
  **1m10s** (opt-level 0), test **5/5** (ecad) e **4/4** (engine). Demo:
  netlist VIN/FILTER/OUT/GND/VCC corretta.
- 2026-10-26 — In attesa di conferma prima del modello board (Fase 2/3).
- 2026-10-26 — **Fase 0 completata**: `docs/ecad-research.md` (ricerca clean-room)
  e `tools/kicad-oracle/` (KiCad come programma esterno). Aggiunto
  `schematic.export_svg` + CLI `--svg` per vedere lo schema.
- 2026-10-27 — App desktop compilata (12m32s + 3m54s) e avviata; aggiunta la
  finestra **Schematic** read-only (egui painter, `schematic_panel.rs`), che si
  apre da sola quando esiste uno schema. Screenshot dell'app con lo schema
  "Power Supply" verificato.
- 2026-10-27 — Allineato a `upstream/main` (191 commit, merge `a11a16c3`).
- 2026-10-27 — **Fase 2.1 + Fase 3 MVP**: modello board in
  `crates/ecad/src/pcb.rs` (footprint/pad, track, via, zone, layer, net) e
  comandi `pcb.*`; CLI `pcb` con `--svg`; finestra **PCB** nella UI. Test
  **10/10** (ecad) e **4/4** (engine). Demo "Power Supply": 8 footprint,
  7 track, 3 via, 1 zone, net VIN/FILTER/OUT/GND/VCC.
- 2026-10-27 — Simboli **standard** (IEC/forme convenzionali) via
  `symbol_primitives`, usati da SVG e UI.
- 2026-10-27 — Workspace **Electronics** integrato come Design/Render:
  selettore, central panel con tab Schematic/PCB, canvas. Priorità aggiornata:
  prima la grafica della sezione Electronics, poi DRC/ratsnest/editing.
- 2026-10-27 — **Fase 7.6a + 7.7a completate**: toolbar Electronics (Seleziona,
  palette simboli/footprint, Filo/Etichetta/Traccia/Via/Zona, Termina/Elimina)
  ed editing nel canvas: piazzamento, selezione, trascinamento, routing,
  label, via, zona, delete. Nuovi comandi `schematic.symbol.move/delete` e
  `pcb.footprint.move/delete`. Test: ecad **10/10**, engine schematic **6/6**,
  pcb **5/5**. Layout corretto sotto il ribbon e a destra del browser.
  Localizzazione **italiana** di tutte le nuove stringhe (toolbar, comandi,
  workspace, dialogo import KiCad) via `assets/locales/pending/`.
  La card del workspace mostra **Elettronica**. Screenshot:
  `/tmp/solidcraft-ecad-demo/app-editing-final.png` (R2 piazzato via
  toolbar+canvas e trascinato), `app-editing-it.png` (schematico) e
  `app-editing-it-pcb.png` (PCB). i18n: **92/93** test; l'unico rimasto
  confronta i cataloghi completi e si risolve col fold dei pending su main
  (`cargo xtask fold-pending`); `cargo xtask lists` passa.
- 2026-10-27 — Prossimo passo: pan/zoom via `navigation::handle`, DRC e
  ratsnest; poi sincronizzazione schematico↔PCB (4.3).
