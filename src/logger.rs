#![allow(unused_macros)]

use std::sync::atomic::{self, Ordering};

static VERBOSITY: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

pub fn set_verbosity(verbosity: usize) {
    VERBOSITY.store(verbosity, Ordering::SeqCst);
}

pub fn get_verbosity() -> usize {
    VERBOSITY.load(Ordering::SeqCst)
}

#[doc(hidden)]
#[macro_export]
macro_rules! errorln {
    ($($arg:tt)*) => {
        if $crate::get_verbosity() >= 1 {
            eprintln!($($arg)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! warnln {
    ($($arg:tt)*) => {
        if $crate::get_verbosity() >= 2 {
            eprintln!($($arg)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! infoln {
    ($($arg:tt)*) => {
        if $crate::get_verbosity() >= 3 {
            eprintln!($($arg)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => {
        if $crate::get_verbosity() >= 4 {
            eprintln!($($arg)*);
        }
    };
}
