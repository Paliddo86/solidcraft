//! The Electronics workspace: the schematic and board canvases in the central panel, like the
//! Design and Render workspaces. The documents are the session's `schematic` and `pcb`.

use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, StrokeKind};

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
    let t = crate::ui_kit::Tokens::get();
    ui.painter().rect_filled(ui.max_rect(), 0.0, t.panel);
    ui.horizontal(|ui| {
        ui.add_space(t.space_12);
        for (tab, label) in [(Tab::Schematic, "Schematic"), (Tab::Pcb, "PCB")] {
            let selected = app.electronics.tab == tab;
            if ui.selectable_label(selected, RichText::new(crate::i18n::tr_label(label)).strong()).clicked() {
                app.electronics.tab = tab;
            }
        }
    });
    let rect = ui.available_rect_before_wrap();
    match app.electronics.tab {
        Tab::Schematic => match app.session.schematic.as_ref() {
            Some(sch) => draw_schematic(sch, ui, rect),
            None => empty(ui, rect, "No schematic yet: run schematic.create, then place symbols"),
        },
        Tab::Pcb => match app.session.pcb.as_ref() {
            Some(board) => draw_pcb(board, ui, rect),
            None => empty(ui, rect, "No board yet: run pcb.create, then place footprints"),
        },
    }
}

fn empty(ui: &egui::Ui, rect: Rect, text: &str) {
    let t = crate::ui_kit::Tokens::get();
    ui.painter().rect_filled(rect, 0.0, Color32::from_gray(28));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(16.0), t.text_dim);
}
