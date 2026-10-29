use std::fmt;

/// Errors from loading Wintab or opening a context.
#[derive(Debug)]
pub enum Error {
    /// `wintab32.dll` could not be loaded; usually no tablet driver is installed.
    NotInstalled(windows::core::Error),
    /// The driver does not export a function this crate needs.
    MissingFunction(&'static str),
    /// The driver is installed but Wintab services are not available.
    Unavailable,
    /// `WTOpenW` refused the context.
    OpenFailed,
    /// The driver rejected a call on an open context.
    CallFailed(&'static str),
    /// The driver does not support an extension the context asked for.
    UnsupportedExtension(crate::Extension),
}

/// `Result` with [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInstalled(error) => write!(f, "wintab32.dll could not be loaded: {error}"),
            Self::MissingFunction(name) => write!(f, "wintab32.dll does not export {name}"),
            Self::Unavailable => f.write_str("Wintab services are not available"),
            Self::OpenFailed => f.write_str("the Wintab context could not be opened"),
            Self::CallFailed(name) => write!(f, "{name} failed"),
            Self::UnsupportedExtension(extension) => {
                write!(f, "the driver does not support {extension:?}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotInstalled(error) => Some(error),
            _ => None,
        }
    }
}
