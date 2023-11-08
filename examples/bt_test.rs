use std::time::Duration;

use btleplug::api::bleuuid::uuid_from_u32;
/// To run this example, you will have to have a BT device with the following requirements:
/// Service: 0x13370042
///    - READ char: 0x00002a3d
///    - WRITE char: 0x00002a3d
use comm_handler::adapters::bluetooth::{
    uuid_from_u16, BluetoothAdapterConfiguration, ServiceCharacteristic,
};
use comm_handler::Handler;
use log::trace;

fn main() {
    env_logger::init();

    let service = uuid_from_u32(0x13370042);
    let read = ServiceCharacteristic::new(service, uuid_from_u16(0x2a3d));
    let write = ServiceCharacteristic::new(service, uuid_from_u16(0x2a3d));

    let config = BluetoothAdapterConfiguration::new("48:51:C5:9C:F3:D7", read, write);

    let handler = Handler::spawn(&config).unwrap();

    let sender = handler.get_sender();
    let receiver = handler.get_receiver();

    let original_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

    for _ in 0..5 {
        sender.send(original_data.clone()).unwrap();

        let data = receiver.recv().unwrap();
        trace!("User received data. {data:?}");
        assert_eq!(data, vec![104, 105]);
        std::thread::sleep(Duration::from_millis(100));
    }
}
