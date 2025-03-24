use egui::{
    Context, ScrollArea, SidePanel, TopBottomPanel, Ui, RichText, Button,
};

use crate::data::structs;

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

pub fn script_preset(ui: &mut Ui, pages: &mut structs::Pages, database: &structs::LoadedDatabase, server: &structs::Server, title: impl Into<RichText>, script: impl ToString) {
    let button = ui.add(Button::new(title.into()));

    if button.clicked() {
        pages.pages.push(structs::Page {
            title: String::from(format!("{} ({}:{})", database.name, server.ip, server.port)),
            page_type: structs::PageType::SQLQuery(structs::SQLQueryPage {
                database: database.database.clone(),
                code: script.to_string(),
                code_file_path: None,
                sql_query_execution_status: None,
            }),
            ..Default::default()
        });

        pages.current_page_index = (pages.pages.len() - 1) as u16;
    }
}
