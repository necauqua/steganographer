#![warn(missing_docs)]

//! This crate provides an extremely simple set of tools for hiding data in some other data.
//!
//! This is mainly targeted to hide data in images, because pixels are altered slightly so that
//! the human eye would not notice the difference.

use std::fs::{File, OpenOptions};
use std::io::{Read, stdout, Write, stdin};
use std::path::PathBuf;

use image::{ImageDecoder, GenericImageView};
use image::codecs::png::{PngDecoder, PngEncoder};

mod error;

/// This module provides utilities for loosely hiding bytes in some carrying binary data by
/// replacing its least significant bits.
///
/// This can be used to hide data in a sligt unnoticeable color variations of some image file,
/// for example.
pub mod binary;

use binary::{Bits, hide_bytes, reveal_bytes};
pub use error::Error;

/// Decodes bytes from the image file and writes them to either the supplied output or to the stdout
pub fn decode_from_image(encoded: PathBuf, result: Option<PathBuf>, replace: bool) -> Result<(), Error> {
    let res = reveal_bytes(PngDecoder::new(File::open(encoded)?)?.into_reader()?, Bits::Two)?;
    match result {
        Some(o) => OpenOptions::new()
            .write(true)
            .truncate(true)
            .create_new(!replace)
            .open(o)?
            .write_all(&res)?,
        None => stdout().write_all(&res)?,
    }
    Ok(())
}

/// Encodes bytes either from the supplied file or from the stdin into an image file with a given base image.
pub fn encode_into_image(image: PathBuf, data: Option<PathBuf>, output: PathBuf, replace: bool) -> Result<(), Error> {
    // opening output file early so it'll error out fast when it exists or something
    let output = OpenOptions::new().write(true).truncate(true).create_new(!replace).open(output)?;

    let decoder = PngDecoder::new(File::open(image)?)?;

    let payload = match data {
        Some(data) => {
            let mut data = File::open(data)?;
            let mut payload = Vec::with_capacity(data.metadata()?.len() as usize);
            data.read_to_end(&mut payload)?;
            payload
        },
        _ => {
            let mut payload = Vec::with_capacity(256);
            stdin().read_to_end(&mut payload)?;
            payload
        },
    };

    let (width, height) = decoder.dimensions();
    let color_type = decoder.color_type();

    let mut carrier = decoder.into_reader()?;

    let mut data = hide_bytes(&payload, &mut carrier, Bits::Two)?;
    data.reserve_exact(payload.len() - data.len());

    carrier.read_to_end(&mut data)?;

    PngEncoder::new(output).encode(&data, width, height, color_type)?;

    Ok(())
}
