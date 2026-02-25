//! Device driver contract.

use crate::error::DriverError;
use crate::types::DeviceType;

/// Uniform interface for character and block devices.
pub trait DeviceDriver: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&self) -> Result<(), DriverError>;
    fn read(&self, buf: &mut [u8]) -> Result<usize, DriverError>;
    fn write(&self, buf: &[u8]) -> Result<usize, DriverError>;
    fn ioctl(&self, cmd: u32, arg: usize) -> Result<usize, DriverError>;
}
