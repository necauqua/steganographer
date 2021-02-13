use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub enum Opt {
    /// Encodes data into the image
    #[structopt(name = "encode")]
    Encode {
        /// Original image file
        #[structopt(parse(from_os_str))]
        image: PathBuf,
        /// File with the data to be encoded
        #[structopt(parse(from_os_str))]
        data: PathBuf,
        /// Resulting image with the data hidden in it. If not supplied then the data is read from the stdin
        #[structopt(parse(from_os_str))]
        result: Option<PathBuf>,
        /// Replace the destination file if it already exists
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
    /// Decodes data that was hidden in the image
    #[structopt(name = "decode")]
    Decode {
        /// Image file with hidden data
        #[structopt(parse(from_os_str))]
        encoded: PathBuf,
        /// File to store the extracted data. If not supplied then the data is printed to stdout
        #[structopt(parse(from_os_str))]
        data: Option<PathBuf>,
        /// Replace the destination file if it already exists
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
}
