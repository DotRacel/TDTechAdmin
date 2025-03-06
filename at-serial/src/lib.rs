//! A library for communicating with AT command-based serial devices.
//!
//! This library provides abstractions for working with serial devices 
//! that use AT commands for communication.

mod error;
mod serial;
mod terminal;

pub use error::AtError;
pub use serial::{SerialConfig, SerialConnection};
pub use terminal::{AtCommandTerminal, ReceivedData, DataHandler, SimpleDataHandler};

/// Re-export types from serialport that are used in our public API
pub use serialport::FlowControl;