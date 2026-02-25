//! Interactive terminal shell for MiniOS.

#![no_std]

extern crate alloc;

mod commands;
mod input;
mod parser;
mod shell;

pub use shell::run_shell;
