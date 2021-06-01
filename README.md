# Apps"R"Us

Experimental side project to create an efficient software center for Linux in Rust.

## Goals

- Must be very resource-conscious, with no delay in the UI
- No reliance on PackageKit backends for fetching application data
- Appstream data and icons are stored in a flash-friendly zstd-compressed embedded database
- ECS model to data storage and retrieval
- Support for reading DEP11 YAML appstream sources
- Support for reading Flatpak XML appstream sources
- Instantly reload the cache when package lists are updated
