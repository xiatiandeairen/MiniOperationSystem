//! Circular-buffer message queue for inter-process communication.

extern crate alloc;

use alloc::vec::Vec;
use minios_common::error::IpcError;
use minios_common::id::Pid;

/// Maximum data payload in a single message.
pub const MAX_MSG_DATA: usize = 256;

/// A single IPC message.
#[derive(Clone)]
pub struct Message {
    /// PID of the sending process.
    pub sender: Pid,
    /// Application-defined message type tag.
    pub msg_type: u32,
    /// Payload bytes.
    pub data: [u8; MAX_MSG_DATA],
    /// Number of valid bytes in `data`.
    pub data_len: usize,
}

impl Message {
    /// Creates a new message with the given sender, type, and payload.
    pub fn new(sender: Pid, msg_type: u32, payload: &[u8]) -> Self {
        let mut data = [0u8; MAX_MSG_DATA];
        let len = payload.len().min(MAX_MSG_DATA);
        data[..len].copy_from_slice(&payload[..len]);
        Self {
            sender,
            msg_type,
            data,
            data_len: len,
        }
    }
}

/// Fixed-capacity circular buffer of [`Message`] values.
pub struct MessageQueue {
    buf: Vec<Option<Message>>,
    capacity: usize,
    head: usize,
    tail: usize,
    count: usize,
}

impl MessageQueue {
    /// Creates an empty queue with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        let mut buf = Vec::with_capacity(capacity);
        buf.resize_with(capacity, || None);
        Self {
            buf,
            capacity,
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Enqueues a message. Returns [`IpcError::QueueFull`] if at capacity.
    pub fn send(&mut self, msg: Message) -> Result<(), IpcError> {
        if self.count >= self.capacity {
            return Err(IpcError::QueueFull);
        }
        self.buf[self.tail] = Some(msg);
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        Ok(())
    }

    /// Dequeues the oldest message. Returns [`IpcError::QueueEmpty`] if empty.
    pub fn receive(&mut self) -> Result<Message, IpcError> {
        if self.count == 0 {
            return Err(IpcError::QueueEmpty);
        }
        let msg = self.buf[self.head].take().ok_or(IpcError::QueueEmpty)?;
        self.head = (self.head + 1) % self.capacity;
        self.count -= 1;
        Ok(msg)
    }

    /// Returns the number of messages currently in the queue.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
