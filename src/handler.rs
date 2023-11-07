use std::{sync::Arc, time::Duration};

use crate::communication::CommunicationBuilder;
use crate::Result;
use flume::{Receiver, Sender};
use log::error;

pub struct Handler {
    /// Adapter that is the heart of the communication
    // adapter: Box<dyn Communication>,
    /// Sender used to terminate the communication thread
    terminate_sender: Sender<()>,
    /// Sender used to send data from the application
    send_tx: Sender<Vec<u8>>,
    /// Receiver that receives data from the outer world
    receive_rx: Receiver<Vec<u8>>,
}

impl Handler {
    pub fn spawn(adapter_configuration: &dyn CommunicationBuilder) -> Result<Self> {
        let (send_tx, send_rx) = flume::unbounded::<Vec<u8>>();
        let (receive_tx, receive_rx) = flume::unbounded::<Vec<u8>>();
        let (terminate_tx, terminate_rx) = flume::unbounded();

        let adapter_arc = Arc::new(adapter_configuration.build()?);

        let adapter = adapter_arc.clone();
        let terminate = terminate_rx.clone();
        std::thread::spawn(move || loop {
            if !send_rx.is_empty() {
                for d in send_rx.iter() {
                    if let Err(e) = adapter.send(&d) {
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

        let adapter = adapter_arc;
        let terminate = terminate_rx;
        std::thread::spawn(move || loop {
            match adapter.recv() {
                Ok(recv) => {
                    if let Some(data) = recv {
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
        })
    }

    /// Terminate the ongoing connection
    pub fn terminate(self) -> Result<()> {
        self.terminate_sender.send(())?;
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
