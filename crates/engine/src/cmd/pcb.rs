//! Board authoring: the board document of its own (footprints, tracks, vias, zones, layers) and
//! the basic commands that create it. The model is in `solidcraft_ecad::pcb`.

use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{bad, num, str_};
use crate::{Result, Session};

const CREATE: &str = "pcb.create";
const PLACE: &str = "pcb.footprint.place";
const MOVE_FOOTPRINT: &str = "pcb.footprint.move";
const DELETE_FOOTPRINT: &str = "pcb.footprint.delete";
const SELECT: &str = "pcb.select";
const FINISH: &str = "pcb.finish";
const DELETE_SELECTED: &str = "pcb.delete";
const PAD_NET: &str = "pcb.pad.net";
const TRACK: &str = "pcb.track";
const VIA: &str = "pcb.via";
const ZONE: &str = "pcb.zone";
const INFO: &str = "pcb.info";
const SAVE: &str = "pcb.save";
const OPEN: &str = "pcb.open";
const SVG: &str = "pcb.export_svg";

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new(SELECT, "Select", ui_tool).noundo().doc_only().at("PCB", "DESIGN"),
    CommandSpec::new(DELETE_SELECTED, "Delete", ui_tool).noundo().doc_only().at("PCB", "DESIGN"),
    CommandSpec::new(FINISH, "Finish Track", ui_tool).noundo().doc_only().at("PCB", "ROUTE"),
    CommandSpec::new(CREATE, "New PCB", create).noundo().doc_only().at("PCB", "FILE").params(
        "name?: the board's name (default \"PCB\"); width_mm?, height_mm?: the outline in millimetres (default 80 × 50); thickness_mm?: the substrate (default 1.6) → name, width_mm, height_mm, thickness_mm",
    ),
    CommandSpec::new(PLACE, "Place Footprint", place).noundo().doc_only().at("PCB", "PLACE").params(
        "kind: r_0603 | c_0603 | led_0603 | ic8_dip | header_2 | mounting_hole; at: [x, y] in mm; reference?: default R1, C1, U1… by kind; value?; rotation?: degrees; layer?: F.Cu | B.Cu → id, reference, kind, at, pads [{number, at, size, shape, drill_mm}]",
    ),
    CommandSpec::new(MOVE_FOOTPRINT, "Move Footprint", move_footprint).noundo().doc_only().params(
        "id: the footprint's id; at: its new point [x, y] in mm; rotation?: degrees → id, at",
    ),
    CommandSpec::new(DELETE_FOOTPRINT, "Delete Footprint", delete_footprint).noundo().doc_only().params(
        "id: the footprint's id → id",
    ),
    CommandSpec::new(PAD_NET, "Pad Net", pad_net).noundo().doc_only().params(
        "reference: the footprint's reference; pad: the pad's number; net?: the net's name (none clears it) → reference, pad, net",
    ),
    CommandSpec::new(TRACK, "Track", track).noundo().doc_only().at("PCB", "ROUTE").params(
        "points: [[x, y], …] (at least two, in mm); width_mm?: 0.01 to 100 (default 0.25); layer?: F.Cu | B.Cu; net? → id, points, width_mm, layer",
    ),
    CommandSpec::new(VIA, "Via", via).noundo().doc_only().at("PCB", "PLACE").params(
        "at: [x, y] in mm; drill_mm?: 0.05 to 10 (default 0.4); size_mm?: at least the drill (default twice it); from?, to?: layers (default F.Cu → B.Cu); net? → id, at, drill_mm, size_mm",
    ),
    CommandSpec::new(ZONE, "Copper Zone", zone).noundo().doc_only().at("PCB", "BOARD").params(
        "points: a closed polygon of at least three [x, y] points (do not repeat the first); layer?: F.Cu | B.Cu; net? → id, points, layer, net",
    ),
    CommandSpec::new(INFO, "PCB", info).noundo().doc_only().at("PCB", "INSPECT").params(
        "→ exists, name, width_mm, height_mm, thickness_mm, layers, footprints, tracks, vias, zones, nets [{name, pads, tracks, vias, zones}]",
    ),
    CommandSpec::new(SAVE, "Save PCB", save).noundo().doc_only().params(
        "path: write the board as JSON (a `.solidcraft-pcb` file) → path, bytes",
    ),
    CommandSpec::new(OPEN, "Open PCB", open).noundo().doc_only().params(
        "path: read a board JSON file (replaces the session's board) → path, name, footprints, tracks, vias, zones",
    ),
    CommandSpec::new(SVG, "Export PCB SVG", export_svg).noundo().doc_only().params(
        "path: write the board as an SVG picture (outline, zones, tracks, pads, vias) → path, bytes",
    ),
];

/// UI-only: the Electronics workspace starts the matching tool; a script call is a no-op.
fn ui_tool(_s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({}))
}

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_(p, "name").unwrap_or("PCB").trim();
    if name.chars().count() > 128 {
        return Err(bad(CREATE, "`name` holds at most 128 characters"));
    }
    let mut board = solidcraft_ecad::Pcb::default();
    board.name = if name.is_empty() { "PCB".to_string() } else { name.to_string() };
    if p.get("width_mm").is_some() {
        board.width_mm = num(p, "width_mm").ok_or_else(|| bad(CREATE, "`width_mm` must be a finite number of millimetres"))?;
    }
    if p.get("height_mm").is_some() {
        board.height_mm = num(p, "height_mm").ok_or_else(|| bad(CREATE, "`height_mm` must be a finite number of millimetres"))?;
    }
    if p.get("thickness_mm").is_some() {
        board.thickness_mm = num(p, "thickness_mm").ok_or_else(|| bad(CREATE, "`thickness_mm` must be a finite number of millimetres"))?;
    }
    if !(1.0..=10_000.0).contains(&board.width_mm) {
        return Err(bad(CREATE, "`width_mm` must be 1 to 10000 mm"));
    }
    if !(1.0..=10_000.0).contains(&board.height_mm) {
        return Err(bad(CREATE, "`height_mm` must be 1 to 10000 mm"));
    }
    if !(0.1..=20.0).contains(&board.thickness_mm) {
        return Err(bad(CREATE, "`thickness_mm` must be 0.1 to 20 mm"));
    }
    let (name, width_mm, height_mm, thickness_mm) = (board.name.clone(), board.width_mm, board.height_mm, board.thickness_mm);
    s.pcb = Some(board);
    Ok(json!({"name": name, "width_mm": width_mm, "height_mm": height_mm, "thickness_mm": thickness_mm}))
}

fn place(s: &mut Session, p: &Value) -> Result<Value> {
    let kind =
        str_(p, "kind").ok_or_else(|| bad(PLACE, "`kind` is the footprint's kind (r_0603, c_0603, led_0603, ic8_dip, header_2, mounting_hole)"))?;
    let at = point(p, "at").ok_or_else(|| bad(PLACE, "`at` must be a point [x, y] in millimetres"))?;
    let rotation = match p.get("rotation") {
        None | Some(Value::Null) => 0.0,
        Some(_) => num(p, "rotation").ok_or_else(|| bad(PLACE, "`rotation` must be a finite number of degrees"))?,
    };
    let reference = str_(p, "reference");
    let value = str_(p, "value");
    let layer = str_(p, "layer").unwrap_or("F.Cu");
    let board = s.pcb.as_mut().ok_or_else(|| bad(PLACE, "no board yet: run pcb.create first"))?;
    let (id, reference) = board.place_footprint(kind, at, reference, value, rotation, layer).map_err(|why| bad(PLACE, why))?;
    let f = board.footprints.iter().find(|f| f.id == id).ok_or_else(|| bad(PLACE, "the footprint was not placed"))?;
    let pads: Vec<Value> = f
        .pads
        .iter()
        .map(|pad| json!({"number": pad.number, "at": f.pad_world(pad), "size": pad.size, "shape": pad.shape, "drill_mm": pad.drill_mm}))
        .collect();
    Ok(json!({"id": id, "reference": reference, "kind": f.kind, "at": f.at, "pads": pads}))
}

fn move_footprint(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p.get("id").and_then(Value::as_u64).ok_or_else(|| bad(MOVE_FOOTPRINT, "`id` is the footprint's id"))?;
    let at = point(p, "at").ok_or_else(|| bad(MOVE_FOOTPRINT, "`at` must be a point [x, y] in millimetres"))?;
    let rotation = match p.get("rotation") {
        None | Some(Value::Null) => None,
        Some(_) => Some(num(p, "rotation").ok_or_else(|| bad(MOVE_FOOTPRINT, "`rotation` must be a finite number of degrees"))?),
    };
    let board = s.pcb.as_mut().ok_or_else(|| bad(MOVE_FOOTPRINT, "no board yet: run pcb.create first"))?;
    board.move_footprint(id, at, rotation).map_err(|why| bad(MOVE_FOOTPRINT, why))?;
    Ok(json!({"id": id, "at": at}))
}

fn delete_footprint(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p.get("id").and_then(Value::as_u64).ok_or_else(|| bad(DELETE_FOOTPRINT, "`id` is the footprint's id"))?;
    let board = s.pcb.as_mut().ok_or_else(|| bad(DELETE_FOOTPRINT, "no board yet: run pcb.create first"))?;
    board.delete_footprint(id).map_err(|why| bad(DELETE_FOOTPRINT, why))?;
    Ok(json!({"id": id}))
}

fn pad_net(s: &mut Session, p: &Value) -> Result<Value> {
    let reference = str_(p, "reference").ok_or_else(|| bad(PAD_NET, "`reference` is the footprint's reference"))?;
    let pad = str_(p, "pad").ok_or_else(|| bad(PAD_NET, "`pad` is the pad's number"))?;
    let net = str_(p, "net");
    let board = s.pcb.as_mut().ok_or_else(|| bad(PAD_NET, "no board yet: run pcb.create first"))?;
    board.set_pad_net(reference, pad, net).map_err(|why| bad(PAD_NET, why))?;
    Ok(json!({"reference": reference, "pad": pad, "net": net}))
}

fn track(s: &mut Session, p: &Value) -> Result<Value> {
    let pts = points(p, "points", solidcraft_ecad::MAX_TRACK_POINTS, TRACK)?;
    let width = match p.get("width_mm") {
        None | Some(Value::Null) => 0.25,
        Some(_) => num(p, "width_mm").ok_or_else(|| bad(TRACK, "`width_mm` must be a finite number of millimetres"))?,
    };
    let layer = str_(p, "layer").unwrap_or("F.Cu");
    let net = str_(p, "net");
    let board = s.pcb.as_mut().ok_or_else(|| bad(TRACK, "no board yet: run pcb.create first"))?;
    let id = board.add_track(&pts, width, layer, net).map_err(|why| bad(TRACK, why))?;
    Ok(json!({"id": id, "points": pts, "width_mm": width, "layer": layer}))
}

fn via(s: &mut Session, p: &Value) -> Result<Value> {
    let at = point(p, "at").ok_or_else(|| bad(VIA, "`at` must be a point [x, y] in millimetres"))?;
    let drill = match p.get("drill_mm") {
        None | Some(Value::Null) => 0.4,
        Some(_) => num(p, "drill_mm").ok_or_else(|| bad(VIA, "`drill_mm` must be a finite number of millimetres"))?,
    };
    let size = match p.get("size_mm") {
        None | Some(Value::Null) => (drill * 2.0).max(0.8),
        Some(_) => num(p, "size_mm").ok_or_else(|| bad(VIA, "`size_mm` must be a finite number of millimetres"))?,
    };
    let from = str_(p, "from").unwrap_or("F.Cu");
    let to = str_(p, "to").unwrap_or("B.Cu");
    let net = str_(p, "net");
    let board = s.pcb.as_mut().ok_or_else(|| bad(VIA, "no board yet: run pcb.create first"))?;
    let id = board.add_via(at, drill, size, from, to, net).map_err(|why| bad(VIA, why))?;
    Ok(json!({"id": id, "at": at, "drill_mm": drill, "size_mm": size, "from": from, "to": to}))
}

fn zone(s: &mut Session, p: &Value) -> Result<Value> {
    let polygon = points(p, "points", solidcraft_ecad::MAX_POLYGON_POINTS, ZONE)?;
    let layer = str_(p, "layer").unwrap_or("F.Cu");
    let net = str_(p, "net");
    let board = s.pcb.as_mut().ok_or_else(|| bad(ZONE, "no board yet: run pcb.create first"))?;
    let id = board.add_zone(&polygon, layer, net).map_err(|why| bad(ZONE, why))?;
    Ok(json!({"id": id, "points": polygon.len(), "layer": layer, "net": net}))
}

fn info(s: &mut Session, _p: &Value) -> Result<Value> {
    match &s.pcb {
        Some(board) => {
            let mut v = board.info();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("exists".to_string(), json!(true));
            }
            Ok(v)
        }
        None => Ok(json!({"exists": false})),
    }
}

fn save(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(SAVE, "`path` is where to write the board"))?;
    let board = s.pcb.as_ref().ok_or_else(|| bad(SAVE, "no board yet: run pcb.create first"))?;
    let text = board.to_json().map_err(|why| bad(SAVE, format!("the board could not be written: {why}")))?;
    std::fs::write(path, text.as_bytes()).map_err(|why| bad(SAVE, format!("{path}: {why}")))?;
    Ok(json!({"path": path, "bytes": text.len()}))
}

fn open(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(OPEN, "`path` is the board file to read"))?;
    let meta = std::fs::metadata(path).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    if !meta.is_file() {
        return Err(bad(OPEN, format!("{path}: not a file")));
    }
    if meta.len() > solidcraft_ecad::MAX_PCB_BYTES {
        return Err(bad(OPEN, format!("{path}: larger than {} bytes", solidcraft_ecad::MAX_PCB_BYTES)));
    }
    let bytes = std::fs::read(path).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    let board = solidcraft_ecad::Pcb::from_json(&bytes).map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    board.validate().map_err(|why| bad(OPEN, format!("{path}: {why}")))?;
    let (name, footprints, tracks, vias, zones) =
        (board.name.clone(), board.footprints.len(), board.tracks.len(), board.vias.len(), board.zones.len());
    s.pcb = Some(board);
    Ok(json!({"path": path, "name": name, "footprints": footprints, "tracks": tracks, "vias": vias, "zones": zones}))
}

fn export_svg(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_(p, "path").ok_or_else(|| bad(SVG, "`path` is where to write the SVG"))?;
    let board = s.pcb.as_ref().ok_or_else(|| bad(SVG, "no board yet: run pcb.create first"))?;
    let text = board.to_svg();
    std::fs::write(path, text.as_bytes()).map_err(|why| bad(SVG, format!("{path}: {why}")))?;
    Ok(json!({"path": path, "bytes": text.len()}))
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
fn points(p: &Value, k: &str, max: usize, cmd: &str) -> Result<Vec<[f64; 2]>> {
    let a = p.get(k).and_then(Value::as_array).ok_or_else(|| bad(cmd, format!("`{k}` must be a list of points")))?;
    if a.len() > max {
        return Err(bad(cmd, format!("`{k}` has more than {max} points")));
    }
    a.iter()
        .map(|v| {
            let b = v.as_array().filter(|b| b.len() == 2).ok_or_else(|| bad(cmd, format!("`{k}` must contain [x, y] points")))?;
            let x = b.first().and_then(coordinate).ok_or_else(|| bad(cmd, format!("`{k}` must contain finite [x, y] points")))?;
            let y = b.get(1).and_then(coordinate).ok_or_else(|| bad(cmd, format!("`{k}` must contain finite [x, y] points")))?;
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
    fn a_board_is_created_routed_and_read() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({"name": "Main", "width_mm": 60, "height_mm": 40}));
        let r = run(&mut s, PLACE, json!({"kind": "ic8_dip", "at": [30, 20], "reference": "U1", "value": "NE555"}));
        assert_eq!(r["reference"], json!("U1"));
        run(&mut s, PAD_NET, json!({"reference": "U1", "pad": "1", "net": "VIN"}));
        run(&mut s, TRACK, json!({"points": [[26.19, 16.19], [10, 16.19]], "width_mm": 0.4, "net": "VIN"}));
        run(&mut s, VIA, json!({"at": [10, 16.19], "drill_mm": 0.4, "size_mm": 0.8, "net": "VIN"}));
        run(&mut s, ZONE, json!({"points": [[2, 2], [20, 2], [20, 10], [2, 10]], "layer": "B.Cu", "net": "GND"}));
        let info = run(&mut s, INFO, json!({}));
        assert_eq!(info["exists"], json!(true));
        assert_eq!(info["name"], json!("Main"));
        let nets = info["nets"].as_array().unwrap();
        assert!(nets.iter().any(|n| n["name"] == json!("VIN") && n["pads"][0] == json!("U1.1")), "{nets:?}");
        assert!(nets.iter().any(|n| n["name"] == json!("GND") && n["zones"] == json!(1)), "{nets:?}");
    }

    #[test]
    fn commands_before_create_say_so() {
        let mut s = Session::default();
        let e = s.execute(PLACE, &json!({"kind": "r_0603", "at": [0, 0]})).unwrap_err();
        assert!(format!("{e}").contains("pcb.create"), "{e}");
        assert_eq!(run(&mut s, INFO, json!({}))["exists"], json!(false));
    }

    #[test]
    fn a_bad_footprint_or_point_is_refused() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({}));
        let e = s.execute(PLACE, &json!({"kind": "flux_package", "at": [0, 0]})).unwrap_err();
        assert!(format!("{e}").contains("unknown footprint"), "{e}");
        let e = s.execute(TRACK, &json!({"points": [[0, 0]]})).unwrap_err();
        assert!(format!("{e}").contains("at least two"), "{e}");
        let e = s.execute(VIA, &json!({"at": [0, 0], "drill_mm": 1.0, "size_mm": 0.5})).unwrap_err();
        assert!(format!("{e}").contains("at least the drill"), "{e}");
    }

    #[test]
    fn a_saved_board_opens_again_and_exports_svg() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({"name": "Main"}));
        run(&mut s, PLACE, json!({"kind": "header_2", "at": [10, 10], "reference": "J1", "value": "PWR"}));
        let dir = std::env::temp_dir().join(format!("solidcraft-pcb-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("main.solidcraft-pcb");
        run(&mut s, SAVE, json!({"path": path.to_string_lossy()}));
        let mut t = Session::default();
        let r = run(&mut t, OPEN, json!({"path": path.to_string_lossy()}));
        assert_eq!(r["footprints"], json!(1));
        assert_eq!(r["name"], json!("Main"));
        let svg = dir.join("main.svg");
        run(&mut t, SVG, json!({"path": svg.to_string_lossy()}));
        let text = std::fs::read_to_string(&svg).unwrap();
        assert!(text.starts_with("<svg"), "{text}");
        assert!(text.contains("J1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_footprint_moves_and_is_deleted() {
        let mut s = Session::default();
        run(&mut s, CREATE, json!({}));
        let r = run(&mut s, PLACE, json!({"kind": "r_0603", "at": [20, 20]}));
        let id = r["id"].as_u64().unwrap();
        run(&mut s, MOVE_FOOTPRINT, json!({"id": id, "at": [30, 30], "rotation": 90}));
        let info = run(&mut s, INFO, json!({}));
        assert_eq!(info["footprints"][0]["at"], json!([30.0, 30.0]));
        assert_eq!(info["footprints"][0]["rotation_deg"], json!(90.0));
        run(&mut s, DELETE_FOOTPRINT, json!({"id": id}));
        assert_eq!(run(&mut s, INFO, json!({}))["footprints"].as_array().unwrap().len(), 0);
    }
}
