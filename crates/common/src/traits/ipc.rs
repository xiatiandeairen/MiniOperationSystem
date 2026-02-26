//! Inter-process communication contract.

use crate::error::IpcError;
use crate::id::{Pid, QueueId, ShmId};

/// Message passing and shared memory management.
pub trait IpcManager: Send + Sync {
    /// Creates a named message queue with the given capacity.
    fn create_queue(&self, name: &str, capacity: usize) -> Result<QueueId, IpcError>;
    /// Sends `data` to the specified queue.
    fn send(&self, queue: QueueId, data: &[u8]) -> Result<(), IpcError>;
    /// Receives data from the queue into `buf`.
    fn receive(&self, queue: QueueId, buf: &mut [u8]) -> Result<usize, IpcError>;
    /// Destroys a message queue.
    fn destroy_queue(&self, queue: QueueId) -> Result<(), IpcError>;
    /// Creates a named shared memory region.
    fn create_shm(&self, name: &str, size: usize) -> Result<ShmId, IpcError>;
    /// Attaches a shared memory region to a process's address space.
    fn attach_shm(&self, shm: ShmId, pid: Pid) -> Result<u64, IpcError>;
    /// Detaches a shared memory region from a process.
    fn detach_shm(&self, shm: ShmId, pid: Pid) -> Result<(), IpcError>;
}
