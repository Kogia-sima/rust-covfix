use std::fs;
use std::io;
use std::path::Path;
use tempfile::TempDir;

macro_rules! assert_matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => (),
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($pattern)+)),
        }
    }
}

#[derive(Debug)]
pub struct WorkSpace {
    dir: TempDir,
}

impl WorkSpace {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    pub fn from_template<P: AsRef<Path>>(template_dir: P) -> Self {
        let obj = Self::new();
        copy_dir_contents(template_dir.as_ref(), &obj.path());
        obj
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn sync_to(&self, p: &Path) {
        copy_dir_contents(p, self.path());
    }

    pub fn close(self) -> Result<(), io::Error> {
        self.dir.close()
    }
}

fn copy_dir_contents(p1: &Path, p2: &Path) {
    let mut target_path = p2.to_path_buf();

    for e in fs::read_dir(p1).unwrap() {
        let path = e.unwrap().path();
        let filename = path.file_name().unwrap();
        target_path.push(filename);

        if path.is_dir() {
            fs::create_dir(&target_path).unwrap();
            copy_dir_contents(&path, &target_path);
        } else {
            fs::copy(&path, &target_path).unwrap();
        }

        target_path.pop();
    }
}

// test modules
mod fix;
mod guess_game;
mod invalid_operations;
mod multiple_files;
mod read_lcov;
mod workspace;
mod write_lcov;
