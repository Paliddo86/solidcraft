//! The PCB window: the session's board (outline, zones, tracks, pads, vias) drawn with egui's
//! painter, a read-only look at what the `pcb.*` commands built.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};

use crate::SolveApp;

/// Open the window when a board exists and the user has not just closed it. Adding anything to
/// the board (its `next_id` moves) opens it again.
pub fn open_if_any(app: &mut SolveApp) {
    if crate::electronics_workspace::active(app) {
        return;
    }
    let Some(board) = app.session.pcb.as_ref() else { return };
    let fingerprint = board.next_id;
    if app.pcb_panel.is_none() && app.pcb_panel_closed_for != Some(fingerprint) {
        app.pcb_panel = Some(Panel::default());
    }
}

/// The window's own state.
#[derive(Default)]
pub struct Panel {}

/// Draw the window when it is open. Called once per frame.
pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    if app.pcb_panel.is_none() {
        return;
    }
    if crate::electronics_workspace::active(app) {
        return;
    }
    let Some(board) = app.session.pcb.as_ref() else {
        app.pcb_panel = None;
        return;
    };
    let name = board.name.clone();
    let (w, h) = (board.width_mm, board.height_mm);
    let fingerprint = board.next_id;
    let zones: Vec<Vec<[f64; 2]>> = board.zones.iter().map(|zone| zone.polygon.clone()).collect();
    let tracks: Vec<(Vec<[f64; 2]>, f64)> = board.tracks.iter().map(|track| (track.points.clone(), track.width_mm)).collect();
    let mut pads: Vec<([f64; 2], [f64; 2], bool, Option<f64>)> = Vec::new();
    let mut references: Vec<(String, [f64; 2])> = Vec::new();
    for footprint in &board.footprints {
        references.push((footprint.reference.clone(), footprint.at));
        for pad in &footprint.pads {
            pads.push((footprint.pad_world(pad), pad.size, pad.shape == "round", pad.drill_mm));
        }
    }
    let vias: Vec<([f64; 2], f64, f64)> = board.vias.iter().map(|via| (via.at, via.size_mm, via.drill_mm)).collect();
    let (closed, _) = crate::ui_kit::Window::new(egui::Id::new("sc_pcb"), "canvas", crate::i18n::tr_label("PCB"))
        .actions("", crate::i18n::tr("ui.close"), None)
        .show(ctx, true, |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(760.0, 520.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let margin = 14.0;
            let scale = ((rect.width() - 2.0 * margin) / w as f32).min((rect.height() - 2.0 * margin) / h as f32);
            let ox = rect.left() + (rect.width() - w as f32 * scale) / 2.0;
            let oy = rect.top() + (rect.height() - h as f32 * scale) / 2.0;
            let to_screen = |p: [f64; 2]| Pos2::new(ox + p[0] as f32 * scale, oy + (h - p[1]) as f32 * scale);
            painter.rect_filled(rect, 0.0, Color32::WHITE);
            let a = to_screen([0.0, 0.0]);
            let b = to_screen([w, h]);
            let board_rect = Rect::from_min_max(a.min(b), a.max(b));
            painter.rect_filled(board_rect, 6.0, Color32::from_rgb(26, 92, 58));
            painter.rect_stroke(board_rect, 6.0, Stroke::new(1.5, Color32::from_rgb(11, 42, 26)), StrokeKind::Inside);
            for polygon in &zones {
                let points: Vec<Pos2> = polygon.iter().map(|p| to_screen(*p)).collect();
                if points.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        Color32::from_rgba_unmultiplied(184, 115, 51, 90),
                        Stroke::new(1.0, Color32::from_rgb(138, 85, 32)),
                    ));
                }
            }
            for (points, width) in &tracks {
                let points: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points, Stroke::new((*width as f32 * scale).max(1.0), Color32::from_rgb(184, 115, 51))));
                }
            }
            for (at, size, round, drill) in &pads {
                let center = to_screen(*at);
                let (pw, ph) = (size[0] as f32 * scale, size[1] as f32 * scale);
                let gold = Color32::from_rgb(212, 160, 23);
                if *round {
                    painter.circle_filled(center, pw.min(ph) / 2.0, gold);
                } else {
                    painter.rect_filled(Rect::from_center_size(center, egui::vec2(pw, ph)), 0.0, gold);
                }
                if let Some(drill) = drill {
                    painter.circle_filled(center, *drill as f32 * scale / 2.0, Color32::from_rgb(26, 26, 26));
                }
            }
            for (at, size, drill) in &vias {
                let center = to_screen(*at);
                painter.circle_filled(center, *size as f32 * scale / 2.0, Color32::from_rgb(184, 115, 51));
                painter.circle_filled(center, *drill as f32 * scale / 2.0, Color32::from_rgb(26, 26, 26));
            }
            for (reference, at) in &references {
                painter.text(
                    to_screen(*at) + egui::vec2(3.0, -3.0),
                    Align2::LEFT_BOTTOM,
                    reference.as_str(),
                    FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
            painter.text(board_rect.left_top() + egui::vec2(6.0, 4.0), Align2::LEFT_TOP, name.as_str(), FontId::proportional(14.0), Color32::WHITE);
        });
    if closed {
        app.pcb_panel = None;
        app.pcb_panel_closed_for = Some(fingerprint);
    }
}
