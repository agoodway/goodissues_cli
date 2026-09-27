use std::io::{self, Write};

/// Keep API error bytes intact even when an upstream sends non-UTF-8 text.
pub struct Error(pub Vec<u8>);
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}
pub fn write(bytes: impl AsRef<[u8]>) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes.as_ref())
        .and_then(|()| stdout.flush())
        .map_err(|e| format!("Error: {e}"))
}
pub fn error(error: &Error) {
    let mut stderr = io::stderr().lock();
    let _ = stderr.write_all(&error.0);
    let _ = stderr.write_all(b"\n");
    let _ = stderr.flush();
}
