use egui::{
    Context, ScrollArea, SidePanel, TopBottomPanel, Ui, RichText,
};

pub fn modal_label(ui: &mut Ui, title: impl Into<RichText>) {
    ui.vertical_centered(|ui| {
        ui.heading(title);

        ui.separator();
        ui.add_space(8.0);
    });
}

pub fn top_panel(ctx: &Context, content: impl FnOnce(&mut Ui) -> ()) {
    TopBottomPanel::top("pages_panel").show(ctx, |ui| {
        ScrollArea::both().show(ui, |ui| {
            ui.horizontal_top(|ui| {
                content(ui);
            });
        });
    });
}

pub fn left_panel(ctx: &Context, content: impl FnOnce(&mut Ui) -> ()) {
    SidePanel::left("left_panel").show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            content(ui);
        });
    });
}
