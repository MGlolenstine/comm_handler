use crate::Result;
use std::{sync::Arc, time::Duration};

use log::{debug, trace};
use serial2::{CharSize, FlowControl, IntoSettings, Parity, SerialPort, StopBits};

use crate::traits::{CloneableCommunication, Communication, CommunicationBuilder};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UartAdapterConfiguration {
    port: String,
    pub baud_rate: u32,
    pub char_size: CharSize,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub read_timeout: Duration,
}

impl UartAdapterConfiguration {
    pub fn new<S: ToString>(port: S) -> Self {
        Self {
            port: port.to_string(),
            ..Default::default()
        }
    }
}

impl Default for UartAdapterConfiguration {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            char_size: CharSize::Bits8,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            read_timeout: Duration::from_millis(10),
        }
    }
}

impl IntoSettings for UartAdapterConfiguration {
    fn apply_to_settings(self, settings: &mut serial2::Settings) -> std::io::Result<()> {
        settings.set_baud_rate(self.baud_rate)?;
        settings.set_char_size(self.char_size);
        settings.set_stop_bits(self.stop_bits);
        settings.set_parity(self.parity);
        settings.set_flow_control(self.flow_control);
        Ok(())
    }
}

impl CommunicationBuilder for UartAdapterConfiguration {
    fn build(&self) -> Result<Box<dyn Communication>> {
        debug!("Connecting to SerialPort: {:#?}", self);
        let read_timeout = self.read_timeout;
        let mut port = SerialPort::open(self.port.clone(), self.clone())?;
        port.discard_buffers()?;
        trace!("Successfully connected to the SerialPort");
        port.set_read_timeout(read_timeout)?;
        Ok(Box::new(UartAdapter {
            port: Arc::new(port),
        }))
    }
}

#[derive(Clone)]
pub struct UartAdapter {
    port: Arc<serial2::SerialPort>,
}

impl Communication for UartAdapter {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.error_if_not_connected()?;
        debug!("Writing {:?} to UART.", data);
        self.port.write_all(data)?;
        debug!("Data written to UART.");
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        self.error_if_not_connected()?;
        let mut buf = vec![];

        // Would default of 1024 fit in well?
        let mut data = vec![0u8; 1024];

        trace!("Reading");
        while let Ok(a) = self.port.read(&mut data) {
            if a == 0 {
                break;
            }
            buf.extend_from_slice(&data[0..a]);
        }
        trace!("Read {} bytes!", buf.len());

        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(buf))
        }
    }
}

impl UartAdapter {
    fn connected(&self) -> bool {
        if self.port.get_configuration().is_ok() {
            return true;
        }
        false
    }

    fn error_if_not_connected(&self) -> Result<()> {
        if !self.connected() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Not connected").into());
        }
        Ok(())
    }
}

impl CloneableCommunication for UartAdapter {
    fn boxed_clone(&self) -> Box<dyn Communication> {
        Box::new(self.clone())
    }
}
