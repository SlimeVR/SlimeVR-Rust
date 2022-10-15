//! Contains all the error types used in the program.

/// An error that the Jig detected in the hardware under test.
pub enum HardwareError {
    /// Failed to flash the PCB
    NoFlash,
    /// Failed to establish serial connection to PCB after flashing
    NoConnectSerial,
    /// Failed to get expected serial outputs from PCB
    UnexpectedSerial,
}
