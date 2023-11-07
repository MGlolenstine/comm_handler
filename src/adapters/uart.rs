use crate::communication::Communication;

pub struct UartAdapterConfiguration {

}

pub struct UartAdapter {
    port: serial2::SerialPort,
}

impl Communication for UartAdapter {
    type CommConfig = UartAdapterConfiguration;

    fn connect(&mut self, config: Self::CommConfig) -> Result<bool, std::io::Error> {
        todo!()
    }

    fn disconnect(&mut self) -> Result<bool, std::io::Error> {
        todo!()
    }

    fn connected(&mut self) -> bool {
        todo!()
    }

    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        todo!()
    }

    fn recv(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        todo!()
    }
}