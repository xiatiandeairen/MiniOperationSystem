//! Circular-buffer message queue for inter-process communication.

extern crate alloc;

use alloc::vec::Vec;
use minios_common::error::IpcError;
use minios_common::id::Pid;

/// Maximum data payload in a single message.
pub const MAX_MSG_DATA: usize = 256;

/// A single IPC message carrying optional trace context for cross-process linking.
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
    /// Trace context from the sender, enabling cross-process trace chain linking.
    /// When present, the receiver can call `tracer.set_context()` to continue
    /// the same trace chain, making the full send→receive path visible in the
    /// trace viewer as a single connected trace.
    pub trace_context: Option<minios_common::types::TraceContext>,
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
            trace_context: None,
        }
    }

    /// Creates a message with an attached trace context for cross-process tracing.
    pub fn with_trace(
        sender: Pid,
        msg_type: u32,
        payload: &[u8],
        ctx: minios_common::types::TraceContext,
    ) -> Self {
        let mut msg = Self::new(sender, msg_type, payload);
        msg.trace_context = Some(ctx);
        msg
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

    #[test]
    fn message_clone_preserves_data() {
        let msg = Message::new(Pid(5), 99, b"payload");
        let copy = msg.clone();
        assert_eq!(copy.sender, Pid(5));
        assert_eq!(copy.msg_type, 99);
        assert_eq!(copy.data_len, 7);
        assert_eq!(&copy.data[..copy.data_len], b"payload");
    }

    #[test]
    fn queue_capacity_boundary() {
        let mut q = MessageQueue::new(1);
        assert!(q.send(Message::new(Pid(0), 0, b"a")).is_ok());
        assert!(matches!(
            q.send(Message::new(Pid(0), 0, b"b")),
            Err(IpcError::QueueFull)
        ));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn send_after_receive_frees_space() {
        let mut q = MessageQueue::new(2);
        q.send(Message::new(Pid(0), 0, b"a")).unwrap();
        q.send(Message::new(Pid(0), 0, b"b")).unwrap();
        assert!(q.send(Message::new(Pid(0), 0, b"c")).is_err());
        q.receive().unwrap();
        assert!(q.send(Message::new(Pid(0), 0, b"c")).is_ok());
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn is_empty_and_len() {
        let mut q = MessageQueue::new(4);
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
        q.send(Message::new(Pid(0), 0, b"x")).unwrap();
        assert!(!q.is_empty());
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn message_with_trace_context() {
        use minios_common::id::{SpanId, TraceId};
        use minios_common::types::TraceContext;

        let ctx = TraceContext {
            trace_id: TraceId(42),
            current_span_id: SpanId(7),
            depth: 1,
        };
        let msg = Message::with_trace(Pid(1), 10, b"hi", ctx);
        assert!(msg.trace_context.is_some());
        let tc = msg.trace_context.unwrap();
        assert_eq!(tc.trace_id, TraceId(42));
    }

    #[test]
    fn empty_payload_message() {
        let msg = Message::new(Pid(0), 0, b"");
        assert_eq!(msg.data_len, 0);
    }

    #[test]
    fn single_byte_payload() {
        let msg = Message::new(Pid(1), 5, &[0xFF]);
        assert_eq!(msg.data_len, 1);
        assert_eq!(msg.data[0], 0xFF);
    }

    #[test]
    fn circular_wrap_around() {
        let mut q = MessageQueue::new(3);
        q.send(Message::new(Pid(0), 0, b"a")).unwrap();
        q.send(Message::new(Pid(0), 0, b"b")).unwrap();
        q.receive().unwrap();
        q.receive().unwrap();
        q.send(Message::new(Pid(0), 0, b"c")).unwrap();
        q.send(Message::new(Pid(0), 0, b"d")).unwrap();
        q.send(Message::new(Pid(0), 0, b"e")).unwrap();
        assert_eq!(q.len(), 3);
        let m = q.receive().unwrap();
        assert_eq!(&m.data[..m.data_len], b"c");
    }

    #[test]
    fn max_payload_boundary() {
        let payload = [0xAA; MAX_MSG_DATA];
        let msg = Message::new(Pid(0), 0, &payload);
        assert_eq!(msg.data_len, MAX_MSG_DATA);
        assert_eq!(msg.data[MAX_MSG_DATA - 1], 0xAA);
    }
}
