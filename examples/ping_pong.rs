use std::time::Duration;

/// To run this example, you will have to connect two serial adapters and connect them TX-RX.
/// This example requires "uart" feature.
use comm_handler::{adapters::uart::UartAdapterConfiguration, traits::PacketParser, FramedHandler};
use log::info;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum PingPong {
    #[default]
    Ping,
    Pong,
}

impl PacketParser<PingPong> for PingPong {
    fn new() -> Self {
        PingPong::Ping
    }

    fn clone_inner(&self) -> Self {
        Clone::clone(self)
    }

    fn parse_from_bytes(&mut self, data: &[u8]) -> Option<PingPong> {
        Some(match data[0] {
            1 => PingPong::Ping,
            2 => PingPong::Pong,
            _ => return None,
        })
    }

    fn parse_to_bytes(&mut self, data: &PingPong) -> Vec<u8> {
        vec![match data {
            PingPong::Ping => 1,
            PingPong::Pong => 2,
        }]
    }
}

fn main() {
    env_logger::init();
    spawn_client("/dev/ttyUSB0", false);
    spawn_client("/dev/ttyUSB1", true);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn spawn_client(port: &str, start: bool) {
    let port = port.to_string();
    std::thread::spawn(move || {
        let config = UartAdapterConfiguration::new(&port);

        let handler = FramedHandler::<PingPong, PingPong>::spawn(&config).unwrap();

        let sender = handler.get_sender();
        let receiver = handler.get_receiver();

        if start {
            let original_data = PingPong::Ping;
            sender.send(original_data.clone_inner()).unwrap();
        }

        while let Ok(packet) = receiver.recv() {
            match packet {
                PingPong::Pong => {
                    info!("Received Pong: {port}");
                    sender.send(PingPong::Ping).unwrap();
                }
                PingPong::Ping => {
                    info!("Received Ping: {port}");
                    sender.send(PingPong::Pong).unwrap();
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        info!("Breaking out of the packet reading loop");
    });
}
