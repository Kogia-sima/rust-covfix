#![allow(dead_code)]

use rand::seq::SliceRandom;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(not(windows))]
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets");

#[cfg(windows)]
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "\\tests\\assets");

#[derive(Debug)]
pub struct WorkSpace {
    dir: PathBuf,
}

impl WorkSpace {
    pub fn from_template(name: &str) -> WorkSpace {
        let tmproot = env::temp_dir();
        let target_dir = tempdir_path(tmproot, env!("CARGO_PKG_NAME"));
        let template_dir = Path::new(ASSETS_DIR).join(name);

        copy_dir::copy_dir(&template_dir, &target_dir).unwrap();

        WorkSpace { dir: target_dir }
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }

    fn close(&self) {
        if self.dir.exists() {
            fs::remove_dir_all(&self.dir).unwrap();
        }
    }
}

impl Drop for WorkSpace {
    fn drop(&mut self) {
        self.close();
    }
}

const NUM_RETRIES: usize = 100;
const NUM_RAND_CHARS: usize = 12;
const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn tempdir_path(mut path: PathBuf, prefix: &str) -> PathBuf {
    if !path.is_absolute() {
        let cur_dir = env::current_dir().unwrap();
        path = cur_dir.join(path);
        // return TempDir::new_in(&cur_dir.join(tmpdir), prefix);
    }

    let mut dirname = String::with_capacity(prefix.len() + NUM_RAND_CHARS + 1);
    dirname.push_str(prefix);
    dirname.push('-');
    let mut dirname = dirname.into_bytes();

    let mut rng = rand::thread_rng();
    for _ in 0..NUM_RETRIES {
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, NUM_RAND_CHARS)
            .for_each(|&x| dirname.push(x));

        unsafe {
            path.push(&std::str::from_utf8_unchecked(&dirname));
        }

        if !path.exists() {
            return path;
        } else {
            path.pop();
            dirname.truncate(prefix.len() + 1);
        }
    }

    panic!("failed to create temporary directory.");
}
