//! Serial port driver for COM1 using the 16550 UART.

use core::fmt;
use spin::Mutex;
use uart_16550::SerialPort;

/// COM1 serial port at I/O address `0x3F8`.
///
/// The port must be initialised via [`init`] before first use.
// SAFETY: 0x3F8 is the standard COM1 I/O port address on x86 systems.
pub static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

/// Initialises the COM1 serial port hardware.
pub fn init() {
    SERIAL1.lock().init();
}

/// Writes formatted arguments to the serial port.
///
/// This is the implementation detail behind [`serial_print!`] and
/// [`serial_println!`]. Interrupts are disabled for the duration of the
/// write to prevent deadlocks.
#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    use fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("serial print failed");
    });
}

/// Prints to the COM1 serial port.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_serial_print(format_args!($($arg)*))
    };
}

/// Prints to the COM1 serial port, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
