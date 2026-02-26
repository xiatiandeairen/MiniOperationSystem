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

#[cfg(test)]
mod tests {
    use super::*;
    use minios_common::error::IpcError;

    #[test]
    fn send_and_receive() {
        let mut q = MessageQueue::new(4);
        let msg = Message::new(Pid(0), 1, b"hello");
        q.send(msg).unwrap();
        let recv = q.receive().unwrap();
        assert_eq!(recv.sender, Pid(0));
        assert_eq!(recv.msg_type, 1);
        assert_eq!(&recv.data[..recv.data_len], b"hello");
    }

    #[test]
    fn fifo_order() {
        let mut q = MessageQueue::new(4);
        q.send(Message::new(Pid(0), 1, b"first")).unwrap();
        q.send(Message::new(Pid(0), 2, b"second")).unwrap();
        q.send(Message::new(Pid(0), 3, b"third")).unwrap();
        let a = q.receive().unwrap();
        let b = q.receive().unwrap();
        let c = q.receive().unwrap();
        assert_eq!(&a.data[..a.data_len], b"first");
        assert_eq!(&b.data[..b.data_len], b"second");
        assert_eq!(&c.data[..c.data_len], b"third");
    }

    #[test]
    fn full_queue_returns_error() {
        let mut q = MessageQueue::new(2);
        q.send(Message::new(Pid(0), 0, b"a")).unwrap();
        q.send(Message::new(Pid(0), 0, b"b")).unwrap();
        let r = q.send(Message::new(Pid(0), 0, b"c"));
        assert!(matches!(r, Err(IpcError::QueueFull)));
    }

    #[test]
    fn empty_queue_returns_error() {
        let mut q = MessageQueue::new(2);
        let r = q.receive();
        assert!(matches!(r, Err(IpcError::QueueEmpty)));
    }

    #[test]
    fn message_new_truncates() {
        let payload: [u8; 300] = [42; 300];
        let msg = Message::new(Pid(1), 0, &payload);
        assert_eq!(msg.data_len, MAX_MSG_DATA);
        assert_eq!(msg.data[0], 42);
        assert_eq!(msg.data[MAX_MSG_DATA - 1], 42);
    }
}
