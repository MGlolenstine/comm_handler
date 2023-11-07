type Result<T> = std::result::Result<T, crate::error::Error>;

/// Handler that takes care of the adapters
mod handler;
pub use handler::Handler;

/// Handler that takes care of the adapters and transforms bytes into packets
mod framed_handler;
pub use framed_handler::FramedHandler;

/// Errors that can occur
pub mod error;

/// Adapters store implemented Communication traits for adapters
pub mod adapters;

/// Traits for implementing Adapters and PacketParsers
pub mod traits;
