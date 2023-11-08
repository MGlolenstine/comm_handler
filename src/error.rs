use snafu::Snafu;

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("IO Error: {e}"))]
    Io { e: std::io::Error },

    #[snafu(display("Flume Error: {e}"))]
    Flume { e: String },

    #[cfg(feature = "bluetooth")]
    #[snafu(display("Btleplug Error: {e}"))]
    BtlePlug {
        e: crate::adapters::bluetooth::BluetoothError,
    },
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io { e: value }
    }
}

impl<T> From<flume::SendError<T>> for Error {
    fn from(value: flume::SendError<T>) -> Self {
        Error::Flume {
            e: value.to_string(),
        }
    }
}

#[cfg(feature = "bluetooth")]
impl From<crate::adapters::bluetooth::BluetoothError> for Error {
    fn from(value: crate::adapters::bluetooth::BluetoothError) -> Self {
        Error::BtlePlug { e: value }
    }
}
