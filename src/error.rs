use snafu::Snafu;

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("IO Error: {e}"))]
    Io { e: std::io::Error },

    #[snafu(display("Flume Error: {e}"))]
    Flume { e: String },

    #[cfg(feature = "tcp")]
    #[snafu(display("Failed to lock mutex"))]
    FailedToLockMutex,
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

#[cfg(feature = "tcp")]
impl<I> From<std::sync::PoisonError<I>> for Error {
    fn from(_value: std::sync::PoisonError<I>) -> Self {
        Error::FailedToLockMutex
    }
}
