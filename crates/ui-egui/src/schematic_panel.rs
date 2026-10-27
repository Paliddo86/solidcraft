//! The Schematic window: the session's schematic document (symbols, wires, net labels) drawn
//! with egui's painter, a read-only look at what the `schematic.*` commands built.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};

use crate::SolveApp;

/// Open the window when a schematic exists and the user has not just closed it. Adding anything
/// to the schematic (its `next_id` moves) opens it again.
pub fn open_if_any(app: &mut SolveApp) {
    let Some(sch) = app.session.schematic.as_ref() else { return };
    let fingerprint = sch.next_id;
    if app.schematic_panel.is_none() && app.schematic_panel_closed_for != Some(fingerprint) {
        app.schematic_panel = Some(Panel::default());
    }
}

/// The window's own state.
#[derive(Default)]
pub struct Panel {}

/// Draw the window when it is open. Called once per frame.
pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    if app.schematic_panel.is_none() {
        return;
    }
    let Some(sch) = app.session.schematic.as_ref() else {
        app.schematic_panel = None;
        return;
    };
    let name = sch.name.clone();
    let (w, h) = (sch.width_mm, sch.height_mm);
    let fingerprint = sch.next_id;
    let wires: Vec<Vec<[f64; 2]>> = sch.wires.iter().map(|wire| wire.points.clone()).collect();
    let labels: Vec<(String, [f64; 2])> = sch.labels.iter().map(|label| (label.net.clone(), label.at)).collect();
    let symbols: Vec<Symbol> = sch
        .symbols
        .iter()
        .map(|symbol| Symbol {
            reference: symbol.reference.clone(),
            value: symbol.value.clone(),
            pins: symbol.pins.iter().map(|pin| (symbol.pin_world(pin), pin.number.clone())).collect(),
        })
        .collect();
    let (closed, _) = crate::ui_kit::Window::new(egui::Id::new("sc_schematic"), "canvas", crate::i18n::tr_label("Schematic"))
        .actions("", crate::i18n::tr("ui.close"), None)
        .show(ctx, true, |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(760.0, 520.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let scale = (rect.width() / w as f32).min(rect.height() / h as f32);
            let to_screen = |p: [f64; 2]| Pos2::new(rect.left() + p[0] as f32 * scale, rect.top() + (h - p[1]) as f32 * scale);
            painter.rect_filled(rect, 0.0, Color32::WHITE);
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(120)), StrokeKind::Inside);
            painter.text(rect.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, name.as_str(), FontId::proportional(16.0), Color32::BLACK);
            for wire in &wires {
                let pts: Vec<Pos2> = wire.iter().map(|p| to_screen(*p)).collect();
                if pts.len() >= 2 {
                    painter.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::from_rgb(10, 125, 50))));
                }
            }
            for symbol in &symbols {
                let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
                let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
                for (p, _) in &symbol.pins {
                    min_x = min_x.min(p[0]);
                    min_y = min_y.min(p[1]);
                    max_x = max_x.max(p[0]);
                    max_y = max_y.max(p[1]);
                }
                if !min_x.is_finite() {
                    continue;
                }
                let pad = 3.0;
                let a = to_screen([min_x - pad, max_y + pad]);
                let b = to_screen([max_x + pad, min_y - pad]);
                let box_rect = Rect::from_min_max(a.min(b), a.max(b));
                painter.rect_filled(box_rect, 3.0, Color32::from_rgb(244, 231, 197));
                painter.rect_stroke(box_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(138, 109, 26)), StrokeKind::Inside);
                painter.text(
                    egui::pos2(box_rect.center().x, box_rect.top() - 2.0),
                    Align2::CENTER_BOTTOM,
                    symbol.reference.as_str(),
                    FontId::proportional(12.0),
                    Color32::BLACK,
                );
                if !symbol.value.is_empty() {
                    painter.text(
                        egui::pos2(box_rect.center().x, box_rect.bottom() + 2.0),
                        Align2::CENTER_TOP,
                        symbol.value.as_str(),
                        FontId::proportional(10.0),
                        Color32::from_gray(80),
                    );
                }
                for (p, number) in &symbol.pins {
                    let c = to_screen(*p);
                    painter.circle_filled(c, 2.0, Color32::from_rgb(184, 134, 11));
                    painter.text(c + egui::vec2(3.0, -3.0), Align2::LEFT_BOTTOM, number.as_str(), FontId::proportional(8.0), Color32::from_gray(60));
                }
            }
            for (net, at) in &labels {
                painter.text(
                    to_screen(*at) + egui::vec2(3.0, -2.0),
                    Align2::LEFT_BOTTOM,
                    net.as_str(),
                    FontId::proportional(11.0),
                    Color32::from_rgb(26, 79, 191),
                );
            }
        });
    if closed {
        app.schematic_panel = None;
        app.schematic_panel_closed_for = Some(fingerprint);
    }
}

struct Symbol {
    reference: String,
    value: String,
    pins: Vec<([f64; 2], String)>,
}
