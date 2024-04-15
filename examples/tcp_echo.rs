/// This example requires "tcp" feature to be enabled.
use comm_handler::adapters::tcp::TcpAdapterConfiguration;
use comm_handler::Handler;
use log::{debug, error, info};
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    env_logger::init();
    let listener = std::net::TcpListener::bind("0.0.0.0:50000").expect("Failed to bind a listener");

    std::thread::spawn(move || {
        if let Ok((mut socket, addr)) = listener.accept() {
            info!("Addr {:?} connected!", addr);
            let mut buf = [0; 128];
            loop {
                debug!("Listener: Waiting for the data");
                let bytes_read = socket.read(&mut buf).unwrap();
                debug!("Listener: Writing data back");
                socket.write_all(&buf[0..bytes_read]).unwrap();
                debug!("Listener: Data written back");
            }
        } else {
            error!("Socket failed to connect!");
        }
    });

    let comm = crate::Handler::spawn(&TcpAdapterConfiguration::new("127.0.0.1:50000", 10))
        .expect("Failed to connect to the TCP");

    let rx = comm.get_receiver();
    let tx = comm.get_sender();

    std::thread::sleep(Duration::from_millis(500));

    let mut echo_data = b"Hello World!".to_vec();

    debug!("Adapter: Sending data");
    tx.send(echo_data.clone()).unwrap();
    debug!("Adapter: Waiting for data");
    let data = rx.recv().unwrap();

    debug!("Adapter: Comparing data");
    assert_eq!(&data, &echo_data);

    echo_data.reverse();

    debug!("Adapter: Sending data");
    tx.send(echo_data.clone()).unwrap();
    debug!("Adapter: Waiting for data");
    let data = rx.recv().unwrap();

    debug!("Adapter: Comparing data");
    assert_eq!(data, echo_data);
}
