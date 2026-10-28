//! The component library: a device pairs a schematic symbol kind, a board footprint kind and an
//! optional 3D model. The kinds are the ones `SYMBOLS` and `FOOTPRINTS` already define.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{Pcb, Schematic};

/// The devices a design can place, plus the ones a script adds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub devices: Vec<Device>,
    #[serde(default)]
    pub in_use: BTreeSet<String>,
}

/// One library device: its symbol, its footprint (if it has one) and its 3D model (if any).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub value: String,
}

impl Device {
    pub fn new(id: &str, name: &str, symbol: &str, footprint: Option<&str>, category: &str, value: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            footprint: footprint.map(str::to_string),
            model: None,
            category: category.to_string(),
            value: value.to_string(),
        }
    }
}

impl Library {
    /// The devices SolidCraft ships: the standard symbols, paired with the standard footprints.
    pub fn built_in() -> Self {
        let devices = vec![
            Device::new("resistor", "Resistor", "resistor", Some("r_0603"), "Passive", "10k"),
            Device::new("capacitor", "Capacitor", "capacitor", Some("c_0603"), "Passive", "100n"),
            Device::new("inductor", "Inductor", "inductor", None, "Passive", "10u"),
            Device::new("diode", "Diode", "diode", None, "Semiconductor", "1N4148"),
            Device::new("led", "LED", "led", Some("led_0603"), "Semiconductor", "Red"),
            Device::new("ic8", "8-pin IC", "ic8", Some("ic8_dip"), "Integrated circuit", "NE555"),
            Device::new("connector2", "2-pin connector", "connector2", Some("header_2"), "Connector", "PWR"),
            Device::new("ground", "Ground", "ground", None, "Power", "GND"),
            Device::new("vcc", "Supply", "vcc", None, "Power", "VCC"),
        ];
        let in_use = devices.iter().map(|d| d.id.clone()).collect();
        Self { devices, in_use }
    }

    pub fn device(&self, id: &str) -> Option<&Device> {
        self.devices.iter().find(|d| d.id == id)
    }

    pub fn add(&mut self, device: Device) -> Result<(), String> {
        let id = device.id.trim();
        if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err("`id` must be 1 to 64 letters, digits, `_` or `-`".into());
        }
        if self.devices.iter().any(|d| d.id == id) {
            return Err(format!("the library already has `{id}`"));
        }
        self.in_use.insert(id.to_string());
        self.devices.push(device);
        Ok(())
    }

    /// Enable or disable a device in the design (Fusion's Library Manager "In Use" toggle).
    pub fn enable(&mut self, id: &str, on: bool) -> Result<(), String> {
        if self.device(id).is_none() {
            return Err(format!("no device `{id}`"));
        }
        if on {
            self.in_use.insert(id.to_string());
        } else {
            self.in_use.remove(id);
        }
        Ok(())
    }

    /// How many placed components use this device (the larger of its symbols and footprints).
    pub fn in_design(&self, device: &Device, schematic: Option<&Schematic>, pcb: Option<&Pcb>) -> usize {
        let symbols = schematic.map_or(0, |s| s.symbols.iter().filter(|x| x.kind == device.symbol).count());
        let footprints = device.footprint.as_ref().map_or(0, |f| pcb.map_or(0, |b| b.footprints.iter().filter(|x| x.kind == *f).count()));
        symbols.max(footprints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_devices_pair_symbols_and_footprints() {
        let library = Library::built_in();
        assert!(library.device("resistor").is_some_and(|d| d.symbol == "resistor" && d.footprint.as_deref() == Some("r_0603")));
        assert!(library.device("ground").is_some_and(|d| d.footprint.is_none()));
        assert_eq!(library.in_use.len(), library.devices.len());
    }

    #[test]
    fn devices_are_added_once_and_enabled() {
        let mut library = Library::built_in();
        let device = Device::new("sensor", "Sensor", "ic8", Some("ic8_dip"), "Sensor", "BME280");
        library.add(device.clone()).unwrap();
        assert!(library.add(device).is_err());
        library.enable("sensor", false).unwrap();
        assert!(!library.in_use.contains("sensor"));
        library.enable("sensor", true).unwrap();
        assert!(library.in_use.contains("sensor"));
    }
}
