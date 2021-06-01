use std::env;

use crate::PackageEvent;
use std::path::PathBuf;
use smol::channel::Sender;

const LOCAL: &str = ".local/share/flatpak/appstream/";

pub fn fetch<'a>(executor: &smol::LocalExecutor<'a>, tx: Sender<PackageEvent>) -> anyhow::Result<()> {
    let local_appstream = env::home_dir().unwrap().join(LOCAL);

    for repo_entry in local_appstream.read_dir().unwrap() {
        if let Ok(repo) = repo_entry {
            for arch_entry in repo.path().read_dir().unwrap() {
                if let Ok(arch) = arch_entry {
                    let appstream = arch.path().join("active/appstream.xml");

                    executor.spawn(read_xml(appstream, tx.clone())).detach();
                }
            }
        }
    }

    Ok(())
}

pub async fn read_xml(path: PathBuf, tx: Sender<PackageEvent>) {
    // TODO:
}