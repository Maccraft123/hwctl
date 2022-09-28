use hwctl::{
    sysfs::{
        Block,
        SysfsDevice,
    },
};

fn main() -> Result<(), std::io::Error> {
    let blocks = Block::enumerate_all()?;

    for block in blocks {
        println!("Found block device:");
        println!("sysfs path: {}", block.path().display());
        println!("dev path: {:?}", block.dev_path());
        println!("Name: {:?}", block.fancy_name());
        println!("Is a partition: {:?}", block.is_partition());
        println!("Size: {:?} GB", block.size_gigabytes());
        if !block.is_partition()? {
            println!("Partitions:");
            for part in block.partitions() {
                println!("{}:", part.dev_path().unwrap().display());
                println!("Size: {:?} GB", part.size_gigabytes());
                println!("");
            }
        }
        println!("");
    }

    Ok(())
}
