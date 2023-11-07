use std::time::Duration;

/// To run this example, you will have to connect a serial adapter and short TX and RX pins.
use comm_handler::{
    adapters::uart::UartAdapterConfiguration, framed_handler::FramedHandler,
    packet_parser::PacketParser,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Data {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

#[derive(Clone)]
pub struct ParseEnum;

impl PacketParser<Data> for ParseEnum {
    fn new() -> Self {
        ParseEnum
    }

    fn clone(&self) -> Self {
        Clone::clone(self)
    }

    fn parse_from_bytes(&self, data: &[u8]) -> Data {
        match data[0] {
            0 => Data::Zero,
            1 => Data::One,
            2 => Data::Two,
            3 => Data::Three,
            4 => Data::Four,
            5 => Data::Five,
            6 => Data::Six,
            7 => Data::Seven,
            8 => Data::Eight,
            9 => Data::Nine,
            _ => Data::Zero,
        }
    }

    fn parse_to_bytes(&self, data: &Data) -> Vec<u8> {
        vec![match data {
            Data::Zero => 0,
            Data::One => 1,
            Data::Two => 2,
            Data::Three => 3,
            Data::Four => 4,
            Data::Five => 5,
            Data::Six => 6,
            Data::Seven => 7,
            Data::Eight => 8,
            Data::Nine => 9,
        }]
    }
}

fn main() {
    env_logger::init();
    let config = UartAdapterConfiguration {
        port: "/dev/ttyUSB0".to_string(),
        ..Default::default()
    };

    let handler = FramedHandler::<Data, ParseEnum>::spawn(&config).unwrap();

    let sender = handler.get_sender();
    let receiver = handler.get_receiver();

    let send_and_compare = |original_data: Data| {
        sender.send(original_data).unwrap();
        let data = receiver.recv().unwrap();
        assert_eq!(original_data, data);
        std::thread::sleep(Duration::from_millis(10));
    };

    send_and_compare(Data::One);
    send_and_compare(Data::Three);
    send_and_compare(Data::Three);
    send_and_compare(Data::Seven);
}
