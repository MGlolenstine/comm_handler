use std::time::Duration;

use crate::traits::{CommunicationBuilder, PacketParser};
use crate::Result;
use flume::{Receiver, Sender};
use log::error;

pub struct FramedHandler<I: Send + Sync, R: PacketParser<I>, J = I, T = R> {
    /// Sender used to terminate the communication thread
    terminate_sender: Sender<()>,
    /// Receiver that receives data from the outer world
    receive_rx: Receiver<I>,
    /// Sender used to send data from the application
    send_tx: Sender<J>,
    /// Packet parser used for parsing incoming bytes
    packet_parser_incoming: R,
    /// Packet parser used for generating outgoing bytes
    packet_parser_outgoing: T,
}

impl<
        I: Send + Sync + 'static,
        R: PacketParser<I> + 'static,
        J: Send + Sync + 'static,
        T: PacketParser<J> + 'static,
    > FramedHandler<I, R, J, T>
{
    pub fn spawn(adapter_configuration: &dyn CommunicationBuilder) -> Result<Self> {
        let (send_tx, send_rx) = flume::unbounded::<J>();
        let (receive_tx, receive_rx) = flume::unbounded::<I>();
        let (terminate_tx, terminate_rx) = flume::unbounded();

        let packet_parser_outgoing = T::new();
        let packet_parser_incoming = R::new();

        let adapter_arc = adapter_configuration.build()?;

        let mut adapter = adapter_arc.boxed_clone();
        let terminate = terminate_rx.clone();
        let mut packet_parser_outgoing_clone = packet_parser_outgoing.clone_inner();
        std::thread::spawn(move || loop {
            if !send_rx.is_empty() {
                for d in send_rx.iter() {
                    let data = packet_parser_outgoing_clone.parse_to_bytes(&d);
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

        let mut packet_parser_incoming_clone = packet_parser_incoming.clone_inner();
        std::thread::spawn(move || loop {
            match adapter.recv() {
                Ok(recv) => {
                    if let Some(data) = recv {
                        let data = packet_parser_incoming_clone.parse_from_bytes(&data);
                        if let Some(data) = data {
                            if let Err(e) = receive_tx.send(data) {
                                error!(
                                    "An error occured while sending data to the flume channel: {e}"
                                );
                                return;
                            }
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
            packet_parser_incoming,
            packet_parser_outgoing,
        })
    }

    /// Terminate the ongoing connection
    pub fn terminate(self) -> Result<()> {
        self.terminate_sender.send(())?;
        Ok(())
    }

    /// Get the incoming packet parser
    pub fn get_incoming_packet_parser(&mut self) -> &mut R {
        &mut self.packet_parser_incoming
    }

    /// Get the outgoing packet parser
    pub fn get_outgoing_packet_parser(&mut self) -> &mut T {
        &mut self.packet_parser_outgoing
    }

    /// Get the channel sender for sending data through the adapter.
    pub fn get_sender(&self) -> Sender<J> {
        self.send_tx.clone()
    }

    /// Get the channel receiver for receiving data through the adapter.
    pub fn get_receiver(&self) -> Receiver<I> {
        self.receive_rx.clone()
    }
}

// Support adding with just a single associated type.
impl<I: Send + Sync + 'static, R: PacketParser<I> + 'static> FramedHandler<I, R> {
    pub fn new() -> Self {
        FramedHandler::<I, R, I, R>::default()
    }
}

impl<I: Send + Sync + 'static, R: PacketParser<I> + 'static> Default for FramedHandler<I, R> {
    fn default() -> Self {
        Self::new()
    }
}
