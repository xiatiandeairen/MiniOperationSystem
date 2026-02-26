//! Device driver contract.

use crate::error::DriverError;
use crate::types::DeviceType;

/// Uniform interface for character and block devices.
pub trait DeviceDriver: Send + Sync {
    /// Returns the human-readable device name.
    fn name(&self) -> &str;
    /// Returns the device classification.
    fn device_type(&self) -> DeviceType;
    /// Initializes the device hardware.
    fn init(&self) -> Result<(), DriverError>;
    /// Reads bytes from the device into `buf`.
    fn read(&self, buf: &mut [u8]) -> Result<usize, DriverError>;
    /// Writes bytes from `buf` to the device.
    fn write(&self, buf: &[u8]) -> Result<usize, DriverError>;
    /// Performs a device-specific control operation.
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize, DriverError>;
}
