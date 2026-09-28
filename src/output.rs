use std::io::{self, Write};

/// A failure reported on stderr with exit status 1.
#[derive(Debug)]
pub enum Error {
    /// Printed as `Error: <message>`.
    Message(String),
    /// Printed as `Error: <message>` followed by a response body. The body is
    /// kept as bytes because upstream responses are not guaranteed to be UTF-8.
    Response(String, Vec<u8>),
    /// Printed verbatim, such as usage text.
    Usage(String),
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Message(value.to_string())
    }
}

pub fn write(bytes: impl AsRef<[u8]>) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes.as_ref())?;
    stdout.flush()?;
    Ok(())
}
pub fn error(error: &Error) {
    let mut stderr = io::stderr().lock();
    let _ = match error {
        Error::Message(message) => writeln!(stderr, "Error: {message}"),
        Error::Response(message, body) => writeln!(stderr, "Error: {message}")
            .and_then(|()| stderr.write_all(body))
            .and_then(|()| stderr.write_all(b"\n")),
        Error::Usage(text) => writeln!(stderr, "{text}"),
    };
    let _ = stderr.flush();
}
