//! The schematic of an electronic design: symbols placed on a sheet, wires between their pins,
//! net labels, and the netlist they make.
//!
//! A schematic is a document of its own (saved as a `.solidcraft-sch` JSON file), separate from
//! the 3D design: schematic capture, board layout and mechanical design are separate steps, and
//! this crate holds the first. Units are millimetres; a symbol's pins are in the symbol's own
//! frame and are placed with its rotation.
//!
//! A file read back is untrusted input: [`Schematic::validate`] checks every number, count and
//! name before a schematic replaces a session's.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub mod pcb;
pub use pcb::{
    FOOTPRINTS, Footprint, FootprintDef, Layer, MAX_PCB_BYTES, MAX_PCB_ITEMS, MAX_POLYGON_POINTS, MAX_TRACK_POINTS, NetSummary, Pad, PadDef, Pcb,
    Track, Via, Zone, footprint_def,
};

/// Largest schematic file `schematic.open` reads.
pub const MAX_SCHEMATIC_BYTES: u64 = 16 * 1024 * 1024;
/// Most symbols, wires or labels one schematic holds.
pub const MAX_ITEMS: usize = 100_000;
/// Most points one wire holds.
pub const MAX_WIRE_POINTS: usize = 10_000;
/// Coordinates stay inside this many millimetres of the sheet's origin.
pub const MAX_COORD_MM: f64 = 1.0e6;

/// A schematic sheet: its name and size, and the symbols, wires and labels on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schematic {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_width")]
    pub width_mm: f64,
    #[serde(default = "default_height")]
    pub height_mm: f64,
    /// The next identity handed to a symbol, wire or label.
    #[serde(default = "default_next_id")]
    pub next_id: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<Symbol>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wires: Vec<Wire>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<Label>,
}

impl Default for Schematic {
    fn default() -> Self {
        Self {
            name: default_name(),
            width_mm: default_width(),
            height_mm: default_height(),
            next_id: default_next_id(),
            symbols: Vec::new(),
            wires: Vec::new(),
            labels: Vec::new(),
        }
    }
}

fn default_name() -> String {
    "Schematic".to_string()
}
fn default_width() -> f64 {
    297.0
}
fn default_height() -> f64 {
    210.0
}
fn default_next_id() -> u64 {
    1
}

/// A symbol placed on the sheet. Its pins are stored in its own frame and placed with its
/// rotation ([`Symbol::pin_world`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: u64,
    pub kind: String,
    pub reference: String,
    #[serde(default)]
    pub value: String,
    pub at: [f64; 2],
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pins: Vec<Pin>,
}

/// One pin of a placed symbol, in the symbol's own frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pin {
    pub number: String,
    pub name: String,
    pub offset: [f64; 2],
}

/// A wire: a polyline of points in sheet millimetres.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wire {
    pub id: u64,
    pub points: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// A net label at a point: it names the net that point belongs to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub net: String,
    pub at: [f64; 2],
}

/// One net of the netlist.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Net {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pins: Vec<NetPin>,
}

/// A pin on a net.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetPin {
    pub symbol: u64,
    pub reference: String,
    pub pin: String,
    pub name: String,
}

/// A symbol the library knows: its pins in the symbol's own frame.
#[derive(Clone, Copy, Debug)]
pub struct SymbolDef {
    pub kind: &'static str,
    pub label: &'static str,
    /// The reference prefix (`R1`, `C1`, `U1`…).
    pub prefix: &'static str,
    pub pins: &'static [PinDef],
}

/// One pin of a library symbol, in the symbol's own frame (millimetres).
#[derive(Clone, Copy, Debug)]
pub struct PinDef {
    pub number: &'static str,
    pub name: &'static str,
    pub x: f64,
    pub y: f64,
}

/// The symbols this library offers.
pub const SYMBOLS: &[SymbolDef] = &[
    SymbolDef {
        kind: "resistor",
        label: "Resistor",
        prefix: "R",
        pins: &[PinDef { number: "1", name: "1", x: -3.81, y: 0.0 }, PinDef { number: "2", name: "2", x: 3.81, y: 0.0 }],
    },
    SymbolDef {
        kind: "capacitor",
        label: "Capacitor",
        prefix: "C",
        pins: &[PinDef { number: "1", name: "1", x: -2.54, y: 0.0 }, PinDef { number: "2", name: "2", x: 2.54, y: 0.0 }],
    },
    SymbolDef {
        kind: "inductor",
        label: "Inductor",
        prefix: "L",
        pins: &[PinDef { number: "1", name: "1", x: -3.81, y: 0.0 }, PinDef { number: "2", name: "2", x: 3.81, y: 0.0 }],
    },
    SymbolDef {
        kind: "diode",
        label: "Diode",
        prefix: "D",
        pins: &[PinDef { number: "1", name: "A", x: -2.54, y: 0.0 }, PinDef { number: "2", name: "K", x: 2.54, y: 0.0 }],
    },
    SymbolDef {
        kind: "led",
        label: "LED",
        prefix: "D",
        pins: &[PinDef { number: "1", name: "A", x: -2.54, y: 0.0 }, PinDef { number: "2", name: "K", x: 2.54, y: 0.0 }],
    },
    SymbolDef {
        kind: "ic8",
        label: "8-pin IC",
        prefix: "U",
        pins: &[
            PinDef { number: "1", name: "1", x: -3.81, y: -3.81 },
            PinDef { number: "2", name: "2", x: -1.27, y: -3.81 },
            PinDef { number: "3", name: "3", x: 1.27, y: -3.81 },
            PinDef { number: "4", name: "4", x: 3.81, y: -3.81 },
            PinDef { number: "8", name: "8", x: -3.81, y: 3.81 },
            PinDef { number: "7", name: "7", x: -1.27, y: 3.81 },
            PinDef { number: "6", name: "6", x: 1.27, y: 3.81 },
            PinDef { number: "5", name: "5", x: 3.81, y: 3.81 },
        ],
    },
    SymbolDef { kind: "ground", label: "Ground", prefix: "GND", pins: &[PinDef { number: "1", name: "GND", x: 0.0, y: 0.0 }] },
    SymbolDef { kind: "vcc", label: "Supply", prefix: "VCC", pins: &[PinDef { number: "1", name: "VCC", x: 0.0, y: 0.0 }] },
    SymbolDef {
        kind: "connector2",
        label: "2-pin connector",
        prefix: "J",
        pins: &[PinDef { number: "1", name: "1", x: -2.54, y: 0.0 }, PinDef { number: "2", name: "2", x: 2.54, y: 0.0 }],
    },
];

/// The library symbol `kind` names (trimmed, case-insensitive), if there is one.
pub fn symbol_def(kind: &str) -> Option<&'static SymbolDef> {
    let kind = kind.trim().to_ascii_lowercase();
    SYMBOLS.iter().find(|d| d.kind == kind)
}

/// A finite point inside the coordinate range.
fn finite(p: [f64; 2]) -> bool {
    p[0].is_finite() && p[1].is_finite() && p[0].abs() <= MAX_COORD_MM && p[1].abs() <= MAX_COORD_MM
}

/// A sheet side in millimetres.
fn finite_size(x: f64) -> bool {
    x.is_finite() && (1.0..=10_000.0).contains(&x)
}

impl Symbol {
    /// `pin`'s point on the sheet: the symbol's rotation turns its own frame, then `at` places it.
    pub fn pin_world(&self, pin: &Pin) -> [f64; 2] {
        let (sin, cos) = self.rotation_deg.to_radians().sin_cos();
        let [x, y] = pin.offset;
        [self.at[0] + x * cos - y * sin, self.at[1] + x * sin + y * cos]
    }
}

impl Schematic {
    fn next_id(&mut self) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        id
    }

    /// Reference `prefix` + the smallest number not used yet (`R1`, `R2`…).
    fn next_reference(&self, prefix: &str) -> String {
        for n in 1..=1_000_000u64 {
            let r = format!("{prefix}{n}");
            if !self.symbols.iter().any(|s| s.reference == r) {
                return r;
            }
        }
        format!("{prefix}?")
    }

    /// Place a library symbol. `reference` defaults to `R1`, `C1`, `U1`… by kind.
    pub fn place(
        &mut self,
        kind: &str,
        at: [f64; 2],
        reference: Option<&str>,
        value: Option<&str>,
        rotation_deg: f64,
    ) -> Result<(u64, String), String> {
        let def = symbol_def(kind).ok_or_else(|| {
            format!("unknown symbol kind `{}` (known: {})", kind.trim(), SYMBOLS.iter().map(|d| d.kind).collect::<Vec<_>>().join(", "))
        })?;
        if !finite(at) {
            return Err("`at` must be a point [x, y] in millimetres".into());
        }
        if !rotation_deg.is_finite() || rotation_deg.abs() > 1.0e6 {
            return Err("`rotation` must be a finite number of degrees".into());
        }
        if self.symbols.len() >= MAX_ITEMS {
            return Err(format!("a schematic holds at most {MAX_ITEMS} symbols"));
        }
        let reference = match reference.map(str::trim) {
            Some(r) if !r.is_empty() => {
                if r.chars().count() > 64 {
                    return Err("a reference holds at most 64 characters".into());
                }
                r.to_string()
            }
            _ => self.next_reference(def.prefix),
        };
        let id = self.next_id();
        let pins = def.pins.iter().map(|p| Pin { number: p.number.to_string(), name: p.name.to_string(), offset: [p.x, p.y] }).collect();
        self.symbols.push(Symbol {
            id,
            kind: def.kind.to_string(),
            reference: reference.clone(),
            value: value.unwrap_or("").trim().to_string(),
            at,
            rotation_deg,
            pins,
        });
        Ok((id, reference))
    }

    /// Add a wire from two or more points.
    pub fn add_wire(&mut self, points: &[[f64; 2]], net: Option<&str>) -> Result<u64, String> {
        if points.len() < 2 {
            return Err("a wire needs at least two points".into());
        }
        if points.len() > MAX_WIRE_POINTS {
            return Err(format!("a wire holds at most {MAX_WIRE_POINTS} points"));
        }
        if points.iter().any(|p| !finite(*p)) {
            return Err("every wire point must be two finite numbers in millimetres".into());
        }
        if self.wires.len() >= MAX_ITEMS {
            return Err(format!("a schematic holds at most {MAX_ITEMS} wires"));
        }
        let id = self.next_id();
        let net = net.map(str::trim).filter(|n| !n.is_empty()).map(|n| n.to_string());
        self.wires.push(Wire { id, points: points.to_vec(), net });
        Ok(id)
    }

    /// Add a net label at a point.
    pub fn add_label(&mut self, net: &str, at: [f64; 2]) -> Result<u64, String> {
        let net = net.trim();
        if net.is_empty() {
            return Err("a label needs a net name".into());
        }
        if net.chars().count() > 128 {
            return Err("a net name holds at most 128 characters".into());
        }
        if !finite(at) {
            return Err("`at` must be a point [x, y] in millimetres".into());
        }
        if self.labels.len() >= MAX_ITEMS {
            return Err(format!("a schematic holds at most {MAX_ITEMS} labels"));
        }
        let id = self.next_id();
        self.labels.push(Label { id, net: net.to_string(), at });
        Ok(id)
    }
}

/// A point on the 0.001 mm grid, so points a design means to share do share.
type Node = (i64, i64);

fn node(p: [f64; 2]) -> Node {
    ((p[0] * 1000.0).round() as i64, (p[1] * 1000.0).round() as i64)
}

fn find(parent: &mut BTreeMap<Node, Node>, k: Node) -> Node {
    let p = *parent.get(&k).unwrap_or(&k);
    if p == k {
        return k;
    }
    let root = find(parent, p);
    parent.insert(k, root);
    root
}

fn union(parent: &mut BTreeMap<Node, Node>, a: Node, b: Node) {
    parent.entry(a).or_insert(a);
    parent.entry(b).or_insert(b);
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra != rb {
        parent.insert(ra, rb);
    }
}

impl Schematic {
    /// The netlist: wires, pins and labels that share a point are one net; a net takes a label's
    /// name, else its wire's, else `N$<n>`.
    pub fn nets(&self) -> Vec<Net> {
        let mut parent: BTreeMap<Node, Node> = BTreeMap::new();
        for wire in &self.wires {
            for pair in wire.points.windows(2) {
                union(&mut parent, node(pair[0]), node(pair[1]));
            }
        }
        for label in &self.labels {
            parent.entry(node(label.at)).or_insert(node(label.at));
        }
        for symbol in &self.symbols {
            for pin in &symbol.pins {
                let n = node(symbol.pin_world(pin));
                parent.entry(n).or_insert(n);
            }
        }
        let mut pins: BTreeMap<Node, Vec<NetPin>> = BTreeMap::new();
        for symbol in &self.symbols {
            for pin in &symbol.pins {
                let root = find(&mut parent, node(symbol.pin_world(pin)));
                pins.entry(root).or_default().push(NetPin {
                    symbol: symbol.id,
                    reference: symbol.reference.clone(),
                    pin: pin.number.clone(),
                    name: pin.name.clone(),
                });
            }
        }
        let mut names: BTreeMap<Node, String> = BTreeMap::new();
        for label in &self.labels {
            let root = find(&mut parent, node(label.at));
            names.entry(root).or_insert_with(|| label.net.clone());
        }
        for wire in &self.wires {
            if let (Some(net), Some(first)) = (&wire.net, wire.points.first()) {
                let root = find(&mut parent, node(*first));
                names.entry(root).or_insert_with(|| net.clone());
            }
        }
        let mut roots: BTreeSet<Node> = pins.keys().copied().collect();
        for label in &self.labels {
            roots.insert(find(&mut parent, node(label.at)));
        }
        for wire in &self.wires {
            if let Some(first) = wire.points.first() {
                roots.insert(find(&mut parent, node(*first)));
            }
        }
        let mut nets = Vec::with_capacity(roots.len());
        for (i, root) in roots.into_iter().enumerate() {
            let mut net_pins = pins.remove(&root).unwrap_or_default();
            net_pins.sort_by(|a, b| (a.reference.as_str(), a.pin.as_str()).cmp(&(b.reference.as_str(), b.pin.as_str())));
            let name = names.get(&root).cloned().unwrap_or_else(|| format!("N${}", i + 1));
            nets.push(Net { name, pins: net_pins });
        }
        nets.sort_by(|a, b| a.name.cmp(&b.name));
        nets
    }
}

impl Schematic {
    /// The schematic as JSON for a command's reply.
    pub fn info(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "name": self.name,
            "width_mm": self.width_mm,
            "height_mm": self.height_mm,
            "symbols": self.symbols.iter().map(|s| json!({
                "id": s.id,
                "kind": s.kind,
                "reference": s.reference,
                "value": s.value,
                "at": s.at,
                "rotation_deg": s.rotation_deg,
                "pins": s.pins.iter().map(|p| json!({ "number": p.number, "name": p.name, "at": s.pin_world(p) })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "wires": self.wires.iter().map(|w| json!({ "id": w.id, "points": w.points, "net": w.net })).collect::<Vec<_>>(),
            "labels": self.labels.iter().map(|l| json!({ "id": l.id, "net": l.net, "at": l.at })).collect::<Vec<_>>(),
            "nets": self.nets(),
        })
    }

    /// The schematic as an SVG picture (4 px per millimetre): a quick look at what the commands
    /// built, with the sheet's lower-left corner at the picture's lower-left corner.
    pub fn to_svg(&self) -> String {
        const SCALE: f64 = 4.0;
        let w = (self.width_mm * SCALE).max(1.0);
        let h = (self.height_mm * SCALE).max(1.0);
        let x = |x: f64| x * SCALE;
        let y = |y: f64| (self.height_mm - y) * SCALE;
        let mut svg = String::new();
        svg.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}\" height=\"{h:.2}\" viewBox=\"0 0 {w:.2} {h:.2}\">\n"));
        svg.push_str(&format!("<rect width=\"{w:.2}\" height=\"{h:.2}\" fill=\"#ffffff\"/>\n"));
        let m = 10.0 * SCALE;
        svg.push_str(&format!(
            "<rect x=\"{m:.2}\" y=\"{m:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"none\" stroke=\"#333\" stroke-width=\"1\"/>\n",
            (w - 2.0 * m).max(1.0),
            (h - 2.0 * m).max(1.0)
        ));
        svg.push_str(&format!(
            "<text x=\"{m:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"18\" fill=\"#111\">{}</text>\n",
            m - 6.0,
            escape(&self.name)
        ));
        for wire in &self.wires {
            let pts: Vec<String> = wire.points.iter().map(|p| format!("{:.2},{:.2}", x(p[0]), y(p[1]))).collect();
            svg.push_str(&format!("<polyline points=\"{}\" fill=\"none\" stroke=\"#0a7d32\" stroke-width=\"2\"/>\n", pts.join(" ")));
        }
        for symbol in &self.symbols {
            let world: Vec<[f64; 2]> = symbol.pins.iter().map(|p| symbol.pin_world(p)).collect();
            let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
            let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
            for p in &world {
                min_x = min_x.min(p[0]);
                min_y = min_y.min(p[1]);
                max_x = max_x.max(p[0]);
                max_y = max_y.max(p[1]);
            }
            if !min_x.is_finite() {
                min_x = symbol.at[0];
                max_x = symbol.at[0];
                min_y = symbol.at[1];
                max_y = symbol.at[1];
            }
            let pad = 3.0;
            let (bx, by) = (x(min_x - pad), y(max_y + pad));
            let (bw, bh) = ((max_x - min_x + 2.0 * pad) * SCALE, (max_y - min_y + 2.0 * pad) * SCALE);
            svg.push_str(&format!(
                "<rect x=\"{bx:.2}\" y=\"{by:.2}\" width=\"{bw:.2}\" height=\"{bh:.2}\" rx=\"3\" fill=\"#f4e7c5\" stroke=\"#8a6d1a\" stroke-width=\"1.2\"/>\n"
            ));
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"12\" fill=\"#222\">{}</text>\n",
                x(symbol.at[0]),
                y(max_y + pad + 1.0),
                escape(&symbol.reference)
            ));
            if !symbol.value.is_empty() {
                svg.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#555\">{}</text>\n",
                    x(symbol.at[0]),
                    y(min_y - pad - 1.0),
                    escape(&symbol.value)
                ));
            }
            for p in &world {
                svg.push_str(&format!("<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"2.2\" fill=\"#b8860b\"/>\n", x(p[0]), y(p[1])));
            }
        }
        for label in &self.labels {
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#1a4fbf\">{}</text>\n",
                x(label.at[0]) + 3.0,
                y(label.at[1]) - 3.0,
                escape(&label.net)
            ));
        }
        svg.push_str("</svg>\n");
        svg
    }

    /// Check a schematic read from a file before it replaces a session's.
    pub fn validate(&self) -> Result<(), String> {
        if !finite_size(self.width_mm) || !finite_size(self.height_mm) {
            return Err("the sheet must be 1 to 10000 mm on each side".into());
        }
        if self.name.trim().is_empty() || self.name.chars().count() > 128 {
            return Err("the sheet's name holds 1 to 128 characters".into());
        }
        if self.symbols.len() > MAX_ITEMS || self.wires.len() > MAX_ITEMS || self.labels.len() > MAX_ITEMS {
            return Err(format!("a schematic holds at most {MAX_ITEMS} symbols, wires and labels"));
        }
        for symbol in &self.symbols {
            if !finite(symbol.at) {
                return Err(format!("symbol `{}` has a point outside the sheet's range", symbol.reference));
            }
            if !symbol.rotation_deg.is_finite() || symbol.rotation_deg.abs() > 1.0e6 {
                return Err(format!("symbol `{}` has an invalid rotation", symbol.reference));
            }
            if symbol.reference.trim().is_empty() || symbol.reference.chars().count() > 64 {
                return Err("a reference holds 1 to 64 characters".into());
            }
            if symbol.pins.len() > 1024 {
                return Err(format!("symbol `{}` has too many pins", symbol.reference));
            }
            for pin in &symbol.pins {
                if !finite(pin.offset) {
                    return Err(format!("symbol `{}` has a pin outside the sheet's range", symbol.reference));
                }
            }
        }
        for wire in &self.wires {
            if wire.points.len() < 2 || wire.points.len() > MAX_WIRE_POINTS {
                return Err(format!("a wire holds 2 to {MAX_WIRE_POINTS} points"));
            }
            if wire.points.iter().any(|p| !finite(*p)) {
                return Err("every wire point must be two finite numbers in millimetres".into());
            }
            if wire.net.as_ref().is_some_and(|n| n.chars().count() > 128) {
                return Err("a net name holds at most 128 characters".into());
            }
        }
        for label in &self.labels {
            if !finite(label.at) {
                return Err("a label's point is outside the sheet's range".into());
            }
            if label.net.trim().is_empty() || label.net.chars().count() > 128 {
                return Err("a net name holds 1 to 128 characters".into());
            }
        }
        Ok(())
    }

    /// Serialize for saving.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Read from a file's bytes.
    pub fn from_json(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}

/// Text as SVG/XML content: the five characters XML reserves.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn a_resistor_and_labels_make_two_nets() {
        let mut sch = Schematic::default();
        let (_, reference) = sch.place("resistor", [50.0, 50.0], None, Some("10k"), 0.0).unwrap();
        assert_eq!(reference, "R1");
        sch.add_wire(&[[46.19, 50.0], [43.0, 50.0]], None).unwrap();
        sch.add_label("VCC", [43.0, 50.0]).unwrap();
        sch.add_wire(&[[53.81, 50.0], [60.0, 50.0]], None).unwrap();
        sch.add_label("GND", [60.0, 50.0]).unwrap();
        let nets = sch.nets();
        assert_eq!(nets.len(), 2, "{nets:?}");
        assert!(nets.iter().any(|n| n.name == "VCC" && n.pins.iter().any(|p| p.reference == "R1" && p.pin == "1")));
        assert!(nets.iter().any(|n| n.name == "GND" && n.pins.iter().any(|p| p.reference == "R1" && p.pin == "2")));
    }

    #[test]
    fn a_wire_through_a_shared_point_joins_symbols() {
        let mut sch = Schematic::default();
        sch.place("resistor", [20.0, 20.0], None, None, 0.0).unwrap();
        sch.place("capacitor", [40.0, 20.0], None, None, 0.0).unwrap();
        // R1 pin 2 is at (23.81, 20); C1 pin 1 is at (37.46, 20).
        sch.add_wire(&[[23.81, 20.0], [30.0, 20.0]], None).unwrap();
        sch.add_wire(&[[30.0, 20.0], [37.46, 20.0]], None).unwrap();
        let nets = sch.nets();
        let net = nets.iter().find(|n| n.pins.len() == 2).expect("a shared net");
        let references: Vec<&str> = net.pins.iter().map(|p| p.reference.as_str()).collect();
        assert_eq!(references, ["C1", "R1"]);
    }

    #[test]
    fn unknown_kinds_and_bad_points_are_refused() {
        let mut sch = Schematic::default();
        assert!(sch.place("flux-capacitor", [0.0, 0.0], None, None, 0.0).is_err());
        assert!(sch.place("resistor", [f64::NAN, 0.0], None, None, 0.0).is_err());
        assert!(sch.add_wire(&[[0.0, 0.0]], None).is_err());
        assert!(sch.add_label("", [0.0, 0.0]).is_err());
    }

    #[test]
    fn a_placed_symbol_rotates_its_pins() {
        let mut sch = Schematic::default();
        let (id, _) = sch.place("resistor", [50.0, 50.0], None, None, 90.0).unwrap();
        let symbol = sch.symbols.iter().find(|s| s.id == id).unwrap();
        let at = symbol.pin_world(&symbol.pins[0]);
        assert!((at[0] - 50.0).abs() < 1e-9, "{at:?}");
        assert!((at[1] - 46.19).abs() < 1e-9, "{at:?}");
    }

    #[test]
    fn a_saved_schematic_reads_back() {
        let mut sch = Schematic::default();
        sch.place("ic8", [100.0, 100.0], Some("U1"), Some("NE555"), 0.0).unwrap();
        sch.add_label("GND", [96.19, 96.19]).unwrap();
        let text = sch.to_json().unwrap();
        let back = Schematic::from_json(text.as_bytes()).unwrap();
        assert_eq!(back, sch);
        assert!(back.validate().is_ok());
    }

    #[test]
    fn the_schematic_draws_itself_as_svg() {
        let mut sch = Schematic::default();
        sch.place("resistor", [50.0, 50.0], Some("R1"), Some("10k"), 0.0).unwrap();
        sch.add_wire(&[[46.19, 50.0], [40.0, 50.0]], None).unwrap();
        sch.add_label("VCC", [40.0, 50.0]).unwrap();
        let svg = sch.to_svg();
        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.contains("polyline"));
        assert!(svg.contains("VCC"));
        assert!(svg.contains("R1"));
    }
}
