use crate::{adapters::uart::UartAdapter, Result};

pub trait CommunicationBuilder {
    /// Builds a communication out of
    fn build(&self) -> Result<Box<UartAdapter>>;
}

pub trait CloneableCommunication {
    fn boxed_clone(&self) -> Box<UartAdapter>;
}

pub trait Connectable: CommunicationBuilder {
    /// Connect to a device
    /// Here goes the implementation of how to connect to a device
    fn connect(&mut self) -> Result<bool>;

    /// Disconnect from a device
    /// Here goes the implementation of how to disconnect from a device
    fn disconnect(&mut self) -> Result<bool>;

    /// Check if the device is connected
    fn connected(&mut self) -> bool;
}

pub trait Communication: CloneableCommunication + Send + Sync {
    /// Send data to a device
    fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive data from a device
    fn recv(&mut self) -> Result<Option<Vec<u8>>>;
}
