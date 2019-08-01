use std::{
    collections::HashSet,
    fs,
    os::unix::fs::MetadataExt, // TODO windows compatibility...
    path::{PathBuf},
};

pub struct FileInfo {
    pub path: PathBuf,
    pub file_count: u64,
    pub size: u64,
    pub is_dir: bool,
}
impl FileInfo {
    /// implements a very crude file walker (much could be optimized)
    pub fn from_dir(path: PathBuf) -> FileInfo {
        let mut file_count = 1;
        let mut size = 0;
        let mut inodes = HashSet::<u64>::default(); // to avoid counting twice an inode
        let mut dirs: Vec<PathBuf> = Vec::new();
        dirs.push(path.clone());
        while let Some(dir) = dirs.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for e in entries.flatten() {
                    file_count += 1;
                    if let Ok(md) = e.metadata() {
                        if md.is_dir() {
                            dirs.push(e.path());
                        } else if md.nlink() > 1 {
                            if !inodes.insert(md.ino()) {
                                // it was already in the set
                                continue; // let's not add the size
                            }
                        }
                        size += md.len();
                    }
                }
            }
        }
        FileInfo {
            path,
            file_count,
            size,
            is_dir: true,
        }
    }
}


