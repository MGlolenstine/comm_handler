use crate::adapters::uart::{UartAdapter, UartAdapterConfiguration};
use crate::framed_handler::FramedHandler;
use crate::traits::{CommunicationBuilder, PacketParser};
use crate::Result;
use flume::{Receiver, Sender};

/// Packet parser that forwards packets
#[derive(Clone, Copy)]
pub struct Passthrough;
impl PacketParser<Vec<u8>> for Passthrough {
    fn new() -> Self {
        Passthrough
    }

    fn clone_inner(&self) -> Self {
        Clone::clone(self)
    }

    fn parse_from_bytes(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        Some(data.to_vec())
    }

    fn parse_to_bytes(&mut self, data: &Vec<u8>) -> Vec<u8> {
        data.to_vec()
    }
}

pub struct Handler {
    /// Sender used to send data from the application
    send_tx: Sender<Vec<u8>>,
    /// Receiver that receives data from the outer world
    receive_rx: Receiver<Vec<u8>>,
    /// Handler so that we can terminate the connection
    handler: FramedHandler<Vec<u8>, Passthrough>,
}

impl Handler {
    pub fn spawn(
        adapter_configuration: &UartAdapterConfiguration,
    ) -> Result<(Self, Box<UartAdapter>)> {
        let (handler, adapter) =
            FramedHandler::<Vec<u8>, Passthrough>::spawn(adapter_configuration)?;
        Ok((
            Self {
                send_tx: handler.get_sender(),
                receive_rx: handler.get_receiver(),
                handler,
            },
            adapter,
        ))
    }

    /// Terminate the ongoing connection
    pub fn terminate(self) -> Result<()> {
        self.handler.terminate()?;
        Ok(())
    }

    /// Get the channel sender for sending data through the adapter.
    pub fn get_sender(&self) -> Sender<Vec<u8>> {
        self.send_tx.clone()
    }

    /// Get the channel receiver for receiving data through the adapter.
    pub fn get_receiver(&self) -> Receiver<Vec<u8>> {
        self.receive_rx.clone()
    }
}
