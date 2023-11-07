use snafu::Snafu;

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("IO Error: {e}"))]
    Io { e: std::io::Error },

    #[snafu(display("Flume Error: {e}"))]
    Flume { e: String },
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
