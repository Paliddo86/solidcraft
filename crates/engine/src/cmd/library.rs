//! The component library commands: list and enable devices, add one, and place a device's symbol
//! or footprint. The library model is `solidcraft_ecad::library`.

use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{bad, num, str_};
use crate::{Result, Session};

const LIST: &str = "library.list";
const ENABLE: &str = "library.enable";
const ADD: &str = "library.add_device";
const PLACE: &str = "component.place";
const PCB_PLACE: &str = "pcb.component.place";
const MANAGER: &str = "library.manager";
const REPLACE: &str = "library.replace";

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new(MANAGER, "Component Manager", manager).noundo().doc_only().at("SCHEMATIC", "LIBRARY").icon("select_components"),
    CommandSpec::new(LIST, "Components", list)
        .noundo()
        .doc_only()
        .params("→ devices [{id, name, symbol, footprint, model, category, value, in_design, in_use}], count"),
    CommandSpec::new(ENABLE, "Enable Component", enable)
        .noundo()
        .doc_only()
        .params("id: the device's id; on: true to use it in the design, false to hide it → id, in_use"),
    CommandSpec::new(ADD, "Add Component", add).noundo().doc_only().params(
        "id: 1 to 64 letters, digits, _ or -; name; symbol: a library symbol kind; footprint?: a library footprint kind; model?: a STEP/STL/OBJ/GLB path; category?; value? → id, devices",
    ),
    CommandSpec::new(PLACE, "Place Component", place).noundo().doc_only().params(
        "device: a library device id; at: [x, y] in mm; rotation?: degrees; reference?: the reference; value?: its value → id, reference, device, symbol, footprint, at",
    ),
    CommandSpec::new(PCB_PLACE, "Place Component", pcb_place).noundo().doc_only().params(
        "device: a library device id; at: [x, y] in mm; rotation?: degrees; reference?: the reference; value?: its value; layer?: F.Cu | B.Cu → id, reference, device, footprint, at",
    ),
    CommandSpec::new(REPLACE, "Replace Component", replace).noundo().doc_only().params(
        "reference: the component's reference; device: a library device id → reference, device, replaced",
    ),
];

/// UI-only: the Electronics workspace toggles the Component Manager window.
fn manager(_s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({}))
}

fn list(s: &mut Session, _p: &Value) -> Result<Value> {
    let devices: Vec<Value> = s
        .library
        .devices
        .iter()
        .map(|d| {
            let in_design = s.library.in_design(d, s.schematic.as_ref(), s.pcb.as_ref());
            json!({
                "id": d.id, "name": d.name, "symbol": d.symbol, "footprint": d.footprint,
                "model": d.model, "category": d.category, "value": d.value,
                "in_design": in_design, "in_use": s.library.in_use.contains(&d.id),
            })
        })
        .collect();
    Ok(json!({ "devices": devices, "count": devices.len() }))
}

fn enable(s: &mut Session, p: &Value) -> Result<Value> {
    let id = str_(p, "id").ok_or_else(|| bad(ENABLE, "`id` is the device's id"))?;
    let on = p.get("on").and_then(Value::as_bool).ok_or_else(|| bad(ENABLE, "`on` must be true or false"))?;
    s.library.enable(id, on).map_err(|why| bad(ENABLE, why))?;
    Ok(json!({ "id": id, "in_use": on }))
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    let id = str_(p, "id").ok_or_else(|| bad(ADD, "`id` is the device's id"))?;
    let name = str_(p, "name").ok_or_else(|| bad(ADD, "`name` is the device's name"))?;
    let symbol = str_(p, "symbol").ok_or_else(|| bad(ADD, "`symbol` is a library symbol kind"))?;
    if solidcraft_ecad::SYMBOLS.iter().all(|s| s.kind != symbol) {
        return Err(bad(ADD, format!("no symbol kind `{symbol}`")));
    }
    let footprint = str_(p, "footprint");
    if let Some(f) = footprint
        && solidcraft_ecad::FOOTPRINTS.iter().all(|x| x.kind != f)
    {
        return Err(bad(ADD, format!("no footprint kind `{f}`")));
    }
    let model = str_(p, "model");
    if let Some(m) = model
        && (m.is_empty() || m.chars().any(char::is_control))
    {
        return Err(bad(ADD, "`model` must be a path without control characters"));
    }
    let device = solidcraft_ecad::Device {
        id: id.to_string(),
        name: name.to_string(),
        symbol: symbol.to_string(),
        footprint: footprint.map(str::to_string),
        model: model.map(str::to_string),
        category: str_(p, "category").unwrap_or("").to_string(),
        value: str_(p, "value").unwrap_or("").to_string(),
    };
    s.library.add(device).map_err(|why| bad(ADD, why))?;
    Ok(json!({ "id": id, "devices": s.library.devices.len() }))
}

fn place(s: &mut Session, p: &Value) -> Result<Value> {
    let (device, at, rotation, reference, value) = place_params(s, p, PLACE)?;
    let sch = s.schematic.as_mut().ok_or_else(|| bad(PLACE, "no schematic yet: run schematic.create first"))?;
    let value = value.unwrap_or(device.value.as_str());
    let (id, reference) = sch.place(&device.symbol, at, reference, Some(value), rotation).map_err(|why| bad(PLACE, why))?;
    Ok(json!({ "id": id, "reference": reference, "device": device.id, "symbol": device.symbol, "footprint": device.footprint, "at": at }))
}

fn pcb_place(s: &mut Session, p: &Value) -> Result<Value> {
    let (device, at, rotation, reference, value) = place_params(s, p, PCB_PLACE)?;
    let footprint = device.footprint.as_deref().ok_or_else(|| bad(PCB_PLACE, format!("the device `{}` has no footprint", device.id)))?;
    let layer = str_(p, "layer").unwrap_or("F.Cu");
    let board = s.pcb.as_mut().ok_or_else(|| bad(PCB_PLACE, "no board yet: run pcb.create first"))?;
    let value = value.unwrap_or(device.value.as_str());
    let (id, reference) = board.place_footprint(footprint, at, reference, Some(value), rotation, layer).map_err(|why| bad(PCB_PLACE, why))?;
    Ok(json!({ "id": id, "reference": reference, "device": device.id, "footprint": footprint, "at": at }))
}

fn replace(s: &mut Session, p: &Value) -> Result<Value> {
    let reference = str_(p, "reference").ok_or_else(|| bad(REPLACE, "`reference` is the component's reference"))?;
    let device_id = str_(p, "device").ok_or_else(|| bad(REPLACE, "`device` is a library device id"))?;
    let device = s.library.device(device_id).cloned().ok_or_else(|| bad(REPLACE, format!("no device `{device_id}`")))?;
    let symbol_def = solidcraft_ecad::SYMBOLS
        .iter()
        .find(|d| d.kind == device.symbol)
        .ok_or_else(|| bad(REPLACE, format!("no symbol kind `{}`", device.symbol)))?;
    let pins: Vec<solidcraft_ecad::Pin> =
        symbol_def.pins.iter().map(|p| solidcraft_ecad::Pin { number: p.number.to_string(), name: p.name.to_string(), offset: [p.x, p.y] }).collect();
    let mut replaced = 0;
    if let Some(sch) = s.schematic.as_mut()
        && let Some(symbol) = sch.symbols.iter_mut().find(|x| x.reference == reference)
    {
        symbol.kind = device.symbol.clone();
        symbol.pins = pins;
        replaced += 1;
    }
    if let Some(footprint_kind) = device.footprint.as_deref() {
        let def = solidcraft_ecad::FOOTPRINTS
            .iter()
            .find(|d| d.kind == footprint_kind)
            .ok_or_else(|| bad(REPLACE, format!("no footprint kind `{footprint_kind}`")))?;
        if let Some(board) = s.pcb.as_mut()
            && let Some(footprint) = board.footprints.iter_mut().find(|x| x.reference == reference)
        {
            let nets: std::collections::BTreeMap<String, Option<String>> = footprint.pads.iter().map(|p| (p.number.clone(), p.net.clone())).collect();
            footprint.kind = footprint_kind.to_string();
            footprint.pads = def
                .pads
                .iter()
                .map(|p| solidcraft_ecad::Pad {
                    number: p.number.to_string(),
                    at: [p.x, p.y],
                    size: [p.w, p.h],
                    shape: p.shape.to_string(),
                    drill_mm: p.drill,
                    net: nets.get(p.number).cloned().flatten(),
                })
                .collect();
            replaced += 1;
        }
    }
    if replaced == 0 {
        return Err(bad(REPLACE, format!("no component `{reference}`")));
    }
    Ok(json!({ "reference": reference, "device": device.id, "replaced": replaced }))
}

fn place_params<'p>(s: &Session, p: &'p Value, cmd: &str) -> Result<(solidcraft_ecad::Device, [f64; 2], f64, Option<&'p str>, Option<&'p str>)> {
    let id = str_(p, "device").ok_or_else(|| bad(cmd, "`device` is a library device id"))?;
    let device = s.library.device(id).cloned().ok_or_else(|| bad(cmd, format!("no device `{id}`")))?;
    let at = point(p, "at").ok_or_else(|| bad(cmd, "`at` must be a point [x, y] in millimetres"))?;
    let rotation = match p.get("rotation") {
        None | Some(Value::Null) => 0.0,
        Some(_) => num(p, "rotation").ok_or_else(|| bad(cmd, "`rotation` must be a finite number of degrees"))?,
    };
    Ok((device, at, rotation, str_(p, "reference"), str_(p, "value")))
}

fn point(p: &Value, k: &str) -> Option<[f64; 2]> {
    let a = p.get(k)?.as_array()?;
    if a.len() < 2 {
        return None;
    }
    let x = a.first()?.as_f64()?;
    let y = a.get(1)?.as_f64()?;
    (x.is_finite() && y.is_finite()).then_some([x, y])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    fn run(s: &mut Session, id: &str, params: Value) -> Value {
        s.execute(id, &params).unwrap_or_else(|why| panic!("{id}: {why}"))
    }

    #[test]
    fn a_component_is_replaced_from_the_library() {
        let mut s = Session::default();
        run(&mut s, "schematic.create", json!({}));
        run(&mut s, "pcb.create", json!({}));
        run(&mut s, "component.place", json!({"device": "resistor", "at": [50, 50], "reference": "R7"}));
        run(&mut s, "pcb.component.place", json!({"device": "resistor", "at": [30, 30], "reference": "R7"}));
        let r = run(&mut s, "library.replace", json!({"reference": "R7", "device": "ic8"}));
        assert_eq!(r["replaced"], json!(2));
        assert!(s.schematic.as_ref().unwrap().symbols.iter().any(|x| x.reference == "R7" && x.kind == "ic8" && x.pins.len() == 8));
        assert!(s.pcb.as_ref().unwrap().footprints.iter().any(|x| x.reference == "R7" && x.kind == "ic8_dip"));
    }
}
