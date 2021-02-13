use fmt::{Debug, Display, Formatter};
use io::{Read, Write};
use std::{fmt, io};
use std::convert::TryFrom;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::Error;

/// Hides a slice of bytes along with its length behind bytes from `carrier`.
///
/// Returns a vector of `(4 + payload.len()) * bits.ratio()` bytes which have their least significant
/// bits replaced by the `payload` data prefixed with its length.
///
/// `bits` determine how many least significant bits are replaced.
///
/// # Errors
/// Only lower-level IO errors might occur, depending solely on supplied carrier.
///
/// Most common and obvious one is an `UnexpectedEof` when there are not enough carrier bytes.
///
/// # Examples
///
/// ```
/// # use std::io::Cursor;
/// # use steganographer::binary::{hide_bytes, Bits};
///
/// let mut carrier = Cursor::new([0b11100000; 16]);
/// let cloaked = hide_bytes(&[5, 14, 7, 3], carrier, Bits::Four).unwrap();
///
/// assert_eq!(&cloaked, &[0b11100000, 0b11100000,   // 0 \
///                        0b11100000, 0b11100000,   // 0 |
///                        0b11100000, 0b11100000,   // 0 | u32 number of bytes
///                        0b11100000, 0b11100100,   // 4 /
///                        0b11100000, 0b11100101,   // 5
///                        0b11100000, 0b11101110,   // 14
///                        0b11100000, 0b11100111,   // 7
///                        0b11100000, 0b11100011]); // 3
/// ```
///
pub fn hide_bytes(payload: &[u8], carrier: impl Read, bits: Bits) -> Result<Vec<u8>, Error> {
    let mut result = Vec::with_capacity(4 + payload.len() * bits.ratio());
    let mut writer = SteganographWriter::new(carrier, &mut result).bits(bits);

    writer.write_u32::<BigEndian>(payload.len() as u32)?;
    writer.write_all(&payload)?;
    Ok(result)
}

/// Reveals a slice of bytes previously hidden by the [`hide_bytes`](fn.hide_bytes.html) function.
///
/// Extracts 4 bytes of `length` and then `length` bytes from the `reader` input, reading
/// `(4 + length) * bits.ratio()` bytes from it.
///
/// # Errors
/// Only lower-level IO errors might occur, depending solely on supplied reader.
///
/// Most common and obvious one is an `UnexpectedEof` when size extracted from first `4 * bits.ratio()`
/// bytes is greater than the number of bytes that can be read from the `reader`.
///
/// # Examples
///
/// ```
/// # use std::io::Cursor;
/// # use steganographer::binary::{reveal_bytes, Bits};
///
/// // these bytes are from the hide_bytes example
/// let mut cloaked = Cursor::new([224, 224, 224, 224, 224, 224, 224, 228, 224, 229, 224, 238, 224, 231, 224, 227]);
/// let extracted = reveal_bytes(&mut cloaked, Bits::Four).unwrap();
///
/// assert_eq!(&extracted, &[5, 14, 7, 3]);
/// ```
///
pub fn reveal_bytes(reader: impl Read, bits: Bits) -> Result<Vec<u8>, Error> {
    let mut reader = SteganographReader::new(reader).bits(bits);
    let size = reader.read_u32::<BigEndian>()? as usize;
    let mut result = vec![0; size];
    reader.read_exact(&mut result)?;
    Ok(result)
}

/// A wrapper over some reader that extracts bytes from appropriate least significant bits
///
/// # Examples
///
/// ```
/// # use std::io::{Read, Cursor};
/// # use steganographer::binary::{SteganographReader, Bits};
///
/// let mut cloaked = Cursor::new([224, 227, 225, 226, 224, 225, 225, 227, 224, 224, 225, 226, 225, 227, 227, 227]);
/// let mut reader = SteganographReader::new(cloaked).bits(Bits::Two);
/// let mut buf = vec![0; 4];
///
/// reader.read_exact(&mut buf).unwrap();
///
/// assert_eq!(&buf, &[54, 23, 6, 127]);
/// ```
///
pub struct SteganographReader<T: Read> {
    source: T,
    bits: Bits,
}

impl<T: Read> SteganographReader<T> {
    /// Creates an instance of [SteganographReader](struct.SteganographReader.html)
    /// with 1 bit of hidden data per image color byte.
    pub fn new(source: T) -> Self {
        SteganographReader { source, bits: Bits::default() }
    }

    /// Configures the reader to use a specified number of bits
    /// of hidden data per image color byte.
    pub fn bits(self, bits: Bits) -> Self {
        SteganographReader { bits, ..self }
    }
}

impl<T: Read> Read for SteganographReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let mask = self.bits.mask();
        let mut buffer = vec![0; self.bits.ratio()];

        for i in 0..buf.len() {
            self.source.read_exact(&mut buffer)?;
            buf[i] = buffer.iter()
                .zip((0..8).step_by(self.bits as usize).rev())
                .map(|(&byte, shift)| (byte & mask) << shift)
                .fold(0u8, |acc, b| acc | b);
        }
        Ok(buf.len())
    }
}

/// A wrapper over some reader and some writer hides bytes into least significant bits of data from
/// the reader and then and writes them all into the writer.
///
/// # Examples
///
/// ```
/// # use std::io::{Write, Cursor};
/// # use steganographer::binary::{SteganographWriter, Bits};
///
/// let mut carrier = Cursor::new([224; 16]);
/// let mut result = Vec::new();
/// let mut writer = SteganographWriter::new(carrier, &mut result).bits(Bits::Two);
///
/// writer.write_all(&[54, 23, 6, 127]).unwrap();
///
/// assert_eq!(&result, &[224, 227, 225, 226, 224, 225, 225, 227, 224, 224, 225, 226, 225, 227, 227, 227]);
/// ```
///
pub struct SteganographWriter<R: Read, W: Write> {
    carrier: R,
    destination: W,
    bits: Bits,
}

impl<R: Read, W: Write> SteganographWriter<R, W> {
    /// Creates an instance of [SteganographWriter](struct.SteganographWriter.html)
    /// that expects 1 bit of hidden data per image color byte.
    pub fn new(carrier: R, destination: W) -> SteganographWriter<R, W> {
        SteganographWriter { carrier, destination, bits: Bits::default() }
    }

    /// Configures the writer to expect a specified number of bits
    /// of hidden data per image color byte.
    pub fn bits(self, bits: Bits) -> Self {
        SteganographWriter { bits, ..self }
    }
}

impl<R: Read, W: Write> Write for SteganographWriter<R, W> {
    fn write(&mut self, payload: &[u8]) -> Result<usize, io::Error> {
        let mask = self.bits.mask();
        let mut buffer = vec![0; self.bits.ratio()];

        for payload_byte in payload {
            self.carrier.read_exact(&mut buffer)?;
            let encoded = buffer.iter()
                .zip((0..8).step_by(self.bits as usize).rev())
                .map(|(&byte, shift)| byte & !mask | (payload_byte >> shift) & mask)
                .collect::<Vec<_>>();
            self.destination.write_all(&encoded)?;
        };
        Ok(payload.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.destination.flush()
    }
}

/// A enum that represents the number of least significant bits to be replaced with the payload data.
/// This crate allows only divisors of 8 for simplicity.
#[derive(Copy, Clone)]
pub enum Bits {
    /// Use only one least significant bit to store hidden data
    One = 1,
    /// Use two least significant bits to store hidden data
    Two = 2,
    /// Use a whole least significant half of the byte to store hidden data
    Four = 4,
}

impl Bits {
    /// Returns the mask value where least significant bits are ones.
    ///
    /// # Examples
    ///
    /// ```
    /// # use steganographer::binary::Bits;
    /// assert_eq!(Bits::One.mask(), 0b00000001);
    /// assert_eq!(Bits::Two.mask(), 0b00000011);
    /// assert_eq!(Bits::Four.mask(), 0b00001111);
    /// ```
    pub const fn mask(&self) -> u8 {
        (1 << *self as u8) - 1
    }

    /// Returns how many bytes are needed to hide one byte of data
    /// given that this many least significant bits per byte are replaced.
    ///
    /// # Examples
    ///
    /// ```
    /// # use steganographer::binary::Bits;
    /// assert_eq!(Bits::One.ratio(), 8);
    /// assert_eq!(Bits::Two.ratio(), 4);
    /// assert_eq!(Bits::Four.ratio(), 2);
    /// ```
    pub const fn ratio(&self) -> usize {
        8 / *self as usize
    }
}

impl Default for Bits {
    fn default() -> Self {
        Bits::One
    }
}

impl TryFrom<u8> for Bits {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Bits::One),
            2 => Ok(Bits::Two),
            4 => Ok(Bits::Four),
            x => Err(Error::WrongBits(x))
        }
    }
}

impl From<Bits> for u8 {
    fn from(bits: Bits) -> Self {
        bits as u8
    }
}

impl Debug for Bits {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Bits::One => write!(f, "Bits::One"),
            Bits::Two => write!(f, "Bits::Two"),
            Bits::Four => write!(f, "Bits::Four"),
        }
    }
}

impl Display for Bits {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} bits", *self as u8)
    }
}
