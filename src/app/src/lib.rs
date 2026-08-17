// todo
// https://rust-lang.github.io/rust-clippy/master/index.html#/result_large_err
#![allow(clippy::result_large_err)]

mod ctx;
pub mod depends;
pub mod helper;
pub mod version;
pub mod wizard;
pub mod workflows;

pub use ctx::*;

pub mod butler;
pub mod extract;
pub mod flags;
pub mod gui;
pub mod logger;

pub mod aria2;

#[macro_use]
extern crate rust_i18n;
i18n!();