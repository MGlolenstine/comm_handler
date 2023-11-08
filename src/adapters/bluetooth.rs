use crate::{traits::Connectable, Result};
use snafu::Snafu;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

use log::{debug, error, trace};

use crate::traits::{CloneableCommunication, Communication, CommunicationBuilder};
use btleplug::{
    api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType},
    platform::{Adapter, Manager, Peripheral},
};
use tokio::runtime::Runtime;

#[derive(Snafu, Debug)]
pub enum BluetoothError {
    #[snafu(display("No adapters present"))]
    NoAdaptersPresent,
    #[snafu(display("No peripherals found"))]
    NoPeripheralsFound,
    #[snafu(display("Characteristic not found"))]
    CharacteristicNotFound,

    #[snafu(display("Btleplug: {e}"))]
    Btleplug { e: btleplug::Error },
}

impl From<btleplug::Error> for BluetoothError {
    fn from(value: btleplug::Error) -> Self {
        BluetoothError::Btleplug { e: value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceCharacteristic {
    service: String,
    characteristic: String,
}

impl ServiceCharacteristic {
    pub fn new<A: ToString, B: ToString>(service: A, char: B) -> Self {
        Self {
            service: service.to_string(),
            characteristic: char.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct BluetoothAdapterConfiguration {
    device_mac: String,
    read_characteristic: ServiceCharacteristic,
    write_characteristic: ServiceCharacteristic,
    runtime: Runtime,
}

impl BluetoothAdapterConfiguration {
    pub fn new<T: ToString>(
        device_mac: T,
        read_characteristic: ServiceCharacteristic,
        write_characteristic: ServiceCharacteristic,
    ) -> Self {
        let runtime = Runtime::new().unwrap();
        Self {
            device_mac: device_mac.to_string(),
            read_characteristic,
            write_characteristic,
            runtime,
        }
    }

    fn select_first_adapter(&self) -> Result<Adapter> {
        self.runtime.block_on(async {
            let manager = Manager::new().await.map_err(BluetoothError::from)?;
            let adapters = manager.adapters().await.map_err(BluetoothError::from)?;
            debug!("connected_adapters: {:#?}", adapters);
            let Some(adapter) = adapters.first() else {
                error!("There's no adapters connected!");
                return Err(BluetoothError::NoAdaptersPresent.into());
            };
            debug!("Using the adapter: {:?}", adapter);
            Ok(adapter.clone())
        })
    }

    fn find_peripheral(&self, adapter: &Adapter) -> Result<Peripheral> {
        self.runtime.block_on(async {
            let filter = ScanFilter {
                services: vec![
                    Uuid::parse_str(&self.read_characteristic.service).unwrap(),
                    Uuid::parse_str(&self.write_characteristic.service).unwrap(),
                ],
            };
            trace!("Starting scanning");
            adapter
                .start_scan(filter)
                .await
                .map_err(BluetoothError::from)?;

            if tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    match adapter.peripherals().await {
                        Ok(a) => {
                            if a.into_iter()
                                .inspect(|a| trace!("Found device: {}", a.address()))
                                .any(|a| a.address().to_string() == self.device_mac)
                            {
                                break;
                            } else {
                                trace!("Peripheral search returned empty results.");
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                        Err(e) => {
                            error!("adapter.peripherals() failed: {e}");
                        }
                    }
                }
            })
            .await
            .is_err()
            {
                error!("Failed to find a device you were looking for!");
                return Err(BluetoothError::NoPeripheralsFound.into());
            }
            let peripherals = adapter.peripherals().await;
            if peripherals.is_err() {
                return Err(BluetoothError::NoPeripheralsFound.into());
            }
            let peripherals = peripherals.unwrap();

            if let Some(peripheral) = peripherals
                .into_iter()
                .find(|a| a.address().to_string() == self.device_mac)
            {
                Ok(peripheral)
            } else {
                Err(BluetoothError::NoPeripheralsFound.into())
            }
        })
    }

    fn find_characteristic<T: ToString>(
        &self,
        peripheral: &Peripheral,
        char: T,
    ) -> Result<Characteristic> {
        let uuid = Uuid::parse_str(&char.to_string()).unwrap();
        self.runtime.block_on(async {
            for s in peripheral.services() {
                for c in s.characteristics {
                    if c.uuid == uuid {
                        return Ok(c);
                    }
                }
            }
            Err(BluetoothError::CharacteristicNotFound.into())
        })
    }
}

impl CommunicationBuilder for BluetoothAdapterConfiguration {
    fn build(&self) -> Result<Box<dyn Communication>> {
        debug!("Connecting to Bluetooth: {:#?}", self);
        trace!("Selecting first adapter");
        let adapter = self.select_first_adapter()?;
        trace!("Finding peripheral");
        let peripheral = self.find_peripheral(&adapter)?;

        trace!("Connecting to the peripheral");
        self.runtime
            .block_on(peripheral.connect())
            .map_err(BluetoothError::from)?;

        trace!("Discovering services");
        self.runtime
            .block_on(peripheral.discover_services())
            .map_err(BluetoothError::from)?;
        trace!("Finding services");
        let read_characteristic =
            self.find_characteristic(&peripheral, &self.read_characteristic.characteristic)?;
        let write_characteristic =
            self.find_characteristic(&peripheral, &self.write_characteristic.characteristic)?;

        trace!("Successfully connected to the Bluetooth");
        Ok(Box::new(BluetoothAdapter {
            peripheral: Arc::new(peripheral),
            read_characteristic,
            write_characteristic,
            runtime: Arc::new(Runtime::new()?),
        }))
    }
}

#[derive(Clone)]
pub struct BluetoothAdapter {
    peripheral: Arc<btleplug::platform::Peripheral>,
    read_characteristic: Characteristic,
    write_characteristic: Characteristic,
    runtime: Arc<Runtime>,
}

impl Communication for BluetoothAdapter {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.error_if_not_connected()?;
        debug!("Writing {:?} to BT.", data);
        self.runtime
            .block_on(self.peripheral.write(
                &self.write_characteristic,
                data,
                WriteType::WithResponse,
            ))
            .map_err(BluetoothError::from)?;
        debug!("Data written to BT.");
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        self.error_if_not_connected()?;
        let data = self
            .runtime
            .block_on(async { self.peripheral.read(&self.read_characteristic).await })
            .map_err(BluetoothError::from)?;
        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }
}

impl BluetoothAdapter {
    fn error_if_not_connected(&mut self) -> Result<()> {
        if !self.connected() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Not connected").into());
        }
        Ok(())
    }
}

impl Connectable for BluetoothAdapter {
    fn connect(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn disconnect(&mut self) -> Result<bool> {
        self.runtime
            .block_on(self.peripheral.disconnect())
            .map_err(BluetoothError::from)?;
        Ok(true)
    }

    fn connected(&mut self) -> bool {
        self.runtime
            .block_on(self.peripheral.is_connected())
            .unwrap_or(false)
    }
}

impl CloneableCommunication for BluetoothAdapter {
    fn boxed_clone(&self) -> Box<dyn Communication> {
        Box::new(self.clone())
    }
}

pub fn uuid_from_u16(id: u16) -> String {
    let bytes = id.to_be_bytes();
    format!(
        "0000{:02x}{:02x}-0000-1000-8000-00805f9b34fb",
        bytes[0], bytes[1],
    )
}

#[test]
fn test_uuid_from_u16() {
    assert_eq!(
        uuid_from_u16(0x2a3d),
        "00002a3d-0000-1000-8000-00805f9b34fb"
    );
}

pub fn uuid_from_u32(id: u32) -> String {
    let bytes = id.to_be_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-0000-1000-8000-00805f9b34fb",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

#[test]
fn test_uuid_from_u32() {
    assert_eq!(
        uuid_from_u32(0x13370042),
        "13370042-0000-1000-8000-00805f9b34fb"
    );
}
