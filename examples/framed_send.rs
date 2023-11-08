#![allow(clippy::disallowed_names)]

/// To run this example, you will have to connect a serial adapter and short TX and RX pins.
use comm_handler::{adapters::uart::UartAdapterConfiguration, traits::PacketParser, FramedHandler};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Data {
    foo: u32,
    bar: String,
    baz: f32,
}

impl PacketParser<Data> for Data {
    fn new() -> Self {
        Data::default()
    }

    fn clone_inner(&self) -> Self {
        Clone::clone(self)
    }

    fn parse_from_bytes(&self, data: &[u8]) -> Data {
        // `foo` field in binary takes up 4 bytes
        let foo: u32 = u32::from_le_bytes(data[0..4].try_into().unwrap());

        // `bar` length field in binary takes up 4 bytes
        let bar_length = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        // `bar` field in binary takes up `bar_length` bytes
        let bar = String::from_utf8_lossy(&data[8..8 + bar_length]).to_string();

        // `baz` field in binary takes up 4 bytes
        let baz = f32::from_le_bytes(data[8 + bar_length..].try_into().unwrap());

        Data { foo, bar, baz }
    }

    fn parse_to_bytes(&self, data: &Data) -> Vec<u8> {
        let mut ret = vec![];

        // Add `foo` field to the output
        ret.extend_from_slice(&data.foo.to_le_bytes());

        // Add `bar` field to the output
        // We need to know the length of the string
        ret.extend_from_slice(&(data.bar.len() as u32).to_le_bytes());
        ret.extend_from_slice(data.bar.as_bytes());

        // Add `baz` field to the output
        ret.extend_from_slice(&data.baz.to_le_bytes());

        ret
    }
}

fn main() {
    env_logger::init();
    let config = UartAdapterConfiguration::new("/dev/ttyUSB0");

    let handler = FramedHandler::<Data, Data>::spawn(&config).unwrap();

    let sender = handler.get_sender();
    let receiver = handler.get_receiver();

    let original_data = Data {
        foo: 1337,
        bar: "Testing, testing, is anyone there?".to_string(),
        baz: std::f32::consts::PI,
    };

    sender.send(original_data.clone_inner()).unwrap();
    let data = receiver.recv().unwrap();

    assert_eq!(original_data, data);
}
