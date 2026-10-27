//! Schematic capture: the schematic document of its own (symbols, wires, net labels) and the
//! basic commands that create it. The model and the netlist are in `solidcraft_ecad`.

use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{bad, num, str_};
use crate::{Result, Session};

const CREATE: &str = "schematic.create";
const PLACE: &str = "schematic.symbol.place";
const WIRE: &str = "schematic.wire";
const LABEL: &str = "schematic.label";
const INFO: &str = "schematic.info";
const SAVE: &str = "schematic.save";
const OPEN: &str = "schematic.open";
const SVG: &str = "schematic.export_svg";

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new(CREATE, "New Schematic", create).noundo().doc_only().params(
        "name?: the sheet's name (default \"Schematic\"); width_mm?, height_mm?: the sheet's size in millimetres (default A4 landscape, 297 × 210) → name, width_mm, height_mm",
    ),
    CommandSpec::new(PLACE, "Place Symbol", place).noundo().doc_only().params(
        "kind: resistor | capacitor | inductor | diode | led | ic8 | ground | vcc | connector2; at: [x, y] in mm; reference?: the reference (default R1, C1, U1… by kind); value?: its value; rotation?: degrees → id, reference, kind, at, pins [{number, name, at}]",
    ),
    CommandSpec::new(WIRE, "Wire", wire).noundo().doc_only().params(
        "points: [[x, y], …] (at least two, in mm); net?: a name for this wire → id, points",
    ),
    CommandSpec::new(LABEL, "Net Label", label).noundo().doc_only().params(
        "net: the net's name (1 to 128 characters); at: [x, y] in mm → id, net, at",
    ),
    CommandSpec::new(INFO, "Schematic", info).noundo().doc_only().params(
        "→ exists, name, width_mm, height_mm, symbols, wires, labels, nets [{name, pins: [{symbol, reference, pin, name}]}]",
    ),
    CommandSpec::new(SAVE, "Save Schematic", save).noundo().doc_only().params(
        "path: write the schematic as JSON (a `.solidcraft-sch` file) → path, bytes",
    ),
    CommandSpec::new(OPEN, "Open Schematic", open).noundo().doc_only().params(
        "path: read a schematic JSON file (replaces the session's schematic) → path, name, symbols, wires, labels",
    ),
    CommandSpec::new(SVG, "Export Schematic SVG", export_svg).noundo().doc_only().params(
        "path: write the schematic as an SVG picture (a quick look at it) → path, bytes",
    ),
];

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "name").unwrap_or("Schematic").trim();
    if name.chars().count() > 128 {
        return Err(bad(CREATE, "`name` holds at most 128 characters"));
    }
    let mut sch = solidcraft_ecad::Schematic::default();
    sch.name = if name.is_empty() { "Schematic".to_string() } else { name.to_string() };
    if p.get("width_mm").is_some() {
        sch.width_mm = num(p, "width_mm").ok_or_else(|| bad(CREATE, "`width_mm` must be a finite number of millimetres"))?;
    }
    if p.get("height_mm").is_some() {
        sch.height_mm = num(p, "height_mm").ok_or_else(|| bad(CREATE, "`height_mm` must be a finite number of millimetres"))?;
    }
    if !(1.0..=10_000.0).contains(&sch.width_mm) {
        return Err(bad(CREATE, "`width_mm` must be 1 to 10000 mm"));
    }
    if !(1.0..=10_000.0).contains(&sch.height_mm) {
        return Err(bad(CREATE, "`height_mm` must be 1 to 10000 mm"));
    }
    let (name, width_mm, height_mm) = (sch.name.clone(), sch.width_mm, sch.height_mm);
    s.schematic = Some(sch);
    Ok(json!({"name": name, "width_mm": width_mm, "height_mm": height_mm}))
}

fn place(s: &mut Session, p: &Value) -> Result<Value> {
    let kind = str_(p, "kind")
        .ok_or_else(|| bad(PLACE, "`kind` is the symbol's kind (resistor, capacitor, inductor, diode, led, ic8, ground, vcc, connector2)"))?;
    let at = point(p, "at").ok_or_else(|| bad(PLACE, "`at` must be a point [x, y] in millimetres"))?;
    let rotation = match p.get("rotation") {
        None | Some(Value::Null) => 0.0,
        Some(_) => num(p, "rotation").ok_or_else(|| bad(PLACE, "`rotation` must be a finite number of degrees"))?,
    };
    let reference = str_(p, "reference");
    let value = str_(p, "value");
    let sch = s.schematic.as_mut().ok_or_else(|| bad(PLACE, "no schematic yet: run schematic.create first"))?;
    let (id, reference) = sch.place(kind, at, reference, value, rotation).map_err(|why| bad(PLACE, why))?;
    let symbol = sch.symbols.iter().find(|x| x.id == id).ok_or_else(|| bad(PLACE, "the symbol was not placed"))?;
    let pins: Vec<Value> = symbol.pins.iter().map(|pin| json!({"number": pin.number, "name": pin.name, "at": symbol.pin_world(pin)})).collect();
    Ok(json!({"id": id, "reference": reference, "kind": symbol.kind, "at": symbol.at, "pins": pins}))
}

fn wire(s: &mut Session, p: &Value) -> Result<Value> {
    let pts = points(p, "points", solidcraft_ecad::MAX_WIRE_POINTS)?;
    let net = str_(p, "net");
    let sch = s.schematic.as_mut().ok_or_else(|| bad(WIRE, "no schematic yet: run schematic.create first"))?;
    let id = sch.add_wire(&pts, net).map_err(|why| bad(WIRE, why))?;
    Ok(json!({"id": id, "points": pts}))
}

fn label(s: &mut Session, p: &Value) -> Result<Value> {
    let net = str_(p, "net").ok_or_else(|| bad(LABEL, "`net` is the net's name"))?;
    let at = point(p, "at").ok_or_else(|| bad(LABEL, "`at` must be a point [x, y] in millimetres"))?;
    let sch = s.schematic.as_mut().ok_or_else(|| bad(LABEL, "no schematic yet: run schematic.create first"))?;
    let id = sch.add_label(net, at).map_err(|why| bad(LABEL, why))?;
    Ok(json!({"id": id, "net": net.trim(), "at": at}))
}

fn info(s: &mut Session, _p: &Value) -> Result<Value> {
    match &s.schematic {
        Some(sch) => {
            let mut v = sch.info();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("exists".to_string(), json!(true));
            }
            Ok(v)
        }
        None => Ok(json!({"exists": false})),
    }
}

fn export_svg(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(SVG, "`path` is where to write the SVG"))?;
    let sch = s.schematic.as_ref().ok_or_else(|| bad(SVG, "no schematic yet: run schematic.create first"))?;
    let text = sch.to_svg();
    std::fs::write(path, text.as_bytes()).map_err(|why| bad(SVG, format!("{path}: {why}")))?;
    Ok(json!({"path": path, "bytes": text.len()}))
}

fn save(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(SAVE, "`path` is where to write the schematic"))?;
    let sch = s.schematic.as_ref().ok_or_else(|| bad(SAVE, "no schematic yet: run schematic.create first"))?;
    let text = sch.to_json().map_err(|why| bad(SAVE, format!("the schematic could not be written: {why}")))?;
    std::fs::write(path, text.as_bytes()).map_err(|why| bad(SAVE, format!("{path}: {why}")))?;
    Ok(json!({"path": path, "bytes": text.len()}))
}

fn open(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(OPEN, "`path` is the schematic file to read"))?;
    let meta = std::fs::metadata(path).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    if !meta.is_file() {
        return Err(bad(OPEN, format!("{path}: not a file")));
    }
    if meta.len() > solidcraft_ecad::MAX_SCHEMATIC_BYTES {
        return Err(bad(OPEN, format!("{path}: larger than {} bytes", solidcraft_ecad::MAX_SCHEMATIC_BYTES)));
    }
    let bytes = std::fs::read(path).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    let sch = solidcraft_ecad::Schematic::from_json(&bytes).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    sch.validate().map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    let (name, symbols, wires, labels) = (sch.name.clone(), sch.symbols.len(), sch.wires.len(), sch.labels.len());
    s.schematic = Some(sch);
    Ok(json!({"path": path, "name": name, "symbols": symbols, "wires": wires, "labels": labels}))
}

/// A finite `[x, y]` point given under `k`.
fn point(p: &Value, k: &str) -> Option<[f64; 2]> {
    let a = p.get(k)?.as_array()?;
    if a.len() != 2 {
        return None;
    }
    Some([coordinate(a.first()?)?, coordinate(a.get(1)?)?])
}

fn coordinate(v: &Value) -> Option<f64> {
    v.as_f64().filter(|x| x.is_finite() && x.abs() < 1e9)
}

/// A list of finite `[x, y]` points given under `k`.
fn points(p: &Value, k: &str, max: usize) -> Result<Vec<[f64; 2]>> {
    let a = p.get(k).and_then(Value::as_array).ok_or_else(|| bad(WIRE, format!("`{k}` must be a list of points")))?;
    if a.len() > max {
        return Err(bad(WIRE, format!("`{k}` has more than {max} points")));
    }
    a.iter()
        .map(|v| {
            let b = v.as_array().filter(|b| b.len() == 2).ok_or_else(|| bad(WIRE, format!("`{k}` must contain [x, y] points")))?;
            let x = b.first().and_then(coordinate).ok_or_else(|| bad(WIRE, format!("`{k}` must contain finite [x, y] points")))?;
            let y = b.get(1).and_then(coordinate).ok_or_else(|| bad(WIRE, format!("`{k}` must contain finite [x, y] points")))?;
            Ok([x, y])
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::Session;

    fn run(s: &mut Session, id: &str, p: Value) -> Value {
        s.execute(id, &p).unwrap()
    }

    #[test]
    fn a_schematic_is_created_placed_and_wired() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({"name": "Power", "width_mm": 200, "height_mm": 100}));
        let r = run(&mut s, PLACE, json!({"kind": "resistor", "at": [50, 50], "value": "10k"}));
        assert_eq!(r["reference"], json!("R1"));
        let pin = r["pins"][0]["at"][0].as_f64().unwrap();
        assert!((pin - 46.19).abs() < 1e-9, "{pin}");
        run(&mut s, WIRE, json!({"points": [[46.19, 50], [40, 50]]}));
        run(&mut s, LABEL, json!({"net": "VCC", "at": [40, 50]}));
        let info = run(&mut s, INFO, json!({}));
        assert_eq!(info["exists"], json!(true));
        assert_eq!(info["name"], json!("Power"));
        assert_eq!(info["symbols"][0]["reference"], json!("R1"));
        let nets = info["nets"].as_array().unwrap();
        assert!(nets.iter().any(|n| n["name"] == json!("VCC") && n["pins"][0]["reference"] == json!("R1")), "{nets:?}");
    }

    #[test]
    fn commands_before_create_say_so() {
        let mut s = Session::default();
        let e = s.execute(PLACE, &json!({"kind": "resistor", "at": [0, 0]})).unwrap_err();
        assert!(format!("{e}").contains("schematic.create"), "{e}");
        assert_eq!(run(&mut s, INFO, json!({}))["exists"], json!(false));
    }

    #[test]
    fn a_bad_symbol_or_point_is_refused() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({}));
        let e = s.execute(PLACE, &json!({"kind": "flux-capacitor", "at": [0, 0]})).unwrap_err();
        assert!(format!("{e}").contains("unknown symbol"), "{e}");
        let e = s.execute(PLACE, &json!({"kind": "resistor", "at": ["x", 0]})).unwrap_err();
        assert!(format!("{e}").contains("`at`"), "{e}");
        let e = s.execute(WIRE, &json!({"points": [[0, 0]]})).unwrap_err();
        assert!(format!("{e}").contains("at least two"), "{e}");
    }

    #[test]
    fn a_saved_schematic_opens_again() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({"name": "Power"}));
        run(&mut s, PLACE, json!({"kind": "ic8", "at": [100, 100], "reference": "U1", "value": "NE555"}));
        let dir = std::env::temp_dir().join(format!("solidcraft-schematic-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("power.solidcraft-sch");
        run(&mut s, SAVE, json!({"path": path.to_string_lossy()}));
        let mut t = Session::default();
        let r = run(&mut t, OPEN, json!({"path": path.to_string_lossy()}));
        assert_eq!(r["symbols"], json!(1));
        assert_eq!(r["name"], json!("Power"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_schematic_exports_an_svg() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({}));
        run(&mut s, PLACE, json!({"kind": "resistor", "at": [50, 50]}));
        let dir = std::env::temp_dir().join(format!("solidcraft-schematic-svg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("schematic.svg");
        let r = run(&mut s, SVG, json!({"path": path.to_string_lossy()}));
        assert!(r["bytes"].as_u64().unwrap() > 100, "{r}");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("<svg"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
