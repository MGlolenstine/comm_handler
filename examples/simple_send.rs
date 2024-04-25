/// To run this example, you will have to connect a serial adapter and short TX and RX pins.
use comm_handler::adapters::uart::UartAdapterConfiguration;
use comm_handler::Handler;
use log::trace;

fn main() {
    env_logger::init();
    let config = UartAdapterConfiguration::new("/dev/ttyUSB0");

    let (handler, _adapter) = Handler::spawn(&config).unwrap();

    let sender = handler.get_sender();
    let receiver = handler.get_receiver();

    let original_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    sender.send(original_data.clone()).unwrap();

    let data = receiver.recv().unwrap();
    trace!("User received data.");

    assert_eq!(original_data, data);
    dbg!(data);
}
