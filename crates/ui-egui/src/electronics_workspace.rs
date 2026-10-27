//! The Electronics workspace: the schematic and board canvases in the central panel, like the
//! Design and Render workspaces. The documents are the session's `schematic` and `pcb`.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};

use crate::SolveApp;

/// Which electronics document the workspace shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Schematic,
    Pcb,
}

/// The workspace's own state.
#[derive(Default)]
pub struct State {
    pub open: bool,
    pub tab: Tab,
}

pub fn active(app: &SolveApp) -> bool {
    app.electronics.open
}

/// Enter or leave the Electronics workspace; entering it leaves the Render workspace.
pub fn enter(app: &mut SolveApp, open: bool) -> Result<(), String> {
    app.electronics.open = open;
    if open {
        crate::render_workspace::enter(app, false)?;
    }
    Ok(())
}

/// The central panel while the Electronics workspace is active.
pub fn show(app: &mut SolveApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get();
    let full = ui.max_rect();
    let bar = Rect::from_min_size(full.left_top(), egui::vec2(full.width(), 34.0));
    ui.painter().rect_filled(full, 0.0, t.panel);
    let mut x = bar.left() + t.space_12;
    for (tab, label) in [(Tab::Schematic, "Schematic"), (Tab::Pcb, "PCB")] {
        let selected = app.electronics.tab == tab;
        let text = crate::i18n::tr_label(label);
        let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(14.0), if selected { t.text } else { t.text_dim });
        let rect = Rect::from_min_size(egui::pos2(x, bar.center().y - galley.size().y / 2.0 - 3.0), galley.size() + egui::vec2(16.0, 6.0));
        if selected {
            ui.painter().rect_filled(rect, t.radius_sm, t.panel_header);
        }
        let response = ui.interact(rect, egui::Id::new(label), egui::Sense::click());
        ui.painter().galley(
            egui::pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0),
            galley,
            if selected { t.text } else { t.text_dim },
        );
        if response.clicked() {
            app.electronics.tab = tab;
        }
        x = rect.right() + t.space_16;
    }
    let body = Rect::from_min_max(egui::pos2(full.left(), bar.bottom()), full.max);
    match app.electronics.tab {
        Tab::Schematic => match app.session.schematic.as_ref() {
            Some(sch) => draw_schematic(sch, ui, body),
            None => empty(ui, body, "No schematic yet: run schematic.create, then place symbols"),
        },
        Tab::Pcb => match app.session.pcb.as_ref() {
            Some(board) => draw_pcb(board, ui, body),
            None => empty(ui, body, "No board yet: run pcb.create, then place footprints"),
        },
    }
}

fn empty(ui: &egui::Ui, rect: Rect, text: &str) {
    let t = crate::theme::Tokens::get();
    ui.painter().rect_filled(rect, 0.0, Color32::from_gray(28));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(16.0), t.text_dim);
}

/// Draw the schematic sheet fitted into `rect`.
fn draw_schematic(sch: &solidcraft_ecad::Schematic, ui: &egui::Ui, rect: Rect) {
    let (w, h) = (sch.width_mm, sch.height_mm);
    let painter = ui.painter_at(rect);
    let scale = (rect.width() / w as f32).min(rect.height() / h as f32);
    let to_screen = |p: [f64; 2]| Pos2::new(rect.left() + p[0] as f32 * scale, rect.top() + (h - p[1]) as f32 * scale);
    painter.rect_filled(rect, 0.0, Color32::WHITE);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(120)), StrokeKind::Inside);
    painter.text(rect.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, sch.name.as_str(), FontId::proportional(16.0), Color32::BLACK);
    for wire in &sch.wires {
        let pts: Vec<Pos2> = wire.points.iter().map(|p| to_screen(*p)).collect();
        if pts.len() >= 2 {
            painter.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::from_rgb(10, 125, 50))));
        }
    }
    for symbol in &sch.symbols {
        for primitive in solidcraft_ecad::symbol_primitives(&symbol.kind) {
            match primitive {
                solidcraft_ecad::Primitive::Polyline { points, width, closed, filled } => {
                    let pts: Vec<Pos2> = points.iter().map(|p| to_screen(symbol.point_world(*p))).collect();
                    if pts.len() >= 2 {
                        let stroke = Stroke::new((width as f32 * scale).max(1.0), Color32::from_rgb(34, 34, 34));
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
                    let c = to_screen(symbol.point_world(center));
                    let r = (radius as f32 * scale).max(0.5);
                    if filled {
                        painter.circle_filled(c, r, Color32::from_rgb(34, 34, 34));
                    } else {
                        painter.circle_stroke(c, r, Stroke::new((width as f32 * scale).max(1.0), Color32::from_rgb(34, 34, 34)));
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
                to_screen([symbol.at[0], max_y + 3.0]),
                Align2::CENTER_BOTTOM,
                symbol.reference.as_str(),
                FontId::proportional(12.0),
                Color32::BLACK,
            );
            if !symbol.value.is_empty() {
                painter.text(
                    to_screen([symbol.at[0], min_y - 3.0]),
                    Align2::CENTER_TOP,
                    symbol.value.as_str(),
                    FontId::proportional(10.0),
                    Color32::from_gray(80),
                );
            }
        }
        for pin in &symbol.pins {
            painter.circle_filled(to_screen(symbol.pin_world(pin)), 1.6, Color32::from_rgb(184, 134, 11));
        }
    }
    for label in &sch.labels {
        painter.text(
            to_screen(label.at) + egui::vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            label.net.as_str(),
            FontId::proportional(11.0),
            Color32::from_rgb(26, 79, 191),
        );
    }
}

/// Draw the board fitted into `rect`.
fn draw_pcb(board: &solidcraft_ecad::Pcb, ui: &egui::Ui, rect: Rect) {
    let (w, h) = (board.width_mm, board.height_mm);
    let painter = ui.painter_at(rect);
    let margin = 14.0;
    let scale = ((rect.width() - 2.0 * margin) / w as f32).min((rect.height() - 2.0 * margin) / h as f32);
    let ox = rect.left() + (rect.width() - w as f32 * scale) / 2.0;
    let oy = rect.top() + (rect.height() - h as f32 * scale) / 2.0;
    let to_screen = |p: [f64; 2]| Pos2::new(ox + p[0] as f32 * scale, oy + (h - p[1]) as f32 * scale);
    painter.rect_filled(rect, 0.0, Color32::from_gray(22));
    let a = to_screen([0.0, 0.0]);
    let b = to_screen([w, h]);
    let board_rect = Rect::from_min_max(a.min(b), a.max(b));
    painter.rect_filled(board_rect, 6.0, Color32::from_rgb(26, 92, 58));
    painter.rect_stroke(board_rect, 6.0, Stroke::new(1.5, Color32::from_rgb(11, 42, 26)), StrokeKind::Inside);
    for zone in &board.zones {
        let points: Vec<Pos2> = zone.polygon.iter().map(|p| to_screen(*p)).collect();
        if points.len() >= 3 {
            painter.add(egui::Shape::convex_polygon(
                points,
                Color32::from_rgba_unmultiplied(184, 115, 51, 90),
                Stroke::new(1.0, Color32::from_rgb(138, 85, 32)),
            ));
        }
    }
    for track in &board.tracks {
        let points: Vec<Pos2> = track.points.iter().map(|p| to_screen(*p)).collect();
        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, Stroke::new((track.width_mm as f32 * scale).max(1.0), Color32::from_rgb(184, 115, 51))));
        }
    }
    for footprint in &board.footprints {
        for pad in &footprint.pads {
            let center = to_screen(footprint.pad_world(pad));
            let (pw, ph) = (pad.size[0] as f32 * scale, pad.size[1] as f32 * scale);
            let gold = Color32::from_rgb(212, 160, 23);
            if pad.shape == "round" {
                painter.circle_filled(center, pw.min(ph) / 2.0, gold);
            } else {
                painter.rect_filled(Rect::from_center_size(center, egui::vec2(pw, ph)), 0.0, gold);
            }
            if let Some(drill) = pad.drill_mm {
                painter.circle_filled(center, drill as f32 * scale / 2.0, Color32::from_rgb(26, 26, 26));
            }
        }
        painter.text(
            to_screen(footprint.at) + egui::vec2(3.0, -3.0),
            Align2::LEFT_BOTTOM,
            footprint.reference.as_str(),
            FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
    for via in &board.vias {
        let center = to_screen(via.at);
        painter.circle_filled(center, via.size_mm as f32 * scale / 2.0, Color32::from_rgb(184, 115, 51));
        painter.circle_filled(center, via.drill_mm as f32 * scale / 2.0, Color32::from_rgb(26, 26, 26));
    }
    painter.text(board_rect.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, board.name.as_str(), FontId::proportional(14.0), Color32::WHITE);
}
