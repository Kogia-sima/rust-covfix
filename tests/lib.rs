use fs2::FileExt;
use getrandom::getrandom;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const NUM_RETRIES: usize = 100;
const NUM_RAND_CHARS: usize = 12;
const ASCII_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEF";

#[derive(Debug)]
pub struct WorkSpace {
    path: PathBuf,
}

impl WorkSpace {
    pub fn new() -> Self {
        Self { path: tempdir() }
    }

    pub fn from_template<P: AsRef<Path>>(template_dir: P) -> Self {
        let obj = Self { path: tempdir() };
        copy_dir_contents(template_dir.as_ref(), &obj.path);
        obj
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn sync_to(&self, p: &Path) {
        copy_dir_contents(p, &self.path);
    }

    pub fn close(self) {
        drop(self)
    }
}

impl Drop for WorkSpace {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).unwrap_or_default();
        }
    }
}

struct LockedFile {
    fp: File,
}

impl LockedFile {
    pub fn new(fp: File) -> Self {
        fp.lock_exclusive().unwrap();
        Self { fp }
    }
}

impl Drop for LockedFile {
    fn drop(&mut self) {
        self.fp.unlock().unwrap();
    }
}

fn tempdir() -> PathBuf {
    let mut lock_path = env::temp_dir();
    lock_path.push(concat!(
        env!("CARGO_PKG_NAME"),
        "-",
        env!("CARGO_PKG_VERSION"),
        "_temp.lock"
    ));

    let lock_file = fs::File::create(lock_path).unwrap();
    let _ = LockedFile::new(lock_file);

    let p = unsafe { temppath() };
    fs::create_dir(&p).unwrap();

    p
}

// thread unsafe function
unsafe fn temppath() -> PathBuf {
    let mut path = env::temp_dir();

    if !path.is_absolute() {
        let cur_dir = env::current_dir().unwrap();
        path = cur_dir.join(path);
        // return TempDir::new_in(&cur_dir.join(tmpdir), prefix);
    }

    let mut dirname = String::with_capacity(5 + NUM_RAND_CHARS);
    dirname.push_str(".tmp.");
    dirname.push_str(std::str::from_utf8_unchecked(&[b'0'; NUM_RAND_CHARS]));
    let start_pos = dirname.len() - NUM_RAND_CHARS;

    for _ in 0..NUM_RETRIES {
        getrandom_ascii(&mut dirname.as_bytes_mut()[start_pos..]);

        path.push(&dirname);
        if !path.exists() {
            return path;
        } else {
            path.pop();
        }
    }

    panic!("failed to create temporary directory.");
}

unsafe fn getrandom_ascii(buf: &mut [u8]) {
    // maybe getrandom is thread unsafe?
    getrandom(buf).unwrap();

    for elem in buf {
        let idx = (*elem >> 3) as usize;
        *elem = ASCII_CHARS.as_bytes()[idx];
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
