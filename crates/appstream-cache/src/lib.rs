#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

pub mod dep11;
pub mod flatpak;
pub mod yaml;

pub use self::dep11::appstream::Dep11Package;
use std::rc::Rc;

use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashMap};

#[repr(u8)]
pub enum PackageEvent {
    Dep11 {
        origin: String,
        info: Dep11Package,
    },

    Dep11Icon {
        size: &'static str,
        name: String,
        buffer: Vec<u8>
    },

    MediaUrl {
        origin: String,
        base_url: String,
    }
}

const KEY_TYPES: &str = "types";
const KEY_IDS: &str = "ids";
const KEY_NAMES: &str = "names";
const KEY_ICONS: &str = "icons";
const KEY_PACKAGES: &str = "packages";
const KEY_SUMMARIES: &str = "summaries";
const KEY_DESCRIPTIONS: &str = "descriptions";
// const KEY_CATEGORIES: &str = "categories";
// const KEY_KEYWORDS: &str = "keywords";
// const KEY_LICENSES: &str = "licenses";
// const KEY_URLS: &str = "urls";
// const KEY_LAUNCHABLES: &str = "launchables";

pub type Entity = u32;

pub struct OriginDb {
    id: Entity,
    db: sled::Db,

    pub types: sled::Tree,
    pub ids: sled::Tree,
    pub names: sled::Tree,
    pub icons: sled::Tree,
    pub packages: sled::Tree,
    pub summaries: sled::Tree,
}

impl OriginDb {
    pub fn new(origin: &str, db: &Path) -> Self {
        let db = sled::open(db.join(origin)).unwrap();
        let _ = db.clear();

        Self {
            id: 0,
            types: db.open_tree(KEY_TYPES).unwrap(),
            ids: db.open_tree(KEY_IDS).unwrap(),
            names: db.open_tree(KEY_NAMES).unwrap(),
            icons: db.open_tree(KEY_ICONS).unwrap(),
            packages: db.open_tree(KEY_PACKAGES).unwrap(),
            summaries: db.open_tree(KEY_SUMMARIES).unwrap(),
            db
        }
    }

    pub fn set_media_url(&self, url: &str) {
        let _ = self.db.insert("media-url", url.as_bytes());
    }

    pub fn media_url(&self) -> Option<String> {
        self.db.get("media-url")
            .ok()
            .flatten()
            .and_then(|ivec| String::from_utf8(ivec.to_vec()).ok())
    }

    pub fn add_dep11_package(&mut self, package: Dep11Package, language: &str) {
        let id = &self.id.to_ne_bytes();
        self.id += 1;

        if let Some(name) = package.name.get(language).or_else(|| package.name.get("C")) {
            let _ = self.names.insert(name.as_bytes(), id);
            let _ = self.ids.insert(id, package.id.as_bytes());
            let _ = self.types.insert(id, package.type_.as_bytes());
            let _ = self.packages.insert(id, package.package.as_bytes());

            if let Some(summary) = package.summary.get(language).or_else(|| package.name.get("C")) {
                let _ = self.summaries.insert(id, summary.as_bytes());
            }

            if let Some(icon) = package.icon {
                let _ = self.icons.insert(id, bincode::serialize(&icon).unwrap());
            }
        }
    }

    pub fn id(&self, package: Entity) -> Option<String> {
        self.fetch_string(&self.ids, package)
    }

    pub fn icon(&self, package: Entity) -> Option<String> {
        if let Some(icon) = self.icons.get(&package.to_ne_bytes()).ok().flatten() {
            if let Ok(icon) = bincode::deserialize::<dep11::appstream::Icon>(&icon) {
                if let Some(cached) = icon.cached {
                    if let Some(cached) = cached.first() {
                        return Some(cached.name.clone());
                    }
                }
            }
        }

        None
    }

    pub fn summary(&self, package: Entity) -> Option<String> {
        self.fetch_string(&self.summaries, package)
    }

    pub fn iter(&self, mut fun: impl FnMut(Entity, &str)) {
        for (id, key) in self.names.iter().filter_map(Result::ok) {
            if let Ok(id) = std::str::from_utf8(&id) {
                let mut entity = [0u8; 4];
                entity.copy_from_slice(&key[..4]);
                let entity = u32::from_ne_bytes(entity);

                fun(entity, id)
            }
        }
    }

    fn fetch_string(&self, db: &sled::Tree, package: Entity) -> Option<String> {
        if let Some(ivec) = db.get(&package.to_ne_bytes()).ok().flatten() {
            if let Ok(string) = std::str::from_utf8(&ivec) {
                return Some(String::from(string))
            }
        }

        None
    }
}


pub struct Database {
    pub path: PathBuf,
    pub language: String,
    pub icons: sled::Db,
    pub origins: BTreeMap<String, OriginDb>,
}

impl Database {
    pub fn new(path: PathBuf, language: String) -> Self {
        let icons = sled::open(path.join("icons")).unwrap();
        Self { path, origins: BTreeMap::new(), icons, language }
    }

    pub fn get_origin(&mut self, origin: &str) -> &mut OriginDb {
        let path = self.path.clone();
        self.origins.entry(origin.to_owned())
            .or_insert_with(|| OriginDb::new(origin, &path))
    }

    pub async fn flush(&self) {
        for db in self.origins.values() {
            let _ = db.db.flush_async().await;
        }
    }

    pub async fn refresh_appstream_components(&mut self) -> anyhow::Result<()> {
        // Each package list is going to contain a stream of packages we'll collate
        let (tx, rx) = smol::channel::unbounded();

        // This executor shall spawn an I/O task for each package list, and then all information
        // will be converged into a singular location in our sled database. The purpose of doing
        // so is to make future queries for packages quick and efficient.
        let executor = &smol::LocalExecutor::new();

        executor.run(async move {
            dep11::fetch(executor, tx.clone())?;
            flatpak::fetch(executor, tx)?;

            let language = self.language.clone();

            while let Ok(event) = rx.recv().await {
                match event {
                    PackageEvent::Dep11 { origin, info } => {
                        self.get_origin(&origin).add_dep11_package(info, &language);
                    }

                    PackageEvent::Dep11Icon { size, name, buffer } => {
                        let _ = self.icons.open_tree(size).unwrap().insert(name.as_bytes(), buffer);
                    }

                    PackageEvent::MediaUrl { origin, base_url } => {
                        self.get_origin(&origin).set_media_url(&base_url);
                    }
                }
            }

            let _ = self.flush().await;

            Ok(())
        }).await
    }

    pub async fn search_for(&self, package: &str) -> Vec<(Rc<str>, Entity, String)> {
        let mut packages = Vec::new();

        for (origin, origin_db) in &self.origins {
            let origin: Rc<str> = Rc::from(origin.as_str());
            origin_db.iter(|entity, name| {
                if name.contains(package) {
                    packages.push((origin.clone(), entity, name.to_owned()));
                    return
                }
            });
        }

        packages
    }
}



