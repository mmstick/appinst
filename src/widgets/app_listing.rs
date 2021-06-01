use gtk::prelude::*;
use super::embedded_png;

pub struct AppListing {
    pub container: gtk::Grid,
}

impl AppListing {
    pub fn new(name: &str, summary: &str, icon: &[u8]) -> Self {
        let name = gtk::LabelBuilder::new()
            .xalign(0.0)
            .label(name)
            .build();

        let summary = gtk::LabelBuilder::new()
            .xalign(0.0)
            .label(summary)
            .build();

        let container = cascade! {
            let grid = gtk::Grid::new();
            ..set_column_spacing(4);
            ..set_row_spacing(4);
            if let Some(png) = embedded_png(icon, 48) {
                grid.attach(&png, 0, 0, 1, 2);
            };
            ..attach(&name, 1, 0, 1, 1);
            ..attach(&summary, 1, 1, 1, 1);
            ..show_all();
        };

        Self { container }
    }
}