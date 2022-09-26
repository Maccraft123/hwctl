pub mod lib;
use hwctl::{
    Backlight,
    Sysfs,
    SysfsClass,
    SysfsDevice,
    SysfsInnerDevice,
};

use clap::Parser;
use anyhow::Result;
use anyhow::anyhow;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    verbose: bool,

    #[clap(short = 'B', long, value_parser)]
    inc_brightness: Option<i16>,

    #[clap(short = 'b', long, value_parser)]
    set_brightness: Option<u8>,

    #[clap(long)]
    list_block_devices: bool,
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

    if args.list_block_devices {
        for dev in SysfsClass::new("block")?.enum_devices()? {
            if let SysfsInnerDevice::Block(block) = dev.into_inner() {
                println!("Found block device:");
                println!("sysfs path: {}", block.inner.path.display());
                println!("dev path: {:?}", block.dev_path());
                println!("Name: {:?}", block.fancy_name());
                println!("Is a partition: {:?}", block.is_partition());
                println!("Size in gigabytes: {:?}", block.size_gigabytes());
                if !block.is_partition()? {
                    println!("Partitions:");
                    println!("{:#?}", block.partitions());
                }
                println!("");
            }
        }
    }

    Ok(())
}
