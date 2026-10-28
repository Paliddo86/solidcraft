# SolidCraft ECAD — Handoff (come continuare)

> Ultimo aggiornamento: 2026-10-28 · Branch `ecad` · merge upstream `247f73b5` + M2d
> Questa pagina è il punto di ingresso per un nuovo agente: **non serve
> rianalizzare tutto**.

## Leggi, in quest'ordine

1. `AGENTS.md` — regole hard e skill router.
2. `docs/ecad-plan.md` — fasi, stato, log.
3. `docs/ecad-ui-plan.md` — piano UI/libreria (M1–M6) e redesign del manager.
4. `docs/ecad-research.md`, `docs/kicad-import.md` — ricerca e import KiCad.
5. `git status` e `git log --oneline -10` sul branch `ecad`.
6. Codice: `crates/ecad/`, `crates/engine/src/cmd/{schematic,pcb,library}.rs`,
   `crates/ui-egui/src/{electronics_workspace,workspace,toolbar}.rs`.
7. La skill dell'area (`skills/solidcraft-*`) per il tipo di lavoro.

## Stato

- **Fatto**: Fase 0–3 MVP, schematic capture, workspace Electronics, ribbon
  contestuale **SCHEMATIC/PCB** (M1), libreria + **Component Manager** (M2),
  **Replace** dalla libreria (M2b).
- **Prossimo**: redesign del Component Manager (riferimenti in
  `docs/ecad-ui-plan.md` §2b), import KiCad/STEP nella libreria (M2c),
  revisione comandi editing uno per uno (Fase 9), editor e board 3D (M3),
  DRC/ERC/ratsnest (M4), Eagle `.sch`/`.brd` (M6).
- **Upstream**: `upstream/main` è a `cee995e7` (nuovi commit dopo `ae07e36a`),
  **non ancora fuso**. Regola: dopo conferma dell'utente, **prima di un nuovo
  piano**, `git fetch upstream && git merge upstream/main`, risolvi i conflitti
  (main vince sulle parti comuni), poi build/test.

## Prompt per un nuovo agente

> "Leggi `AGENTS.md`, `docs/ecad-plan.md`, `docs/ecad-ui-plan.md` e
> `docs/ecad-handoff.md`. Siamo sul branch `ecad`, head `858268d8`. Riprendi dal
> prossimo punto del piano. Prima di un nuovo piano, aggiorna da
> `upstream/main` dopo conferma."

## Demo e controllo

- App: `./target/ci/solidcraft --control 47000`.
- Demo: `python3 /tmp/solidcraft-ecad-demo/library_demo.py`.
- Screenshot: `/tmp/solidcraft-ecad-demo/app-m2.png` (Gestore componenti).
