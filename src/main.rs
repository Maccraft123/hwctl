pub mod lib;
use lib::{
    Brightness,
};

use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'b', long, value_parser)]
    brightness_delta: Option<i8>,

    #[clap(short = 'B', long, value_parser)]
    brightness: Option<u8>,

    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(brightness) = args.brightness {
        Brightness::set(brightness)?;
    }

    if let Some(brightness_delta) = args.brightness_delta {
    }

    Ok(())
}
