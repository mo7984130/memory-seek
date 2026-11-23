use std::{path::Path, sync::LazyLock};

static PROJECT_DIRS: LazyLock<directories::ProjectDirs> = LazyLock::new(|| {
    directories::ProjectDirs::from("cn", "DriftCloud", "MemorySeek")
        .expect("无法获取合适的项目目录")
});

pub fn config_dir() -> &'static Path {
    PROJECT_DIRS.config_dir()
}

// pub fn data_dir() -> &'static Path {
//     PROJECT_DIRS.data_dir()
// }
