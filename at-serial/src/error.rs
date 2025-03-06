//! Error types for the AT command library.

use std::io;
use thiserror::Error;

/// Errors that can occur in the AT command library.
#[derive(Error, Debug)]
pub enum AtError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Failed to open serial port.
    #[error("Failed to open serial port: {0}")]
    PortOpenError(String),

    /// Serial port connection was lost.
    #[error("Serial port connection lost: {0}")]
    ConnectionLost(String),

    /// AT command error.
    #[error("AT command error: {0}")]
    CommandError(String),

    /// Other errors.
    #[error("{0}")]
    Other(String),
}

impl From<serialport::Error> for AtError {
    fn from(error: serialport::Error) -> Self {
        AtError::PortOpenError(error.to_string())
    }
}