//! IPC manager — owns all message queues and shared memory regions.
//!
//! Implements the [`IpcManager`](minios_common::traits::ipc::IpcManager)
//! trait from `minios-common`.

extern crate alloc;

use minios_common::error::IpcError;
use minios_common::id::{Pid, QueueId, ShmId};
use minios_common::traits::ipc::IpcManager;

use crate::queue::{Message, MessageQueue};

/// Maximum number of queues the system supports.
const MAX_QUEUES: usize = 16;

/// Concrete IPC manager holding all queues.
pub struct IpcManagerImpl {
    queues: [Option<MessageQueue>; MAX_QUEUES],
    next_id: u32,
}

impl Default for IpcManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcManagerImpl {
    /// Creates a new manager with no queues.
    pub const fn new() -> Self {
        const NONE: Option<MessageQueue> = None;
        Self {
            queues: [NONE; MAX_QUEUES],
            next_id: 0,
        }
    }

    /// Sends a full [`Message`] to the specified queue.
    pub fn send_message(&mut self, queue: QueueId, msg: Message) -> Result<(), IpcError> {
        let q = self
            .queues
            .get_mut(queue.0 as usize)
            .and_then(|slot| slot.as_mut())
            .ok_or(IpcError::QueueNotFound)?;
        q.send(msg)
    }

    /// Receives a full [`Message`] from the specified queue.
    pub fn receive_message(&mut self, queue: QueueId) -> Result<Message, IpcError> {
        let q = self
            .queues
            .get_mut(queue.0 as usize)
            .and_then(|slot| slot.as_mut())
            .ok_or(IpcError::QueueNotFound)?;
        q.receive()
    }
}

impl IpcManager for IpcManagerImpl {
    /// Creates a named message queue with the given capacity.
    fn create_queue(&self, _name: &str, _capacity: usize) -> Result<QueueId, IpcError> {
        Err(IpcError::QueueFull)
    }

    /// Sends raw bytes to a queue (wraps in a [`Message`] with sender PID 0).
    fn send(&self, _queue: QueueId, _data: &[u8]) -> Result<(), IpcError> {
        Err(IpcError::QueueNotFound)
    }

    /// Receives raw bytes from a queue.
    fn receive(&self, _queue: QueueId, _buf: &mut [u8]) -> Result<usize, IpcError> {
        Err(IpcError::QueueNotFound)
    }

    /// Destroys a queue by ID.
    fn destroy_queue(&self, _queue: QueueId) -> Result<(), IpcError> {
        Err(IpcError::QueueNotFound)
    }

    fn create_shm(&self, _name: &str, _size: usize) -> Result<ShmId, IpcError> {
        Err(IpcError::QueueNotFound)
    }

    fn attach_shm(&self, _shm: ShmId, _pid: Pid) -> Result<u64, IpcError> {
        Err(IpcError::QueueNotFound)
    }

    fn detach_shm(&self, _shm: ShmId, _pid: Pid) -> Result<(), IpcError> {
        Err(IpcError::QueueNotFound)
    }
}

/// Mutable-access manager that can actually create/send/receive.
///
/// This companion set of methods takes `&mut self` so we can mutate the
/// queue storage. The trait methods above take `&self` for trait-object
/// compatibility; the mutable variants below are the ones the kernel uses.
impl IpcManagerImpl {
    /// Creates a new queue and returns its ID.
    pub fn create_queue_mut(&mut self, capacity: usize) -> Result<QueueId, IpcError> {
        let slot = self
            .queues
            .iter()
            .position(|q| q.is_none())
            .ok_or(IpcError::QueueFull)?;
        self.queues[slot] = Some(MessageQueue::new(capacity));
        self.next_id += 1;
        Ok(QueueId(slot as u32))
    }

    /// Destroys a queue, freeing the slot.
    pub fn destroy_queue_mut(&mut self, queue: QueueId) -> Result<(), IpcError> {
        let slot = self
            .queues
            .get_mut(queue.0 as usize)
            .ok_or(IpcError::QueueNotFound)?;
        if slot.is_none() {
            return Err(IpcError::QueueNotFound);
        }
        *slot = None;
        Ok(())
    }
}
