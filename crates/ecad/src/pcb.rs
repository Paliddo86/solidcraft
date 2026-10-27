//! The printed-circuit board of an electronic design: its outline and thickness, layers,
//! footprints (pads), tracks, vias and copper zones, and the nets they name.
//!
//! A board is a document of its own, separate from the schematic (which is its netlist's source)
//! and from the 3D design. Units are millimetres; a footprint's pads are in its own frame and
//! are placed with its rotation.

use serde::{Deserialize, Serialize};

/// Largest board file `pcb.open` reads.
pub const MAX_PCB_BYTES: u64 = 64 * 1024 * 1024;
/// Most footprints, tracks, vias or zones one board holds.
pub const MAX_PCB_ITEMS: usize = 200_000;
/// Most points one track holds.
pub const MAX_TRACK_POINTS: usize = 10_000;
/// Most points one zone polygon holds.
pub const MAX_POLYGON_POINTS: usize = 50_000;
/// Coordinates stay inside this many millimetres of the board's origin.
pub const MAX_COORD_MM: f64 = 1.0e6;

/// A board: its outline and stackup, and what is placed and routed on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pcb {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_width")]
    pub width_mm: f64,
    #[serde(default = "default_height")]
    pub height_mm: f64,
    #[serde(default = "default_thickness")]
    pub thickness_mm: f64,
    /// The next identity handed to a footprint, track, via or zone.
    #[serde(default = "default_next_id")]
    pub next_id: u64,
    /// The stackup, top to bottom.
    #[serde(default = "default_layers", skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footprints: Vec<Footprint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tracks: Vec<Track>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vias: Vec<Via>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zones: Vec<Zone>,
}

impl Default for Pcb {
    fn default() -> Self {
        Self {
            name: default_name(),
            width_mm: default_width(),
            height_mm: default_height(),
            thickness_mm: default_thickness(),
            next_id: default_next_id(),
            layers: default_layers(),
            footprints: Vec::new(),
            tracks: Vec::new(),
            vias: Vec::new(),
            zones: Vec::new(),
        }
    }
}

fn default_name() -> String {
    "PCB".to_string()
}
fn default_width() -> f64 {
    80.0
}
fn default_height() -> f64 {
    50.0
}
fn default_thickness() -> f64 {
    1.6
}
fn default_next_id() -> u64 {
    1
}

/// One layer of the stackup. `kind` is `copper`, `silk`, `mask` or `mechanical`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub thickness_um: f64,
}

fn default_layers() -> Vec<Layer> {
    [("F.Cu", "copper", 35.0), ("B.Cu", "copper", 35.0), ("F.SilkS", "silk", 0.0), ("B.SilkS", "silk", 0.0), ("Edge.Cuts", "mechanical", 0.0)]
        .into_iter()
        .map(|(name, kind, thickness_um)| Layer { name: name.to_string(), kind: kind.to_string(), thickness_um })
        .collect()
}

/// A footprint placed on the board. Its pads are stored in its own frame and placed with its
/// rotation ([`Footprint::pad_world`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Footprint {
    pub id: u64,
    pub kind: String,
    pub reference: String,
    #[serde(default)]
    pub value: String,
    pub at: [f64; 2],
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default = "front")]
    pub layer: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pads: Vec<Pad>,
}

fn front() -> String {
    "F.Cu".to_string()
}
fn back() -> String {
    "B.Cu".to_string()
}
fn rect() -> String {
    "rect".to_string()
}

/// One pad of a footprint, in the footprint's own frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pad {
    pub number: String,
    pub at: [f64; 2],
    pub size: [f64; 2],
    #[serde(default = "rect")]
    pub shape: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drill_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// A track: a polyline of copper on one layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: u64,
    pub points: Vec<[f64; 2]>,
    pub width_mm: f64,
    #[serde(default = "front")]
    pub layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// A via: a plated hole joining two copper layers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Via {
    pub id: u64,
    pub at: [f64; 2],
    pub drill_mm: f64,
    pub size_mm: f64,
    #[serde(default = "front")]
    pub from: String,
    #[serde(default = "back")]
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// A copper zone: a filled polygon on one layer, usually connected to a net.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: u64,
    pub polygon: Vec<[f64; 2]>,
    #[serde(default = "front")]
    pub layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// A footprint the library knows: its pads in the footprint's own frame.
#[derive(Clone, Copy, Debug)]
pub struct FootprintDef {
    pub kind: &'static str,
    pub label: &'static str,
    pub pads: &'static [PadDef],
}

/// One pad of a library footprint.
#[derive(Clone, Copy, Debug)]
pub struct PadDef {
    pub number: &'static str,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub shape: &'static str,
    pub drill: Option<f64>,
}

/// The footprints this library offers.
pub const FOOTPRINTS: &[FootprintDef] = &[
    FootprintDef {
        kind: "r_0603",
        label: "Resistor 0603",
        pads: &[
            PadDef { number: "1", x: -0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
            PadDef { number: "2", x: 0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
        ],
    },
    FootprintDef {
        kind: "c_0603",
        label: "Capacitor 0603",
        pads: &[
            PadDef { number: "1", x: -0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
            PadDef { number: "2", x: 0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
        ],
    },
    FootprintDef {
        kind: "led_0603",
        label: "LED 0603",
        pads: &[
            PadDef { number: "1", x: -0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
            PadDef { number: "2", x: 0.75, y: 0.0, w: 0.8, h: 0.9, shape: "rect", drill: None },
        ],
    },
    FootprintDef {
        kind: "ic8_dip",
        label: "8-pin DIP",
        pads: &[
            PadDef { number: "1", x: -3.81, y: -3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "2", x: -1.27, y: -3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "3", x: 1.27, y: -3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "4", x: 3.81, y: -3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "5", x: 3.81, y: 3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "6", x: 1.27, y: 3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "7", x: -1.27, y: 3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
            PadDef { number: "8", x: -3.81, y: 3.81, w: 1.6, h: 1.6, shape: "round", drill: Some(0.8) },
        ],
    },
    FootprintDef {
        kind: "header_2",
        label: "2-pin header",
        pads: &[
            PadDef { number: "1", x: -1.27, y: 0.0, w: 1.8, h: 1.8, shape: "round", drill: Some(1.0) },
            PadDef { number: "2", x: 1.27, y: 0.0, w: 1.8, h: 1.8, shape: "round", drill: Some(1.0) },
        ],
    },
    FootprintDef {
        kind: "mounting_hole",
        label: "Mounting hole",
        pads: &[PadDef { number: "", x: 0.0, y: 0.0, w: 6.0, h: 6.0, shape: "round", drill: Some(3.2) }],
    },
];

/// The library footprint `kind` names (trimmed, case-insensitive), if there is one.
pub fn footprint_def(kind: &str) -> Option<&'static FootprintDef> {
    let kind = kind.trim().to_ascii_lowercase();
    FOOTPRINTS.iter().find(|d| d.kind == kind)
}

/// A finite point inside the coordinate range.
fn finite(p: [f64; 2]) -> bool {
    p[0].is_finite() && p[1].is_finite() && p[0].abs() <= MAX_COORD_MM && p[1].abs() <= MAX_COORD_MM
}

impl Footprint {
    /// `pad`'s point on the board: the footprint's rotation turns its own frame, then `at` places it.
    pub fn pad_world(&self, pad: &Pad) -> [f64; 2] {
        let (sin, cos) = self.rotation_deg.to_radians().sin_cos();
        let [x, y] = pad.at;
        [self.at[0] + x * cos - y * sin, self.at[1] + x * sin + y * cos]
    }
}

impl Pcb {
    fn next_id(&mut self) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        id
    }

    /// Place a library footprint. `reference` defaults to `R1`, `C1`, `U1`… by kind.
    pub fn place_footprint(
        &mut self,
        kind: &str,
        at: [f64; 2],
        reference: Option<&str>,
        value: Option<&str>,
        rotation_deg: f64,
        layer: &str,
    ) -> Result<(u64, String), String> {
        let def = footprint_def(kind).ok_or_else(|| {
            format!("unknown footprint kind `{}` (known: {})", kind.trim(), FOOTPRINTS.iter().map(|d| d.kind).collect::<Vec<_>>().join(", "))
        })?;
        if !finite(at) {
            return Err("`at` must be a point [x, y] in millimetres".into());
        }
        if !rotation_deg.is_finite() || rotation_deg.abs() > 1.0e6 {
            return Err("`rotation` must be a finite number of degrees".into());
        }
        if self.footprints.len() >= MAX_PCB_ITEMS {
            return Err(format!("a board holds at most {MAX_PCB_ITEMS} footprints"));
        }
        let reference = match reference.map(str::trim) {
            Some(r) if !r.is_empty() => {
                if r.chars().count() > 64 {
                    return Err("a reference holds at most 64 characters".into());
                }
                r.to_string()
            }
            _ => self.next_reference(def.kind),
        };
        let layer = if layer.trim().is_empty() { front() } else { layer.trim().to_string() };
        let id = self.next_id();
        let pads = def
            .pads
            .iter()
            .map(|p| Pad { number: p.number.to_string(), at: [p.x, p.y], size: [p.w, p.h], shape: p.shape.to_string(), drill_mm: p.drill, net: None })
            .collect();
        self.footprints.push(Footprint {
            id,
            kind: def.kind.to_string(),
            reference: reference.clone(),
            value: value.unwrap_or("").trim().to_string(),
            at,
            rotation_deg,
            layer,
            pads,
        });
        Ok((id, reference))
    }

    fn next_reference(&self, kind: &str) -> String {
        let prefix = match kind {
            "r_0603" => "R",
            "c_0603" => "C",
            "led_0603" => "D",
            "ic8_dip" => "U",
            "header_2" => "J",
            _ => "MH",
        };
        for n in 1..=1_000_000u64 {
            let r = format!("{prefix}{n}");
            if !self.footprints.iter().any(|f| f.reference == r) {
                return r;
            }
        }
        format!("{prefix}?")
    }
}

impl Pcb {
    /// Set the net of one pad (by the footprint's reference and the pad's number).
    pub fn set_pad_net(&mut self, reference: &str, pad: &str, net: Option<&str>) -> Result<(), String> {
        let footprint =
            self.footprints.iter_mut().find(|f| f.reference == reference).ok_or_else(|| format!("no footprint with reference `{reference}`"))?;
        let p = footprint.pads.iter_mut().find(|p| p.number == pad).ok_or_else(|| format!("`{reference}` has no pad `{pad}`"))?;
        p.net = net.map(str::trim).filter(|n| !n.is_empty()).map(|n| n.to_string());
        Ok(())
    }

    /// Add a track from two or more points.
    pub fn add_track(&mut self, points: &[[f64; 2]], width_mm: f64, layer: &str, net: Option<&str>) -> Result<u64, String> {
        if points.len() < 2 {
            return Err("a track needs at least two points".into());
        }
        if points.len() > MAX_TRACK_POINTS {
            return Err(format!("a track holds at most {MAX_TRACK_POINTS} points"));
        }
        if points.iter().any(|p| !finite(*p)) {
            return Err("every track point must be two finite numbers in millimetres".into());
        }
        if !width_mm.is_finite() || !(0.01..=100.0).contains(&width_mm) {
            return Err("`width_mm` must be 0.01 to 100 mm".into());
        }
        if self.tracks.len() >= MAX_PCB_ITEMS {
            return Err(format!("a board holds at most {MAX_PCB_ITEMS} tracks"));
        }
        let id = self.next_id();
        let layer = if layer.trim().is_empty() { front() } else { layer.trim().to_string() };
        let net = net.map(str::trim).filter(|n| !n.is_empty()).map(|n| n.to_string());
        self.tracks.push(Track { id, points: points.to_vec(), width_mm, layer, net });
        Ok(id)
    }

    /// Add a via.
    pub fn add_via(&mut self, at: [f64; 2], drill_mm: f64, size_mm: f64, from: &str, to: &str, net: Option<&str>) -> Result<u64, String> {
        if !finite(at) {
            return Err("`at` must be a point [x, y] in millimetres".into());
        }
        if !drill_mm.is_finite() || !(0.05..=10.0).contains(&drill_mm) {
            return Err("`drill_mm` must be 0.05 to 10 mm".into());
        }
        if !size_mm.is_finite() || !(drill_mm..=20.0).contains(&size_mm) {
            return Err("`size_mm` must be at least the drill and at most 20 mm".into());
        }
        if self.vias.len() >= MAX_PCB_ITEMS {
            return Err(format!("a board holds at most {MAX_PCB_ITEMS} vias"));
        }
        let id = self.next_id();
        let from = if from.trim().is_empty() { front() } else { from.trim().to_string() };
        let to = if to.trim().is_empty() { back() } else { to.trim().to_string() };
        let net = net.map(str::trim).filter(|n| !n.is_empty()).map(|n| n.to_string());
        self.vias.push(Via { id, at, drill_mm, size_mm, from, to, net });
        Ok(id)
    }

    /// Add a copper zone from a closed polygon (the first point is not repeated).
    pub fn add_zone(&mut self, polygon: &[[f64; 2]], layer: &str, net: Option<&str>) -> Result<u64, String> {
        if polygon.len() < 3 {
            return Err("a zone needs at least three points".into());
        }
        if polygon.len() > MAX_POLYGON_POINTS {
            return Err(format!("a zone holds at most {MAX_POLYGON_POINTS} points"));
        }
        if polygon.iter().any(|p| !finite(*p)) {
            return Err("every zone point must be two finite numbers in millimetres".into());
        }
        if self.zones.len() >= MAX_PCB_ITEMS {
            return Err(format!("a board holds at most {MAX_PCB_ITEMS} zones"));
        }
        let id = self.next_id();
        let layer = if layer.trim().is_empty() { front() } else { layer.trim().to_string() };
        let net = net.map(str::trim).filter(|n| !n.is_empty()).map(|n| n.to_string());
        self.zones.push(Zone { id, polygon: polygon.to_vec(), layer, net });
        Ok(id)
    }
}

/// A net's summary: the pads, tracks, vias and zones that name it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetSummary {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pads: Vec<String>,
    #[serde(default)]
    pub tracks: usize,
    #[serde(default)]
    pub vias: usize,
    #[serde(default)]
    pub zones: usize,
}

impl Pcb {
    /// The nets the board names, from its pads, tracks, vias and zones.
    pub fn nets(&self) -> Vec<NetSummary> {
        use std::collections::BTreeMap;
        let mut nets: BTreeMap<String, NetSummary> = BTreeMap::new();
        for footprint in &self.footprints {
            for pad in &footprint.pads {
                if let Some(net) = &pad.net {
                    let e =
                        nets.entry(net.clone()).or_insert_with(|| NetSummary { name: net.clone(), pads: Vec::new(), tracks: 0, vias: 0, zones: 0 });
                    e.pads.push(format!("{}.{}", footprint.reference, pad.number));
                }
            }
        }
        for track in &self.tracks {
            if let Some(net) = &track.net {
                nets.entry(net.clone()).or_insert_with(|| summary(net)).tracks += 1;
            }
        }
        for via in &self.vias {
            if let Some(net) = &via.net {
                nets.entry(net.clone()).or_insert_with(|| summary(net)).vias += 1;
            }
        }
        for zone in &self.zones {
            if let Some(net) = &zone.net {
                nets.entry(net.clone()).or_insert_with(|| summary(net)).zones += 1;
            }
        }
        nets.into_values().collect()
    }
}

fn summary(name: &str) -> NetSummary {
    NetSummary { name: name.to_string(), pads: Vec::new(), tracks: 0, vias: 0, zones: 0 }
}

impl Pcb {
    /// The board as JSON for a command's reply.
    pub fn info(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "name": self.name,
            "width_mm": self.width_mm,
            "height_mm": self.height_mm,
            "thickness_mm": self.thickness_mm,
            "layers": self.layers.iter().map(|layer| json!({
                "name": layer.name, "kind": layer.kind, "thickness_um": layer.thickness_um,
            })).collect::<Vec<_>>(),
            "footprints": self.footprints.iter().map(|f| json!({
                "id": f.id,
                "kind": f.kind,
                "reference": f.reference,
                "value": f.value,
                "at": f.at,
                "rotation_deg": f.rotation_deg,
                "layer": f.layer,
                "pads": f.pads.iter().map(|p| json!({
                    "number": p.number,
                    "at": f.pad_world(p),
                    "size": p.size,
                    "shape": p.shape,
                    "drill_mm": p.drill_mm,
                    "net": p.net,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "tracks": self.tracks.iter().map(|t| json!({
                "id": t.id, "points": t.points, "width_mm": t.width_mm, "layer": t.layer, "net": t.net,
            })).collect::<Vec<_>>(),
            "vias": self.vias.iter().map(|v| json!({
                "id": v.id, "at": v.at, "drill_mm": v.drill_mm, "size_mm": v.size_mm,
                "from": v.from, "to": v.to, "net": v.net,
            })).collect::<Vec<_>>(),
            "zones": self.zones.iter().map(|z| json!({
                "id": z.id, "polygon": z.polygon, "layer": z.layer, "net": z.net,
            })).collect::<Vec<_>>(),
            "nets": self.nets(),
        })
    }
}

impl Pcb {
    /// Check a board read from a file before it replaces a session's.
    pub fn validate(&self) -> Result<(), String> {
        if !self.width_mm.is_finite() || !(1.0..=10_000.0).contains(&self.width_mm) {
            return Err("the board must be 1 to 10000 mm wide".into());
        }
        if !self.height_mm.is_finite() || !(1.0..=10_000.0).contains(&self.height_mm) {
            return Err("the board must be 1 to 10000 mm high".into());
        }
        if !self.thickness_mm.is_finite() || !(0.1..=20.0).contains(&self.thickness_mm) {
            return Err("the board must be 0.1 to 20 mm thick".into());
        }
        if self.footprints.len() > MAX_PCB_ITEMS
            || self.tracks.len() > MAX_PCB_ITEMS
            || self.vias.len() > MAX_PCB_ITEMS
            || self.zones.len() > MAX_PCB_ITEMS
        {
            return Err(format!("a board holds at most {MAX_PCB_ITEMS} footprints, tracks, vias and zones"));
        }
        for footprint in &self.footprints {
            if !finite(footprint.at) || !footprint.rotation_deg.is_finite() || footprint.rotation_deg.abs() > 1.0e6 {
                return Err(format!("footprint `{}` has an invalid placement", footprint.reference));
            }
            if footprint.reference.trim().is_empty() || footprint.reference.chars().count() > 64 {
                return Err("a reference holds 1 to 64 characters".into());
            }
            if footprint.pads.len() > 1024 {
                return Err(format!("footprint `{}` has too many pads", footprint.reference));
            }
            for pad in &footprint.pads {
                if !finite(pad.at) || pad.size.iter().any(|v| !v.is_finite() || *v <= 0.0) {
                    return Err(format!("footprint `{}` has an invalid pad", footprint.reference));
                }
                if let Some(drill) = pad.drill_mm
                    && (!drill.is_finite() || drill <= 0.0 || drill > pad.size[0].min(pad.size[1]))
                {
                    return Err(format!("footprint `{}` has a pad drill larger than the pad", footprint.reference));
                }
            }
        }
        for track in &self.tracks {
            if track.points.len() < 2 || track.points.len() > MAX_TRACK_POINTS || track.points.iter().any(|p| !finite(*p)) {
                return Err("a track holds 2 to 10000 finite points".into());
            }
            if !track.width_mm.is_finite() || !(0.01..=100.0).contains(&track.width_mm) {
                return Err("a track's width is 0.01 to 100 mm".into());
            }
        }
        for via in &self.vias {
            if !finite(via.at) || !via.drill_mm.is_finite() || !(0.05..=10.0).contains(&via.drill_mm) {
                return Err("a via has an invalid point or drill".into());
            }
            if !via.size_mm.is_finite() || !(via.drill_mm..=20.0).contains(&via.size_mm) {
                return Err("a via's size is at least its drill and at most 20 mm".into());
            }
        }
        for zone in &self.zones {
            if zone.polygon.len() < 3 || zone.polygon.len() > MAX_POLYGON_POINTS || zone.polygon.iter().any(|p| !finite(*p)) {
                return Err("a zone holds 3 to 50000 finite points".into());
            }
        }
        for layer in &self.layers {
            if layer.name.trim().is_empty() || layer.name.chars().count() > 64 || !layer.thickness_um.is_finite() {
                return Err("a layer needs a name of 1 to 64 characters and a finite thickness".into());
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

impl Pcb {
    /// The board as an SVG picture (8 px per millimetre): outline, zones, tracks, pads and vias.
    pub fn to_svg(&self) -> String {
        const SCALE: f64 = 8.0;
        let w = (self.width_mm * SCALE).max(1.0);
        let h = (self.height_mm * SCALE).max(1.0);
        let x = |x: f64| x * SCALE;
        let y = |y: f64| (self.height_mm - y) * SCALE;
        let mut svg = String::new();
        svg.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}\" height=\"{h:.2}\" viewBox=\"0 0 {w:.2} {h:.2}\">\n"));
        svg.push_str(&format!("<rect width=\"{w:.2}\" height=\"{h:.2}\" fill=\"#ffffff\"/>\n"));
        svg.push_str(&format!(
            "<rect x=\"0\" y=\"0\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"6\" fill=\"#1a5c3a\" stroke=\"#0b2a1a\" stroke-width=\"2\"/>\n"
        ));
        for zone in &self.zones {
            let pts: Vec<String> = zone.polygon.iter().map(|p| format!("{:.2},{:.2}", x(p[0]), y(p[1]))).collect();
            svg.push_str(&format!(
                "<polygon points=\"{}\" fill=\"#b87333\" fill-opacity=\"0.35\" stroke=\"#8a5520\" stroke-width=\"1\"/>\n",
                pts.join(" ")
            ));
        }
        for track in &self.tracks {
            let pts: Vec<String> = track.points.iter().map(|p| format!("{:.2},{:.2}", x(p[0]), y(p[1]))).collect();
            svg.push_str(&format!(
                "<polyline points=\"{}\" fill=\"none\" stroke=\"#b87333\" stroke-width=\"{:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n",
                pts.join(" "),
                track.width_mm * SCALE
            ));
        }
        for footprint in &self.footprints {
            for pad in &footprint.pads {
                let p = footprint.pad_world(pad);
                let (cx, cy) = (x(p[0]), y(p[1]));
                let (pw, ph) = (pad.size[0] * SCALE, pad.size[1] * SCALE);
                if pad.shape == "round" {
                    svg.push_str(&format!(
                        "<ellipse cx=\"{cx:.2}\" cy=\"{cy:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" fill=\"#d4a017\"/>\n",
                        pw / 2.0,
                        ph / 2.0
                    ));
                } else {
                    svg.push_str(&format!(
                        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{pw:.2}\" height=\"{ph:.2}\" fill=\"#d4a017\"/>\n",
                        cx - pw / 2.0,
                        cy - ph / 2.0
                    ));
                }
                if let Some(drill) = pad.drill_mm {
                    svg.push_str(&format!("<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{:.2}\" fill=\"#1a1a1a\"/>\n", drill * SCALE / 2.0));
                }
            }
        }
        for via in &self.vias {
            let (cx, cy) = (x(via.at[0]), y(via.at[1]));
            svg.push_str(&format!("<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{:.2}\" fill=\"#b87333\"/>\n", via.size_mm * SCALE / 2.0));
            svg.push_str(&format!("<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{:.2}\" fill=\"#1a1a1a\"/>\n", via.drill_mm * SCALE / 2.0));
        }
        for footprint in &self.footprints {
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#ffffff\">{}</text>\n",
                x(footprint.at[0]) + 4.0,
                y(footprint.at[1]) - 4.0,
                crate::escape(&footprint.reference)
            ));
        }
        svg.push_str(&format!(
            "<text x=\"6\" y=\"18\" font-family=\"sans-serif\" font-size=\"15\" fill=\"#ffffff\">{}</text>\n",
            crate::escape(&self.name)
        ));
        svg.push_str("</svg>\n");
        svg
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn a_board_holds_footprints_tracks_vias_and_zones() {
        let mut pcb = Pcb::default();
        let (_, reference) = pcb.place_footprint("ic8_dip", [40.0, 25.0], Some("U1"), Some("NE555"), 0.0, "F.Cu").unwrap();
        assert_eq!(reference, "U1");
        pcb.set_pad_net("U1", "1", Some("VIN")).unwrap();
        pcb.set_pad_net("U1", "2", Some("GND")).unwrap();
        pcb.add_track(&[[36.19, 21.19], [20.0, 21.19]], 0.4, "F.Cu", Some("VIN")).unwrap();
        pcb.add_via([20.0, 21.19], 0.4, 0.8, "F.Cu", "B.Cu", Some("VIN")).unwrap();
        pcb.add_zone(&[[10.0, 10.0], [30.0, 10.0], [30.0, 20.0], [10.0, 20.0]], "B.Cu", Some("GND")).unwrap();
        let info = pcb.info();
        assert_eq!(info["footprints"][0]["reference"], serde_json::json!("U1"));
        let nets = info["nets"].as_array().unwrap();
        assert!(nets.iter().any(|n| n["name"] == serde_json::json!("VIN") && n["pads"][0] == serde_json::json!("U1.1")));
        assert!(nets.iter().any(|n| n["name"] == serde_json::json!("GND") && n["zones"] == serde_json::json!(1)));
    }

    #[test]
    fn a_footprint_rotates_its_pads() {
        let mut pcb = Pcb::default();
        let (id, _) = pcb.place_footprint("r_0603", [10.0, 10.0], None, None, 90.0, "F.Cu").unwrap();
        let f = pcb.footprints.iter().find(|f| f.id == id).unwrap();
        let p = f.pad_world(&f.pads[0]);
        assert!((p[0] - 10.0).abs() < 1e-9, "{p:?}");
        assert!((p[1] - 9.25).abs() < 1e-9, "{p:?}");
    }

    #[test]
    fn unknown_footprints_and_bad_geometry_are_refused() {
        let mut pcb = Pcb::default();
        assert!(pcb.place_footprint("flux_package", [0.0, 0.0], None, None, 0.0, "F.Cu").is_err());
        assert!(pcb.add_track(&[[0.0, 0.0]], 0.3, "F.Cu", None).is_err());
        assert!(pcb.add_zone(&[[0.0, 0.0], [1.0, 1.0]], "F.Cu", None).is_err());
        assert!(pcb.add_via([0.0, 0.0], 1.0, 0.5, "F.Cu", "B.Cu", None).is_err());
    }

    #[test]
    fn a_saved_board_reads_back_and_draws_itself() {
        let mut pcb = Pcb::default();
        pcb.place_footprint("header_2", [10.0, 10.0], Some("J1"), Some("PWR"), 0.0, "F.Cu").unwrap();
        let text = pcb.to_json().unwrap();
        let back = Pcb::from_json(text.as_bytes()).unwrap();
        assert_eq!(back, pcb);
        assert!(back.validate().is_ok());
        let svg = pcb.to_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("J1"));
        assert!(!svg.contains("polygon"));
    }
}
