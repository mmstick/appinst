use appstream_cache::Database;
use crate::Event;
use crate::utils;
use crate::widgets::AppListing;
use gtk::prelude::*;
use smol::channel::Sender;
use std::collections::HashMap;
use std::rc::Rc;

struct AppMeta {
    pub discovered: HashMap<Rc<str>, u32>,
}

pub struct App {
    list: gtk::ListBox,
    search: gtk::SearchEntry,

    // Where all the appstream-related information is stored
    db: Database,

    // Send events to the application's event handler
    tx: Sender<Event>,

    app_list: HashMap<String, AppMeta>
}

impl App {
    pub fn new(app: &gtk::Application, tx: Sender<Event>) -> Self {
        let db = Database::new("target/db".into(), "en_US".to_owned());

        let list = gtk::ListBox::new();
        list.show();

        let scroller = gtk::ScrolledWindowBuilder::new()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();

        scroller.add(&list);

        let search = cascade! {
            gtk::SearchEntry::new();
            ..connect_changed(glib::clone!(@strong tx => move |_| {
                let tx = tx.clone();
                utils::block_on(async move {
                    let _ = tx.send(Event::Search).await;
                });
            }));
            ..show();
        };

        let container = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 8);
            ..add(&search);
            ..add(&scroller);
        };

        let _window = cascade! {
            gtk::ApplicationWindow::new(app);
            ..add(&container);
            ..set_title("Apps \"R\" Us");
            ..connect_delete_event(move |_win, _| {
                gtk::Inhibit(false)
            });
            ..show_all();
        };

        Self { db, search, list, tx, app_list: HashMap::new() }
    }

    pub async fn refresh_database(&mut self) {
        let _ = self.db.refresh_appstream_components().await;
    }

    pub async fn search(&mut self) {
        for child in self.list.get_children() {
            self.list.remove(&child);
        }

        self.app_list.clear();

        let text = self.search.get_text();

        if text.len() < 2 { return; }

        let mut entries = self.db.search_for(&text).await.into_iter();

        while let Some((origin_name, entity, name)) = entries.next() {
            let origin = self.db.get_origin(&origin_name);

            let id = origin.id(entity);
            let icon = origin.icon(entity);
            let summary = origin.summary(entity);

            // TODO: Improve the ergonomics of this
            let data = id.and_then(|n| icon.map(|i| (n, i)))
                .and_then(|v| summary.map(|s| (v, s)));

            if let Some(((id, icon), summary)) = data {
                let mut add_app = false;
                self.app_list.entry(id)
                    .or_insert_with(|| {
                        add_app = true;
                        AppMeta { discovered: HashMap::default() }
                    })
                    .discovered
                    .insert(origin_name, entity);

                if add_app {
                    self.add_app(&name, &icon, &summary);
                }
            }
        }
    }

    pub fn add_app(&mut self, name: &str, icon: &str, summary: &str) {
        if let Ok(Some(img)) = self.db.icons.open_tree("48x48").unwrap().get(icon.as_bytes()) {
            let listing = AppListing::new(name, summary, &img);
            self.list.add(&listing.container);
        }
    }
}