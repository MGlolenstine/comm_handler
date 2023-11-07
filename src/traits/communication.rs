pub trait CommunicationBuilder {
    /// Builds a communication out of
    fn build(&self) -> Result<Box<dyn Communication>, std::io::Error>;
}

pub trait CloneableCommunication {
    fn boxed_clone(&self) -> Box<dyn Communication>;
}

pub trait Connectable {
    /// Type that stores communication configuration
    type CommConfig;

    /// Connect to a device
    /// Here goes the implementation of how to connect to a device
    fn connect(&mut self, config: Self::CommConfig) -> Result<bool, std::io::Error>;

    /// Disconnect from a device
    /// Here goes the implementation of how to disconnect from a device
    fn disconnect(&mut self) -> Result<bool, std::io::Error>;

    /// Check if the device is connected
    fn connected(&mut self) -> bool;
}

pub trait Communication: CloneableCommunication + Send + Sync {
    /// Send data to a device
    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error>;

    /// Receive data from a device
    fn recv(&mut self) -> Result<Option<Vec<u8>>, std::io::Error>;
}