//! The Component Manager, redesigned after the professional ECAD panels: a category tree on the
//! left, a device table in the middle and a symbol/footprint/3D preview on the right.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, vec2};
use serde_json::json;

use crate::SolveApp;
use crate::electronics_workspace::Tool;

pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    if !app.electronics.manager_open {
        return;
    }
    let t = crate::theme::Tokens::get();
    let mut open = app.electronics.manager_open;
    let mut filter = std::mem::take(&mut app.electronics.manager_filter);
    let mut category = app.electronics.manager_category.clone();
    let mut selected = app.electronics.manager_selected.clone();
    let mut place = None;
    let mut replace = None;
    let replace_reference = crate::electronics_workspace::selected_reference(app);
    let rows: Vec<(solidcraft_ecad::Device, usize)> = app
        .session
        .library
        .devices
        .iter()
        .map(|d| (d.clone(), app.session.library.in_design(d, app.session.schematic.as_ref(), app.session.pcb.as_ref())))
        .collect();
    egui::Window::new(crate::i18n::tr("library.manager")).open(&mut open).default_width(880.0).default_height(560.0).resizable(true).show(
        ctx,
        |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut filter).hint_text(crate::i18n::tr("library.search")).desired_width(260.0));
                ui.separator();
                ui.label(egui::RichText::new(format!("{} {}", rows.len(), crate::i18n::tr("library.devices"))).small().color(t.text_dim));
            });
            ui.separator();
            let needle = filter.trim().to_lowercase();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(vec2(160.0, 440.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.label(egui::RichText::new(crate::i18n::tr("library.categories")).strong().color(t.text));
                    if ui.selectable_label(category.is_none(), format!("{}  {}", crate::i18n::tr("library.all"), rows.len())).clicked() {
                        category = None;
                    }
                    let mut cats: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                    for (d, _) in &rows {
                        if needle.is_empty() || format!("{} {} {}", d.id, d.name, d.category).to_lowercase().contains(&needle) {
                            *cats.entry(d.category.clone()).or_default() += 1;
                        }
                    }
                    for (cat, count) in &cats {
                        if ui.selectable_label(category.as_deref() == Some(cat.as_str()), format!("{cat}  {count}")).clicked() {
                            category = Some(cat.clone());
                        }
                    }
                });
                ui.separator();
                let filtered: Vec<&(solidcraft_ecad::Device, usize)> = rows
                    .iter()
                    .filter(|(d, _)| {
                        category.as_ref().is_none_or(|c| *c == d.category)
                            && (needle.is_empty() || format!("{} {} {}", d.id, d.name, d.category).to_lowercase().contains(&needle))
                    })
                    .collect();
                ui.allocate_ui_with_layout(vec2(390.0, 440.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("manager_devices").striped(true).num_columns(5).min_col_width(42.0).show(ui, |ui| {
                            for h in ["library.component", "library.symbol", "library.footprint", "library.model3d", "library.in_design"] {
                                ui.label(egui::RichText::new(crate::i18n::tr(h)).small().strong().color(t.text_dim));
                            }
                            ui.end_row();
                            for (d, count) in filtered.iter().copied() {
                                if ui.selectable_label(selected.as_deref() == Some(d.id.as_str()), &d.name).clicked() {
                                    selected = Some(d.id.clone());
                                }
                                ui.label(&d.symbol);
                                ui.label(d.footprint.as_deref().unwrap_or("—"));
                                ui.label(crate::i18n::tr(if d.model.is_some() { "library.has_3d" } else { "library.no_3d" }));
                                ui.label(count.to_string());
                                ui.end_row();
                            }
                        });
                        if filtered.is_empty() {
                            ui.label(crate::i18n::tr("library.none"));
                        }
                    });
                });
                ui.separator();
                let preview = selected.as_ref().and_then(|id| rows.iter().find(|(d, _)| d.id == *id)).or_else(|| filtered.first().copied());
                ui.allocate_ui_with_layout(vec2(280.0, 440.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if let Some((d, count)) = preview {
                        ui.label(egui::RichText::new(&d.name).strong().size(16.0).color(t.text));
                        ui.label(egui::RichText::new(format!("{} · {} · {}", d.category, d.value, count)).small().color(t.text_dim));
                        ui.add_space(t.space_6);
                        preview_symbol(ui, &d.symbol, vec2(ui.available_width(), 118.0), &t);
                        ui.add_space(t.space_4);
                        preview_footprint(ui, d.footprint.as_deref(), vec2(ui.available_width(), 118.0), &t);
                        ui.add_space(t.space_4);
                        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, t.radius_sm, t.panel_header);
                        let icon = Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), vec2(18.0, 18.0));
                        crate::icons::paint_icon(ui.painter(), icon, "select_components", 1.0);
                        let text = d.model.as_deref().map_or_else(
                            || crate::i18n::tr("library.no_model").to_string(),
                            |m| format!("{}: {m}", crate::i18n::tr("library.model3d")),
                        );
                        ui.painter().text(
                            Pos2::new(rect.left() + 36.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            text,
                            FontId::proportional(11.0),
                            t.text_dim,
                        );
                        ui.add_space(t.space_6);
                        ui.horizontal(|ui| {
                            if ui.button(crate::i18n::tr("library.place")).clicked() {
                                place = Some(d.id.clone());
                            }
                            if replace_reference.is_some() && ui.button(crate::i18n::tr("library.replace")).clicked() {
                                replace = Some(d.id.clone());
                            }
                        });
                    } else {
                        ui.label(crate::i18n::tr("library.none"));
                    }
                });
            });
        },
    );
    app.electronics.manager_filter = filter;
    app.electronics.manager_category = category;
    app.electronics.manager_selected = selected;
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

/// A small symbol preview, drawn from the same primitives the schematic uses.
fn preview_symbol(ui: &mut egui::Ui, kind: &str, size: egui::Vec2, t: &crate::theme::Tokens) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, t.radius_sm, Color32::WHITE);
    painter.rect_stroke(rect, t.radius_sm, Stroke::new(1.0, t.border), StrokeKind::Inside);
    let primitives = solidcraft_ecad::symbol_primitives(kind);
    let def = solidcraft_ecad::SYMBOLS.iter().find(|d| d.kind == kind);
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    let mut add = |x: f64, y: f64| {
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
    };
    for primitive in &primitives {
        match primitive {
            solidcraft_ecad::Primitive::Polyline { points, .. } => {
                for p in points {
                    add(p[0], p[1]);
                }
            }
            solidcraft_ecad::Primitive::Circle { center, radius, .. } => {
                add(center[0] - radius, center[1] - radius);
                add(center[0] + radius, center[1] + radius);
            }
        }
    }
    if let Some(def) = def {
        for pin in def.pins {
            add(pin.x, pin.y);
        }
    }
    if !min[0].is_finite() {
        return;
    }
    let span = [(max[0] - min[0]).max(0.1), (max[1] - min[1]).max(0.1)];
    let scale = ((rect.width() - 24.0) / span[0] as f32).min((rect.height() - 24.0) / span[1] as f32).max(0.5);
    let mid = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
    let to = |p: [f64; 2]| Pos2::new(rect.center().x + (p[0] - mid[0]) as f32 * scale, rect.center().y - (p[1] - mid[1]) as f32 * scale);
    let ink = Color32::from_rgb(34, 34, 34);
    for primitive in &primitives {
        match primitive {
            solidcraft_ecad::Primitive::Polyline { points, width, closed, filled } => {
                let mut pts: Vec<Pos2> = points.iter().map(|p| to(*p)).collect();
                if pts.len() >= 2 {
                    if *filled {
                        painter.add(egui::Shape::convex_polygon(pts, ink, Stroke::NONE));
                    } else {
                        if *closed && let Some(first) = pts.first().copied() {
                            pts.push(first);
                        }
                        painter.add(egui::Shape::line(pts, Stroke::new((*width as f32 * scale).max(1.0), ink)));
                    }
                }
            }
            solidcraft_ecad::Primitive::Circle { center, radius, width, filled } => {
                let c = to(*center);
                let r = (*radius as f32 * scale).max(0.5);
                if *filled {
                    painter.circle_filled(c, r, ink);
                } else {
                    painter.circle_stroke(c, r, Stroke::new((*width as f32 * scale).max(1.0), ink));
                }
            }
        }
    }
    if let Some(def) = def {
        for pin in def.pins {
            painter.circle_filled(to([pin.x, pin.y]), 2.0, Color32::from_rgb(184, 134, 11));
        }
    }
}

/// A small footprint preview: the pads on a green board, with their drills.
fn preview_footprint(ui: &mut egui::Ui, kind: Option<&str>, size: egui::Vec2, t: &crate::theme::Tokens) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, t.radius_sm, Color32::from_rgb(26, 92, 58));
    painter.rect_stroke(rect, t.radius_sm, Stroke::new(1.0, Color32::from_rgb(11, 42, 26)), StrokeKind::Inside);
    let Some(kind) = kind else {
        painter.text(rect.center(), Align2::CENTER_CENTER, "—", FontId::proportional(14.0), Color32::WHITE);
        return;
    };
    let Some(def) = solidcraft_ecad::FOOTPRINTS.iter().find(|d| d.kind == kind) else {
        return;
    };
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for pad in def.pads {
        min[0] = min[0].min(pad.x - pad.w / 2.0);
        min[1] = min[1].min(pad.y - pad.h / 2.0);
        max[0] = max[0].max(pad.x + pad.w / 2.0);
        max[1] = max[1].max(pad.y + pad.h / 2.0);
    }
    let span = [(max[0] - min[0]).max(0.1), (max[1] - min[1]).max(0.1)];
    let scale = ((rect.width() - 24.0) / span[0] as f32).min((rect.height() - 24.0) / span[1] as f32).max(0.5);
    let mid = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
    let to = |p: [f64; 2]| Pos2::new(rect.center().x + (p[0] - mid[0]) as f32 * scale, rect.center().y - (p[1] - mid[1]) as f32 * scale);
    for pad in def.pads {
        let c = to([pad.x, pad.y]);
        let w = pad.w as f32 * scale;
        let h = pad.h as f32 * scale;
        let gold = Color32::from_rgb(212, 160, 23);
        if pad.shape == "round" {
            painter.circle_filled(c, w.min(h) / 2.0, gold);
        } else {
            painter.rect_filled(Rect::from_center_size(c, vec2(w, h)), 0.0, gold);
        }
        if let Some(drill) = pad.drill {
            painter.circle_filled(c, drill as f32 * scale / 2.0, Color32::from_rgb(26, 26, 26));
        }
    }
}
