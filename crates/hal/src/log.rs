//! Kernel logging system with levels and module filtering.

extern crate alloc;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use spin::Mutex;

/// Log levels (lower = more severe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN ",
            Self::Info => "INFO ",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "error" | "ERROR" => Some(Self::Error),
            "warn" | "WARN" => Some(Self::Warn),
            "info" | "INFO" => Some(Self::Info),
            "debug" | "DEBUG" => Some(Self::Debug),
            "trace" | "TRACE" => Some(Self::Trace),
            _ => None,
        }
    }
}

/// Global minimum log level (messages below this are dropped).
static MIN_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

/// Module filter: when set, only this module's logs are shown.
/// Empty = show all modules.
static MODULE_FILTER: Mutex<[u8; 32]> = Mutex::new([0u8; 32]);
static MODULE_FILTER_LEN: AtomicUsize = AtomicUsize::new(0);

/// Log entry in the ring buffer.
#[derive(Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub module: [u8; 16],
    pub module_len: usize,
    pub message: [u8; 128],
    pub message_len: usize,
    pub tick: u64,
}

impl LogEntry {
    const fn empty() -> Self {
        Self {
            level: LogLevel::Info,
            module: [0; 16],
            module_len: 0,
            message: [0; 128],
            message_len: 0,
            tick: 0,
        }
    }
    pub fn module_str(&self) -> &str {
        core::str::from_utf8(&self.module[..self.module_len]).unwrap_or("?")
    }
    pub fn message_str(&self) -> &str {
        core::str::from_utf8(&self.message[..self.message_len]).unwrap_or("?")
    }
}

/// Ring buffer for recent log entries.
const LOG_CAPACITY: usize = 256;
static LOG_BUFFER: Mutex<LogRing> = Mutex::new(LogRing::new());

struct LogRing {
    entries: [LogEntry; LOG_CAPACITY],
    head: usize,
    count: usize,
}

impl LogRing {
    const fn new() -> Self {
        const EMPTY: LogEntry = LogEntry::empty();
        Self {
            entries: [EMPTY; LOG_CAPACITY],
            head: 0,
            count: 0,
        }
    }
    fn push(&mut self, entry: LogEntry) {
        self.entries[self.head] = entry;
        self.head = (self.head + 1) % LOG_CAPACITY;
        if self.count < LOG_CAPACITY {
            self.count += 1;
        }
    }
    fn recent(&self, n: usize) -> impl Iterator<Item = &LogEntry> {
        let n = n.min(self.count);
        let start = if self.count < LOG_CAPACITY {
            self.count - n
        } else {
            (self.head + LOG_CAPACITY - n) % LOG_CAPACITY
        };
        (0..n).map(move |i| &self.entries[(start + i) % LOG_CAPACITY])
    }
}

/// Sets the minimum log level.
pub fn set_level(level: LogLevel) {
    MIN_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Gets the current minimum log level.
pub fn current_level() -> LogLevel {
    match MIN_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

/// Sets the module filter (empty string = show all).
pub fn set_module_filter(module: &str) {
    let mut filter = MODULE_FILTER.lock();
    let len = module.len().min(31);
    filter[..len].copy_from_slice(&module.as_bytes()[..len]);
    MODULE_FILTER_LEN.store(len, Ordering::Relaxed);
}

/// Core log function — called by the klog! macro.
pub fn log(level: LogLevel, module: &str, message: &str) {
    if (level as u8) > MIN_LEVEL.load(Ordering::Relaxed) {
        return;
    }

    let filter_len = MODULE_FILTER_LEN.load(Ordering::Relaxed);
    if filter_len > 0 {
        let filter = MODULE_FILTER.lock();
        let filter_str = core::str::from_utf8(&filter[..filter_len]).unwrap_or("");
        if filter_str != "all" && module != filter_str {
            return;
        }
    }

    let mut entry = LogEntry::empty();
    entry.level = level;
    let mlen = module.len().min(15);
    entry.module[..mlen].copy_from_slice(&module.as_bytes()[..mlen]);
    entry.module_len = mlen;
    let msglen = message.len().min(127);
    entry.message[..msglen].copy_from_slice(&message.as_bytes()[..msglen]);
    entry.message_len = msglen;
    entry.tick = crate::interrupts::tick_count();

    LOG_BUFFER.lock().push(entry.clone());

    crate::serial::_serial_print(format_args!(
        "[{}] [{}] {}\n",
        entry.level.as_str(),
        module,
        message
    ));
}

/// Returns recent log entries.
pub fn recent_logs(count: usize) -> alloc::vec::Vec<LogEntry> {
    let ring = LOG_BUFFER.lock();
    ring.recent(count).cloned().collect()
}

/// The klog! macro for structured kernel logging.
#[macro_export]
macro_rules! klog {
    ($level:ident, $module:expr, $($arg:tt)*) => {{
        let msg = alloc::format!($($arg)*);
        $crate::log::log($crate::log::LogLevel::$level, $module, &msg);
    }};
}
