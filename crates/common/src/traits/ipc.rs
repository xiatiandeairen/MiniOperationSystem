//! Inter-process communication contract.

use crate::error::IpcError;
use crate::id::{Pid, QueueId, ShmId};

/// Message passing and shared memory management.
pub trait IpcManager: Send + Sync {
    fn create_queue(&self, name: &str, capacity: usize) -> Result<QueueId, IpcError>;
    fn send(&self, queue: QueueId, data: &[u8]) -> Result<(), IpcError>;
    fn receive(&self, queue: QueueId, buf: &mut [u8]) -> Result<usize, IpcError>;
    fn destroy_queue(&self, queue: QueueId) -> Result<(), IpcError>;
    fn create_shm(&self, name: &str, size: usize) -> Result<ShmId, IpcError>;
    fn attach_shm(&self, shm: ShmId, pid: Pid) -> Result<u64, IpcError>;
    fn detach_shm(&self, shm: ShmId, pid: Pid) -> Result<(), IpcError>;
}
