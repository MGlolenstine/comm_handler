use crate::traits::{CloneableCommunication, Communication, CommunicationBuilder};
use crate::Result;
use log::{debug, trace};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TcpAdapterConfiguration {
    address: String,
    timeout: u64,
}

impl TcpAdapterConfiguration {
    pub fn new<S: ToString>(address: S, timeout: u64) -> Self {
        Self {
            address: address.to_string(),
            timeout,
        }
    }
}

impl CommunicationBuilder for TcpAdapterConfiguration {
    fn build(&self) -> Result<Box<dyn Communication>> {
        debug!("Connecting to Tcp socket: {:#?}", self);

        let stream = TcpStream::connect(self.address.clone())?;
        stream.set_read_timeout(Some(Duration::from_millis(self.timeout)))?;
        stream.set_write_timeout(Some(Duration::from_millis(self.timeout)))?;

        trace!("Successfully connected to the Tcp socket");
        Ok(Box::new(TcpAdapter {
            connected: true,
            in_stream: Arc::new(Mutex::new(stream.try_clone()?)),
            out_stream: Arc::new(Mutex::new(stream)),
        }))
    }
}

#[derive(Clone)]
pub struct TcpAdapter {
    connected: bool,
    in_stream: Arc<Mutex<TcpStream>>,
    out_stream: Arc<Mutex<TcpStream>>,
}

impl Communication for TcpAdapter {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.error_if_not_connected()?;
        debug!("Writing {:?} to Tcp socket.", data);
        if let Err(e) = self.out_stream.lock()?.write_all(data) {
            dbg!("Disconnected!");
            self.connected = false;
            return Err(e.into());
        }
        debug!("Data written to Tcp socket.");
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        self.error_if_not_connected()?;
        let mut buf = vec![];

        // Would default of 1024 fit in well?
        let mut data = vec![0u8; 1024];

        trace!("Waiting for bytes");
        //TODO: Handle disconnection when trying to read from the stream.
        while let Ok(a) = self.in_stream.lock()?.read(&mut data) {
            trace!("Read {a} bytes");
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

impl TcpAdapter {
    fn connected(&self) -> bool {
        self.connected
    }

    fn error_if_not_connected(&self) -> Result<()> {
        if !self.connected() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Not connected").into());
        }
        Ok(())
    }
}

impl CloneableCommunication for TcpAdapter {
    fn boxed_clone(&self) -> Box<dyn Communication> {
        Box::new(self.clone())
    }
}
