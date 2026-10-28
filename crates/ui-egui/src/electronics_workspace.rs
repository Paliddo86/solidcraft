//! The Electronics workspace: schematic capture and board layout in the central panel, like the
//! Design and Render workspaces. Symbols and footprints can be placed, moved and deleted; wires,
//! tracks, labels, vias and zones are drawn by clicking points and pressing Finish.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};
use serde_json::{Value, json};

use crate::SolveApp;

/// Which electronics document the workspace shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Schematic,
    Pcb,
}

/// What a click on the canvas does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Symbol(&'static str),
    Wire,
    Label,
    Device,
    Footprint(&'static str),
    Track,
    Via,
    Zone,
}

/// The workspace's own state.
pub struct State {
    pub open: bool,
    pub tab: Tab,
    pub tool: Tool,
    pub selected: Option<u64>,
    pub pending: Vec<[f64; 2]>,
    pub label: String,
    pub status: String,
    pub manager_open: bool,
    pub manager_filter: String,
    pub place_device: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            tab: Tab::Schematic,
            tool: Tool::Select,
            selected: None,
            pending: Vec::new(),
            label: "NET".to_string(),
            status: String::new(),
            manager_open: true,
            manager_filter: String::new(),
            place_device: None,
        }
    }
}

pub fn active(app: &SolveApp) -> bool {
    app.electronics.open
}

/// Enter or leave the Electronics workspace; entering it leaves the Render workspace.
pub fn enter(app: &mut SolveApp, open: bool) -> Result<(), String> {
    app.electronics.open = open;
    if open {
        app.ui.tab = if app.electronics.tab == Tab::Pcb { "PCB".to_string() } else { "SCHEMATIC".to_string() };
        crate::render_workspace::enter(app, false)?;
    } else if app.ui.tab == "SCHEMATIC" || app.ui.tab == "PCB" {
        app.ui.tab = "MODEL".to_string();
    }
    Ok(())
}

/// The central panel while the Electronics workspace is active.
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get();
    match app.ui.tab.as_str() {
        "PCB" => app.electronics.tab = Tab::Pcb,
        "SCHEMATIC" => app.electronics.tab = Tab::Schematic,
        _ => app.ui.tab = if app.electronics.tab == Tab::Pcb { "PCB".to_string() } else { "SCHEMATIC".to_string() },
    }
    let view = ui.max_rect();
    let top = app.shell.cards.iter().map(|r| r.bottom()).fold(view.top(), f32::max);
    let left = app.shell.browser.map_or(view.left(), |b| (b.right() + 8.0).max(view.left()));
    let full = Rect::from_min_max(egui::pos2(left, top), view.max);
    let tabbar = Rect::from_min_size(full.left_top(), egui::vec2(full.width(), 34.0));
    let toolbar = Rect::from_min_size(egui::pos2(full.left(), tabbar.bottom()), egui::vec2(full.width(), 40.0));
    ui.painter().rect_filled(full, 0.0, t.panel);
    tab_button(app, ui, Rect::from_min_size(tabbar.left_top() + egui::vec2(12.0, 6.0), egui::vec2(100.0, 22.0)), Tab::Schematic, "Schematic");
    tab_button(app, ui, Rect::from_min_size(tabbar.left_top() + egui::vec2(120.0, 6.0), egui::vec2(70.0, 22.0)), Tab::Pcb, "PCB");
    ui.scope_builder(egui::UiBuilder::new().max_rect(toolbar.shrink2(egui::vec2(t.space_12, 5.0))), |ui| {
        ui.horizontal_wrapped(|ui| {
            tool_button(app, ui, Tool::Select, "Select");
            match app.electronics.tab {
                Tab::Schematic => {
                    for (kind, label) in [
                        ("resistor", "Resistor"),
                        ("capacitor", "Capacitor"),
                        ("ic8", "IC"),
                        ("ground", "Ground"),
                        ("vcc", "VCC"),
                        ("connector2", "Connector"),
                    ] {
                        tool_button(app, ui, Tool::Symbol(kind), label);
                    }
                    tool_button(app, ui, Tool::Wire, "Wire");
                    tool_button(app, ui, Tool::Label, "Net Label");
                    if app.electronics.tool == Tool::Label {
                        ui.add(egui::TextEdit::singleline(&mut app.electronics.label).desired_width(90.0));
                    }
                }
                Tab::Pcb => {
                    for (kind, label) in
                        [("r_0603", "R 0603"), ("c_0603", "C 0603"), ("ic8_dip", "DIP-8"), ("header_2", "Header"), ("mounting_hole", "Hole")]
                    {
                        tool_button(app, ui, Tool::Footprint(kind), label);
                    }
                    tool_button(app, ui, Tool::Track, "Track");
                    tool_button(app, ui, Tool::Via, "Via");
                    tool_button(app, ui, Tool::Zone, "Copper Zone");
                }
            }
            if !app.electronics.pending.is_empty() {
                let finish_label = match app.electronics.tab {
                    Tab::Schematic => "Finish Wire",
                    Tab::Pcb => "Finish Track",
                };
                if ui.button(crate::i18n::tr_label(finish_label)).clicked() {
                    finish(app);
                }
            }
            if app.electronics.selected.is_some() && ui.button(crate::i18n::tr_label("Delete")).clicked() {
                delete_selected(app);
            }
        });
    });
    let body = Rect::from_min_max(egui::pos2(full.left(), toolbar.bottom()), full.max);
    let response = ui.interact(body, egui::Id::new("electronics_canvas"), egui::Sense::click_and_drag());
    let pointer = response.interact_pointer_pos();
    if response.clicked()
        && let Some(pos) = pointer
    {
        handle_click(app, pos, body);
    }
    if response.drag_stopped()
        && let Some(pos) = pointer
    {
        move_selected(app, pos, body);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
        delete_selected(app);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.electronics.tool = Tool::Select;
        app.electronics.pending.clear();
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        finish(app);
    }
    match app.electronics.tab {
        Tab::Schematic => match app.session.schematic.as_ref() {
            Some(sch) => {
                let view = View::fit(body, sch.width_mm, sch.height_mm, 0.0);
                draw_schematic(sch, ui, &view, app.electronics.selected, &app.electronics.pending, pointer);
            }
            None => empty(ui, body, "No schematic yet: run schematic.create, then place symbols"),
        },
        Tab::Pcb => match app.session.pcb.as_ref() {
            Some(board) => {
                let view = View::fit(body, board.width_mm, board.height_mm, 14.0);
                draw_pcb(board, ui, &view, app.electronics.selected, &app.electronics.pending, pointer);
            }
            None => empty(ui, body, "No board yet: run pcb.create, then place footprints"),
        },
    }
    if !app.electronics.status.is_empty() {
        ui.painter().text(
            egui::pos2(body.left() + 8.0, body.bottom() - 6.0),
            Align2::LEFT_BOTTOM,
            app.electronics.status.as_str(),
            FontId::proportional(11.0),
            t.text_dim,
        );
    }
    manager_window(app, ui.ctx());
}

fn tab_button(app: &mut SolveApp, ui: &mut egui::Ui, rect: Rect, tab: Tab, label: &str) {
    let t = crate::theme::Tokens::get();
    let selected = app.electronics.tab == tab;
    if selected {
        ui.painter().rect_filled(rect, t.radius_sm, t.panel_header);
    }
    let galley =
        ui.painter().layout_no_wrap(crate::i18n::tr_label(label).to_string(), FontId::proportional(14.0), if selected { t.text } else { t.text_dim });
    ui.painter().galley(
        egui::pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0),
        galley,
        if selected { t.text } else { t.text_dim },
    );
    if ui.interact(rect, egui::Id::new(label), egui::Sense::click()).clicked() {
        app.electronics.tab = tab;
        app.ui.tab = if tab == Tab::Pcb { "PCB".to_string() } else { "SCHEMATIC".to_string() };
        app.electronics.tool = Tool::Select;
        app.electronics.pending.clear();
        app.electronics.selected = None;
    }
}

fn tool_button(app: &mut SolveApp, ui: &mut egui::Ui, tool: Tool, label: &str) {
    if ui.selectable_label(app.electronics.tool == tool, crate::i18n::tr_label(label)).clicked() {
        app.electronics.tool = tool;
        app.electronics.pending.clear();
    }
}

fn empty(ui: &egui::Ui, rect: Rect, text: &str) {
    let t = crate::theme::Tokens::get();
    ui.painter().rect_filled(rect, 0.0, Color32::from_gray(28));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, crate::i18n::tr_label(text), FontId::proportional(16.0), t.text_dim);
}

fn finish(app: &mut SolveApp) {
    let points = std::mem::take(&mut app.electronics.pending);
    if points.len() < 2 {
        return;
    }
    let result = match app.electronics.tab {
        Tab::Schematic => app.run("schematic.wire", json!({"points": points})),
        Tab::Pcb => app.run("pcb.track", json!({"points": points})),
    };
    if let Err(e) = result {
        app.electronics.status = e;
    }
}

fn delete_selected(app: &mut SolveApp) {
    let Some(id) = app.electronics.selected.take() else { return };
    let result = match app.electronics.tab {
        Tab::Schematic => app.run("schematic.symbol.delete", json!({"id": id})),
        Tab::Pcb => app.run("pcb.footprint.delete", json!({"id": id})),
    };
    if let Err(e) = result {
        app.electronics.status = e;
    }
}

fn handle_click(app: &mut SolveApp, pos: Pos2, body: Rect) {
    let Some(world) = world_at(app, body, pos) else { return };
    app.electronics.status.clear();
    match app.electronics.tab {
        Tab::Schematic => match app.electronics.tool {
            Tool::Select => app.electronics.selected = select_symbol(app, world),
            Tool::Symbol(kind) => place_symbol(app, kind, world),
            Tool::Wire => app.electronics.pending.push(world),
            Tool::Label => {
                let net = app.electronics.label.clone();
                if let Err(e) = app.run("schematic.label", json!({"net": net, "at": world})) {
                    app.electronics.status = e;
                }
            }
            Tool::Device => place_device(app, world),
            _ => {}
        },
        Tab::Pcb => match app.electronics.tool {
            Tool::Select => app.electronics.selected = select_footprint(app, world),
            Tool::Footprint(kind) => place_footprint(app, kind, world),
            Tool::Track => app.electronics.pending.push(world),
            Tool::Via => {
                if let Err(e) = app.run("pcb.via", json!({"at": world})) {
                    app.electronics.status = e;
                }
            }
            Tool::Zone => {
                app.electronics.pending.push(world);
                if app.electronics.pending.len() == 2 {
                    let a = app.electronics.pending[0];
                    let b = app.electronics.pending[1];
                    app.electronics.pending.clear();
                    let points = vec![a, [b[0], a[1]], b, [a[0], b[1]]];
                    if let Err(e) = app.run("pcb.zone", json!({"points": points, "layer": "B.Cu", "net": "GND"})) {
                        app.electronics.status = e;
                    }
                }
            }
            Tool::Device => place_device(app, world),
            _ => {}
        },
    }
}

fn place_symbol(app: &mut SolveApp, kind: &str, at: [f64; 2]) {
    match app.run("schematic.symbol.place", json!({"kind": kind, "at": at})) {
        Ok(v) => {
            app.electronics.selected = v.get("id").and_then(Value::as_u64);
            app.electronics.tool = Tool::Select;
        }
        Err(e) => app.electronics.status = e,
    }
}

fn place_footprint(app: &mut SolveApp, kind: &str, at: [f64; 2]) {
    match app.run("pcb.footprint.place", json!({"kind": kind, "at": at})) {
        Ok(v) => {
            app.electronics.selected = v.get("id").and_then(Value::as_u64);
            app.electronics.tool = Tool::Select;
        }
        Err(e) => app.electronics.status = e,
    }
}

/// Place the device picked in the Component Manager: its symbol on the schematic, its footprint
/// on the board.
fn place_device(app: &mut SolveApp, at: [f64; 2]) {
    let Some(device) = app.electronics.place_device.clone() else { return };
    let command = match app.electronics.tab {
        Tab::Schematic => "component.place",
        Tab::Pcb => "pcb.component.place",
    };
    match app.run(command, json!({"device": device, "at": at})) {
        Ok(v) => {
            app.electronics.selected = v.get("id").and_then(Value::as_u64);
            app.electronics.tool = Tool::Select;
            app.electronics.place_device = None;
        }
        Err(e) => app.electronics.status = e,
    }
}

fn select_symbol(app: &SolveApp, at: [f64; 2]) -> Option<u64> {
    let sch = app.session.schematic.as_ref()?;
    sch.symbols
        .iter()
        .map(|s| (s.id, (s.at[0] - at[0]).hypot(s.at[1] - at[1])))
        .filter(|(_, d)| *d <= 6.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

fn selected_reference(app: &SolveApp) -> Option<String> {
    match app.electronics.tab {
        Tab::Schematic => {
            app.session.schematic.as_ref()?.symbols.iter().find(|s| Some(s.id) == app.electronics.selected).map(|s| s.reference.clone())
        }
        Tab::Pcb => app.session.pcb.as_ref()?.footprints.iter().find(|f| Some(f.id) == app.electronics.selected).map(|f| f.reference.clone()),
    }
}

fn select_footprint(app: &SolveApp, at: [f64; 2]) -> Option<u64> {
    let board = app.session.pcb.as_ref()?;
    board
        .footprints
        .iter()
        .map(|f| (f.id, (f.at[0] - at[0]).hypot(f.at[1] - at[1])))
        .filter(|(_, d)| *d <= 8.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

fn move_selected(app: &mut SolveApp, pos: Pos2, body: Rect) {
    let Some(id) = app.electronics.selected else { return };
    let Some(world) = world_at(app, body, pos) else { return };
    let result = match app.electronics.tab {
        Tab::Schematic => app.run("schematic.symbol.move", json!({"id": id, "at": world})),
        Tab::Pcb => app.run("pcb.footprint.move", json!({"id": id, "at": world})),
    };
    if let Err(e) = result {
        app.electronics.status = e;
    }
}

fn world_at(app: &SolveApp, body: Rect, pos: Pos2) -> Option<[f64; 2]> {
    match app.electronics.tab {
        Tab::Schematic => app.session.schematic.as_ref().map(|sch| View::fit(body, sch.width_mm, sch.height_mm, 0.0).to_world(pos)),
        Tab::Pcb => app.session.pcb.as_ref().map(|board| View::fit(body, board.width_mm, board.height_mm, 14.0).to_world(pos)),
    }
}

/// Start an Electronics tool from the ribbon or the command palette, when the workspace is open.
pub fn start_tool(app: &mut SolveApp, id: &str) -> bool {
    if !active(app) {
        return false;
    }
    match id {
        "schematic.select" | "pcb.select" => app.electronics.tool = Tool::Select,
        "schematic.delete" | "pcb.delete" => delete_selected(app),
        "schematic.finish" | "pcb.finish" => finish(app),
        "schematic.wire" => app.electronics.tool = Tool::Wire,
        "schematic.label" => app.electronics.tool = Tool::Label,
        "schematic.symbol.place" => app.electronics.tool = Tool::Symbol("resistor"),
        "pcb.track" => app.electronics.tool = Tool::Track,
        "pcb.via" => app.electronics.tool = Tool::Via,
        "pcb.zone" => app.electronics.tool = Tool::Zone,
        "pcb.footprint.place" => app.electronics.tool = Tool::Footprint("r_0603"),
        "library.manager" => app.electronics.manager_open = !app.electronics.manager_open,
        _ => return false,
    }
    true
}

/// The Component Manager: the library's devices, with a search, their symbol/footprint/3D status
/// and a Place button that arms the canvas.
fn manager_window(app: &mut SolveApp, ctx: &egui::Context) {
    if !app.electronics.manager_open {
        return;
    }
    let t = crate::theme::Tokens::get();
    let mut open = app.electronics.manager_open;
    let mut place = None;
    let mut replace = None;
    let replace_reference = selected_reference(app);
    let mut filter = std::mem::take(&mut app.electronics.manager_filter);
    egui::Window::new(crate::i18n::tr("library.manager")).open(&mut open).default_width(360.0).resizable(true).show(ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut filter).hint_text(crate::i18n::tr("library.search")).desired_width(ui.available_width()));
        ui.add_space(t.space_6);
        let needle = filter.trim().to_lowercase();
        egui::ScrollArea::vertical().max_height(430.0).show(ui, |ui| {
            let mut shown = 0;
            for device in &app.session.library.devices {
                if !needle.is_empty() && !format!("{} {} {}", device.id, device.name, device.category).to_lowercase().contains(&needle) {
                    continue;
                }
                shown += 1;
                let count = app.session.library.in_design(device, app.session.schematic.as_ref(), app.session.pcb.as_ref());
                egui::Frame::NONE.fill(t.panel_header).inner_margin(6.0).corner_radius(t.radius_sm).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                        crate::icons::paint_icon(ui.painter(), rect, "select_components", 1.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&device.name).strong().color(t.text));
                            let footprint = device.footprint.as_deref().unwrap_or("—");
                            let model = device.model.as_deref().unwrap_or("—");
                            ui.label(egui::RichText::new(format!("{} · {} · 3D {}", device.category, footprint, model)).small().color(t.text_dim));
                        });
                        if ui.button(crate::i18n::tr("library.place")).clicked() {
                            place = Some(device.id.clone());
                        }
                        if replace_reference.is_some() && ui.button(crate::i18n::tr("library.replace")).clicked() {
                            replace = Some(device.id.clone());
                        }
                        ui.label(egui::RichText::new(count.to_string()).small().color(t.accent));
                    });
                });
                ui.add_space(t.space_4);
            }
            if shown == 0 {
                ui.label(crate::i18n::tr("library.none"));
            }
        });
    });
    app.electronics.manager_filter = filter;
    app.electronics.manager_open = open;
    if let Some(id) = place {
        app.electronics.place_device = Some(id);
        app.electronics.tool = Tool::Device;
    }
    if let (Some(id), Some(reference)) = (replace, replace_reference)
        && let Err(e) = app.run("library.replace", json!({"reference": reference, "device": id}))
    {
        app.electronics.status = e;
    }
}

/// The screen/world transform shared by the drawing and the pointer.
struct View {
    rect: Rect,
    scale: f32,
    ox: f32,
    oy: f32,
    h: f64,
}

impl View {
    fn fit(rect: Rect, w: f64, h: f64, margin: f32) -> Self {
        let scale = ((rect.width() - 2.0 * margin) / w as f32).min((rect.height() - 2.0 * margin) / h as f32).max(0.05);
        let ox = rect.left() + (rect.width() - w as f32 * scale) / 2.0;
        let oy = rect.top() + (rect.height() - h as f32 * scale) / 2.0;
        Self { rect, scale, ox, oy, h }
    }

    fn to_screen(&self, p: [f64; 2]) -> Pos2 {
        Pos2::new(self.ox + p[0] as f32 * self.scale, self.oy + (self.h - p[1]) as f32 * self.scale)
    }

    fn to_world(&self, p: Pos2) -> [f64; 2] {
        [((p.x - self.ox) / self.scale) as f64, self.h - ((p.y - self.oy) / self.scale) as f64]
    }
}

fn draw_schematic(sch: &solidcraft_ecad::Schematic, ui: &egui::Ui, view: &View, selected: Option<u64>, pending: &[[f64; 2]], pointer: Option<Pos2>) {
    let painter = ui.painter_at(view.rect);
    let a = view.to_screen([0.0, 0.0]);
    let b = view.to_screen([sch.width_mm, sch.height_mm]);
    let sheet = Rect::from_min_max(a.min(b), a.max(b));
    painter.rect_filled(sheet, 0.0, Color32::WHITE);
    painter.rect_stroke(sheet, 0.0, Stroke::new(1.0, Color32::from_gray(120)), StrokeKind::Inside);
    painter.text(sheet.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, sch.name.as_str(), FontId::proportional(16.0), Color32::BLACK);
    for wire in &sch.wires {
        let pts: Vec<Pos2> = wire.points.iter().map(|p| view.to_screen(*p)).collect();
        if pts.len() >= 2 {
            painter.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::from_rgb(10, 125, 50))));
        }
    }
    for symbol in &sch.symbols {
        if selected == Some(symbol.id) {
            painter.circle_stroke(view.to_screen(symbol.at), 4.0 * view.scale, Stroke::new(2.0, Color32::from_rgb(230, 180, 30)));
        }
        for primitive in solidcraft_ecad::symbol_primitives(&symbol.kind) {
            match primitive {
                solidcraft_ecad::Primitive::Polyline { points, width, closed, filled } => {
                    let pts: Vec<Pos2> = points.iter().map(|p| view.to_screen(symbol.point_world(*p))).collect();
                    if pts.len() >= 2 {
                        let stroke = Stroke::new((width as f32 * view.scale).max(1.0), Color32::from_rgb(34, 34, 34));
                        if filled {
                            painter.add(egui::Shape::convex_polygon(pts, Color32::from_rgb(34, 34, 34), Stroke::NONE));
                        } else {
                            let mut pts = pts;
                            if closed && let Some(first) = pts.first().copied() {
                                pts.push(first);
                            }
                            painter.add(egui::Shape::line(pts, stroke));
                        }
                    }
                }
                solidcraft_ecad::Primitive::Circle { center, radius, width, filled } => {
                    let c = view.to_screen(symbol.point_world(center));
                    let r = (radius as f32 * view.scale).max(0.5);
                    if filled {
                        painter.circle_filled(c, r, Color32::from_rgb(34, 34, 34));
                    } else {
                        painter.circle_stroke(c, r, Stroke::new((width as f32 * view.scale).max(1.0), Color32::from_rgb(34, 34, 34)));
                    }
                }
            }
        }
        let (mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY);
        for pin in &symbol.pins {
            let p = symbol.pin_world(pin);
            min_y = min_y.min(p[1]);
            max_y = max_y.max(p[1]);
        }
        if min_y.is_finite() {
            painter.text(
                view.to_screen([symbol.at[0], max_y + 3.0]),
                Align2::CENTER_BOTTOM,
                symbol.reference.as_str(),
                FontId::proportional(12.0),
                Color32::BLACK,
            );
            if !symbol.value.is_empty() {
                painter.text(
                    view.to_screen([symbol.at[0], min_y - 3.0]),
                    Align2::CENTER_TOP,
                    symbol.value.as_str(),
                    FontId::proportional(10.0),
                    Color32::from_gray(80),
                );
            }
        }
        for pin in &symbol.pins {
            painter.circle_filled(view.to_screen(symbol.pin_world(pin)), 1.6, Color32::from_rgb(184, 134, 11));
        }
    }
    for label in &sch.labels {
        painter.text(
            view.to_screen(label.at) + egui::vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            label.net.as_str(),
            FontId::proportional(11.0),
            Color32::from_rgb(26, 79, 191),
        );
    }
    if !pending.is_empty() {
        let mut pts: Vec<Pos2> = pending.iter().map(|p| view.to_screen(*p)).collect();
        if let Some(p) = pointer {
            pts.push(p);
        }
        if pts.len() >= 2 {
            painter.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::from_rgb(10, 125, 50))));
        }
        for p in pending {
            painter.circle_filled(view.to_screen(*p), 3.0, Color32::from_rgb(10, 125, 50));
        }
    }
}

fn draw_pcb(board: &solidcraft_ecad::Pcb, ui: &egui::Ui, view: &View, selected: Option<u64>, pending: &[[f64; 2]], pointer: Option<Pos2>) {
    let painter = ui.painter_at(view.rect);
    painter.rect_filled(view.rect, 0.0, Color32::from_gray(22));
    let a = view.to_screen([0.0, 0.0]);
    let b = view.to_screen([board.width_mm, board.height_mm]);
    let board_rect = Rect::from_min_max(a.min(b), a.max(b));
    painter.rect_filled(board_rect, 6.0, Color32::from_rgb(26, 92, 58));
    painter.rect_stroke(board_rect, 6.0, Stroke::new(1.5, Color32::from_rgb(11, 42, 26)), StrokeKind::Inside);
    for zone in &board.zones {
        let points: Vec<Pos2> = zone.polygon.iter().map(|p| view.to_screen(*p)).collect();
        if points.len() >= 3 {
            painter.add(egui::Shape::convex_polygon(
                points,
                Color32::from_rgba_unmultiplied(184, 115, 51, 90),
                Stroke::new(1.0, Color32::from_rgb(138, 85, 32)),
            ));
        }
    }
    for track in &board.tracks {
        let points: Vec<Pos2> = track.points.iter().map(|p| view.to_screen(*p)).collect();
        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, Stroke::new((track.width_mm as f32 * view.scale).max(1.0), Color32::from_rgb(184, 115, 51))));
        }
    }
    for footprint in &board.footprints {
        if selected == Some(footprint.id) {
            painter.circle_stroke(view.to_screen(footprint.at), 5.0 * view.scale, Stroke::new(2.0, Color32::from_rgb(230, 180, 30)));
        }
        for pad in &footprint.pads {
            let center = view.to_screen(footprint.pad_world(pad));
            let (pw, ph) = (pad.size[0] as f32 * view.scale, pad.size[1] as f32 * view.scale);
            let gold = Color32::from_rgb(212, 160, 23);
            if pad.shape == "round" {
                painter.circle_filled(center, pw.min(ph) / 2.0, gold);
            } else {
                painter.rect_filled(Rect::from_center_size(center, egui::vec2(pw, ph)), 0.0, gold);
            }
            if let Some(drill) = pad.drill_mm {
                painter.circle_filled(center, drill as f32 * view.scale / 2.0, Color32::from_rgb(26, 26, 26));
            }
        }
        painter.text(
            view.to_screen(footprint.at) + egui::vec2(3.0, -3.0),
            Align2::LEFT_BOTTOM,
            footprint.reference.as_str(),
            FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
    for via in &board.vias {
        let center = view.to_screen(via.at);
        painter.circle_filled(center, via.size_mm as f32 * view.scale / 2.0, Color32::from_rgb(184, 115, 51));
        painter.circle_filled(center, via.drill_mm as f32 * view.scale / 2.0, Color32::from_rgb(26, 26, 26));
    }
    painter.text(board_rect.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, board.name.as_str(), FontId::proportional(14.0), Color32::WHITE);
    if !pending.is_empty() {
        let mut pts: Vec<Pos2> = pending.iter().map(|p| view.to_screen(*p)).collect();
        if let Some(p) = pointer {
            pts.push(p);
        }
        if pts.len() >= 2 {
            painter.add(egui::Shape::line(pts, Stroke::new(2.0, Color32::from_rgb(184, 115, 51))));
        }
        for p in pending {
            painter.circle_filled(view.to_screen(*p), 3.0, Color32::from_rgb(184, 115, 51));
        }
    }
}
