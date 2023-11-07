type Result<T> = std::result::Result<T, crate::error::Error>;

/// Communication stores communication trait
pub mod communication;

/// Adapters store implemented Communication traits for adapters
pub mod adapters;

/// Handler that takes care of the adapters
pub mod handler;

/// Errors that can occur
pub mod error;
