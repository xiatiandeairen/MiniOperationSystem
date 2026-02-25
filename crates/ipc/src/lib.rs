//! Inter-process communication subsystem for MiniOS.
//!
//! Provides message queues for passing data between processes.
//! Up to 16 queues can exist simultaneously.

#![no_std]

extern crate alloc;

pub mod manager;
pub mod queue;

pub use manager::IpcManagerImpl;
pub use queue::Message;

use spin::Mutex;

/// Global IPC manager instance.
pub static IPC_MANAGER: Mutex<IpcManagerImpl> = Mutex::new(IpcManagerImpl::new());

/// Initialises the IPC subsystem and returns a reference description.
///
/// Currently a no-op since the global manager is const-initialised,
/// but kept as an explicit init point for future setup work.
pub fn init() {
    // Manager is already initialised via const fn; nothing to do.
}
