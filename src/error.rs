use std::{
    error::Error as StdError,
    fmt,
};

/// Common Rust error implementation for this crate.
///
/// There is a wrapper variant for any other errors
/// as well as specialized variants with crate-related logical errors.
#[derive(Debug)]
pub enum Error {
    /// Specified number of bits is not 1, 2 or 4
    WrongBits(u8),
    /// Wrapped lower level errors
    Wrapped(Box<dyn StdError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            WrongBits(bits) => write!(f, "Specified number of bits ({}) is not 1, 2 or 4", bits),
            Wrapped(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Wrapped(e) => Some(&**e),
            _ => None
        }
    }
}

/// This macro exists due to inability to implement a generic From
/// for all `T: std::error::Error`
macro_rules! from_impls {
    ($($wrapped:ty),*) => {
        $(
            impl From<$wrapped> for Error {
                fn from(e: $wrapped) -> Self {
                    Error::Wrapped(Box::new(e))
                }
            }
        )*
    };
}

from_impls!(std::io::Error, image::ImageError);
