extern crate structopt;

use structopt::clap::{crate_name, Shell};

include!("src/cli.rs");

fn main() {
    if let Some(dir) = std::env::var_os("OUT_DIR") {
        Shell::variants().iter()
            .filter_map(|variant| variant.parse().ok())
            .for_each(|shell| Opt::clap().gen_completions(crate_name!(), shell, &dir));
    }
}
