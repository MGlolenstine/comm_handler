type Result<T> = std::result::Result<T, crate::error::Error>;

/// Handler that takes care of the adapters
pub mod handler;

/// Handler that takes care of the adapters and transforms bytes into packets
pub mod framed_handler;

/// Errors that can occur
pub mod error;

/// Communication trait
pub mod communication;

/// Packet parser trait
pub mod packet_parser;

/// Adapters store implemented Communication traits for adapters
pub mod adapters;
