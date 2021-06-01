#[macro_use]
extern crate cascade;

mod app;
mod utils;
mod widgets;

use gio::prelude::*;
use self::app::App;

const APP_ID: &str = "io.github.mmstick.AppsRUs";

#[derive(Debug)]
pub enum Event {
    Search,
}

fn main() {
    let app = gtk::Application::new(Some(APP_ID), Default::default())
        .expect("failed to init application");

    app.connect_activate(|app| {
        let (tx, rx) = smol::channel::unbounded();

        let mut app = App::new(app, tx);

        let event_handler = async move {
            app.refresh_database().await;

            while let Ok(event) = rx.recv().await {
                let start = std::time::SystemTime::now();

                match event {
                    Event::Search => app.search().await
                }

                let elapsed = std::time::SystemTime::now();
                println!("Took {:?} to handle event {:?}", elapsed.duration_since(start), event);
            }
        };

        utils::spawn(event_handler);
    });

    app.run(&[]);
}
