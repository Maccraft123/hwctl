use hwctl::{
    sysfs::{
        Backlight,
        Sysfs,
        SysfsClass,
        SysfsDevice,
        SysfsInnerDevice,
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

    if let Some(new) = args.inc_brightness {
        let mut backlight: Vec<SysfsDevice> = SysfsClass::new("backlight")?.enum_devices()?.into_iter().collect();
        if backlight.len() == 0 {
            return Err(anyhow!("Failed to find backlight device"))
        }

        if let SysfsInnerDevice::Backlight(dev) = backlight.swap_remove(0).into_inner() {
            dev.inc(new)?;
        }
    }

    if let Some(new) = args.set_brightness {
        let mut backlight: Vec<SysfsDevice> = SysfsClass::new("backlight")?.enum_devices()?.into_iter().collect();
        if backlight.len() == 0 {
            return Err(anyhow!("Failed to find backlight device"))
        }

        if let SysfsInnerDevice::Backlight(dev) = backlight.swap_remove(0).into_inner() {
            dev.set(new)?;
        }
    }
    Ok(())
}
