extern crate structopt;

use structopt::StructOpt;

use cli::Opt;
use steganographer::*;

mod cli;

fn main() -> Result<(), Error> {
    match Opt::from_args() {
        Opt::Encode { image, data, result, force } =>
            encode_into_image(image, data, result, force),
        Opt::Decode { encoded, data, force } =>
            decode_from_image(encoded, data, force),
    }
}
