mod app_listing;

pub use self::app_listing::AppListing;

use gtk::prelude::*;

pub fn embedded_png(icon: &[u8], size: i32) -> Option<gtk::DrawingArea> {
    let surface = cairo::ImageSurface::create_from_png(&mut std::io::Cursor::new(icon)).ok()?;
    Some(cascade! {
        gtk::DrawingArea::new();
        ..set_size_request(size, size);
        ..connect_draw(move |_, ctx| {
            ctx.set_source_surface(&surface, 0.0, 0.0);
            ctx.paint();
            gtk::Inhibit(false)
        });
    })
}