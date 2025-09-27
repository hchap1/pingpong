use std::{fs::create_dir_all, io::ErrorKind, path::{Path, PathBuf}};

use directories::ProjectDirs;

use crate::error::{Error, Res};

#[derive(Clone, Debug)]
pub struct Directory {
    root: PathBuf
}

impl Directory {
    pub fn create_or_load() -> Res<Self> {
        let root = match ProjectDirs::from("com", "hchap1", "pingpong") {
            Some(dir_builder) => dir_builder.data_dir().to_path_buf(),
            None => return Err(Error::FailedToFindLocation)
        };

        let error = Error::FailedToFindLocation;
        let _ = create_dir_all(&root);
        if !root.exists() { return Err(error); }

        Ok(Self { root })
    }

    pub fn get(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn get_ref(&self) -> &Path {
        self.root.as_path()
    }
}
