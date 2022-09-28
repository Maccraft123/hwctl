use hwctl::{
    sysfs::{
        Backlight,
        SysfsDevice,
    },
};

use clap::Parser;
use anyhow::Result;
use anyhow::anyhow;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'B', long, value_parser)]
    inc_brightness: Option<i16>,

    #[clap(short = 'b', long, value_parser)]
    set_brightness: Option<u8>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(delta) = args.inc_brightness {
        let backlight: Vec<Backlight> = Backlight::enumerate_all()?;
        if backlight.len() == 0 {
            return Err(anyhow!("Failed to find backlight device or found too many"))
        }

        backlight[0].inc_bl(delta)?;
    }
    
    if let Some(val) = args.set_brightness {
        let backlight: Vec<Backlight> = Backlight::enumerate_all()?;
        if backlight.len() == 0 {
            return Err(anyhow!("Failed to find backlight device or found too many"))
        }

        backlight[0].set_bl(val)?;
    }
    Ok(())
}
