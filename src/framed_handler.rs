use std::time::Duration;

use crate::Result;
use crate::{communication::CommunicationBuilder, packet_parser::PacketParser};
use flume::{Receiver, Sender};
use log::error;

pub struct FramedHandler<I: Send + Sync, T: PacketParser<I>> {
    /// Sender used to terminate the communication thread
    terminate_sender: Sender<()>,
    /// Sender used to send data from the application
    send_tx: Sender<I>,
    /// Receiver that receives data from the outer world
    receive_rx: Receiver<I>,
    /// Packet parser
    packet_parser: T,
}

impl<I: Send + Sync + 'static, T: PacketParser<I> + 'static> FramedHandler<I, T> {
    pub fn spawn(adapter_configuration: &dyn CommunicationBuilder) -> Result<Self> {
        let (send_tx, send_rx) = flume::unbounded::<I>();
        let (receive_tx, receive_rx) = flume::unbounded::<I>();
        let (terminate_tx, terminate_rx) = flume::unbounded();

        let packet_parser = T::new();

        let adapter_arc = adapter_configuration.build()?;

        let mut adapter = adapter_arc.boxed_clone();
        let terminate = terminate_rx.clone();
        let packet_parser_clone = packet_parser.clone();
        std::thread::spawn(move || loop {
            if !send_rx.is_empty() {
                for d in send_rx.iter() {
                    let data = packet_parser_clone.parse_to_bytes(&d);
                    if let Err(e) = adapter.send(&data) {
                        error!("An error occured while sending data: {e}");
                        return;
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
            if !terminate.is_empty() {
                break;
            }
        });

        let mut adapter = adapter_arc;
        let terminate = terminate_rx;
        let packet_parser_clone = packet_parser.clone();
        std::thread::spawn(move || loop {
            match adapter.recv() {
                Ok(recv) => {
                    if let Some(data) = recv {
                        let data = packet_parser_clone.parse_from_bytes(&data);
                        if let Err(e) = receive_tx.send(data) {
                            error!("An error occured while sending data to the flume channel: {e}");
                            return;
                        }
                    } else {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                Err(e) => {
                    error!("An error occured while receiving data: {e}");
                    return;
                }
            }
            if !terminate.is_empty() {
                break;
            }
        });

        Ok(Self {
            terminate_sender: terminate_tx,
            send_tx,
            receive_rx,
            packet_parser,
        })
    }

    /// Terminate the ongoing connection
    pub fn terminate(self) -> Result<()> {
        self.terminate_sender.send(())?;
        Ok(())
    }

    /// Get the packet parser
    pub fn get_packet_parser(&mut self) -> &mut T {
        &mut self.packet_parser
    }

    /// Get the channel sender for sending data through the adapter.
    pub fn get_sender(&self) -> Sender<I> {
        self.send_tx.clone()
    }

    /// Get the channel receiver for receiving data through the adapter.
    pub fn get_receiver(&self) -> Receiver<I> {
        self.receive_rx.clone()
    }
}
