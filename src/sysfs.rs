
use std::{
    io,
    fs,
    path::PathBuf,
    convert,
    ops,
};

fn map<T: Copy + PartialOrd + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + ops::Add<Output = T>>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T{
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}



pub struct Sysfs();

impl Sysfs {
    pub fn enum_classes() -> Result<Vec<SysfsClass>, io::Error> {
        let mut ret = Vec::new();
        for entry in fs::read_dir("/sys/class")? {
            if let Some(class) = SysfsClass::from_path(&entry?.path()) {
                ret.push(class);
            }
        }

        Ok(ret)
    }
}

#[derive(Clone, Debug)]
pub struct SysfsClass {
    class: String,
    path: PathBuf,
}

impl SysfsClass {
    pub fn from_path(path: &PathBuf) -> Option<SysfsClass> {
        let path_string = path.to_string_lossy();
        let path_split: Vec<&str> = path_string.split("/").collect();
        if let ["", "sys", "class", class] = &path_split[..] {
            Some(Self {
                class: class.to_string(),
                path: path.clone(),
            })
        } else {
            None
        }
    }
    pub fn new(name: &str) -> Result<SysfsClass, io::Error> {
        let path = PathBuf::from(format!("/sys/class/{}/", name));
        if path.exists() {
            Ok(Self {
                class: name.to_string(),
                path: path
            })
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))
        }
    }

    pub fn enum_devices(&self) -> Result<Vec<SysfsDevice>, io::Error> {
        let mut ret = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            if let Some(device) = SysfsDevice::from_path(&entry?.path()) {
                ret.push(device.clone())
            }
        }

        Ok(ret)
    }
}

#[derive(Debug, Clone)]
pub struct SysfsDevice {
    pub class: String,
    pub device: String,
    pub path: PathBuf,
}

impl SysfsDevice {
    pub fn from_path(path: &PathBuf) -> Option<SysfsDevice> {
        let path_string = path.to_string_lossy();
        let path_split: Vec<&str> = path_string.split("/").collect();
        if let ["", "sys", "class", class, device] = &path_split[..] {
            Some(Self {
                class: class.to_string(),
                device: device.to_string(),
                path: path.clone(),
            })
        } else {
            None
        }
    }

    pub fn get_value(&self, val: &str) -> Result<String, io::Error> {
        let path = format!("/sys/class/{}/{}/{}", &self.class, &self.device, val);
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    pub fn set_value(&self, val: &str, data: &str) -> Result<(), io::Error> {
        let path = format!("/sys/class/{}/{}/{}", &self.class, &self.device, val);
        fs::write(path, data)
    }

    pub fn get_device_value(&self, val: &str) -> Result<String, io::Error> {
        let path = format!("/sys/class/{}/{}/device/{}", &self.class, &self.device, val);
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    pub fn into_inner(self) -> SysfsInnerDevice {
        match self.class.as_str() {
            "backlight" => SysfsInnerDevice::Backlight(Backlight{inner: self}),
            "block" => SysfsInnerDevice::Block(Block{inner: self}),
            class => SysfsInnerDevice::Other(class.to_string(), self),
        }
    }
}

#[derive(Debug)]
pub enum SysfsInnerDevice {
    Backlight(Backlight),
    Block(Block),
    Bluetooth,
    Firmware,
    Hwmon,
    I2cDev,
    Input,
    Leds,
    Net,
    PowerSupply,
    Thermal,
    Other(String, SysfsDevice),
}

#[derive(Debug)]
pub struct Backlight {
    inner: SysfsDevice,
}

impl Backlight {
    fn map_to_u8(&self, val: i32) -> Result<u8, io::Error> {
        let max = self.max_brightness()?;
        Ok(map::<i32>(val, 0, max, 0, 255).try_into().unwrap())
    }

    fn map_from_u8(&self, val: u8) -> Result<i32, io::Error> {
        let max = self.max_brightness()?;
        Ok(map(val as i32, 0, 255, 0, max))
    }

    fn max_brightness(&self) -> Result<i32, io::Error> {
        Ok(i32::from_str_radix(&self.inner.get_value("max_brightness")?, 10).unwrap_or_default())
    }

    fn cur_val(&self) -> Result<u8, io::Error> {
        let max = self.max_brightness()?;
        let min = 0;
        let val = u8::from_str_radix(&self.inner.get_value("brightness")?, 10).unwrap_or_default();
        Ok(map(val as i32, min, max, 0 as i32, 255 as i32).try_into().unwrap())
    }

    #[inline]
    pub fn inc(&self, val: i16) -> Result<(), io::Error> {
        let new = if val >= 0 {
            u8::saturating_add(self.cur_val()?, val.try_into().unwrap())
        } else {
            u8::saturating_sub(self.cur_val()?, (val * -1).try_into().unwrap())
        };

        self.set(new)
    }

    #[inline]
    pub fn set(&self, val: u8) -> Result<(), io::Error> {
        self.inner.set_value("brightness", &self.map_from_u8(val.into())?.to_string())
    }
}

#[derive(Debug)]
pub struct Block {
    pub inner: SysfsDevice,
}

impl Block {
    #[inline]
    pub fn dev_path(&self) -> Option<PathBuf> {
        let path: PathBuf = ["/dev/", &self.inner.device].iter().collect();
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[inline]
    pub fn fancy_name(&self) -> Option<String> {
        // trim because there can be lots of spaces on either side and there's a newline randomly
        let model = self.inner.get_device_value("model").unwrap_or_default();
        let vendor = self.inner.get_device_value("vendor").unwrap_or_default();

        // if both empty we got nothing
        if model.is_empty() && vendor.is_empty() {
            return None;
        }

        // if one of them empty return both concatencated, so non-empty one
        if model.is_empty() || vendor.is_empty() {
            return Some(format!("{}{}", vendor, model));
        }

        // if both have something return concatencated with space
        Some(format!("{} {}", vendor, model))
    }

    #[inline]
    pub fn is_partition(&self) -> Result<bool, io::Error> {
        // didn't find any other way
        let has_start = self.inner.get_value("start").is_ok();
        let has_partition = self.inner.get_value("partition").is_ok();
        Ok(has_partition || has_start)
    }

    #[inline]
    pub fn partitions(&self) -> Option<Vec<Block>> {
        let mut ret = Vec::new();
        let self_dir_iter = fs::read_dir(&self.inner.path).unwrap();
        for entry in self_dir_iter {
            let entry = entry.unwrap();
            let entryname = entry.file_name().into_string().unwrap();
            if entryname.starts_with(&self.inner.device) {
                let dev = SysfsDevice::from_path(&PathBuf::from("/sys/class/block/".to_string() + &entryname));
                if let SysfsInnerDevice::Block(out) = dev.unwrap().into_inner() {
                    ret.push(out)
                }
            }
        }

        if !ret.is_empty() {
            Some(ret)
        } else {
            None
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.inner.device
    }

    #[inline]
    pub fn size_bytes(&self) -> Option<u64> {
        if let Ok(val) = self.inner.get_value("size") {
            if let Ok(size) = u64::from_str_radix(&val, 10) {
                return Some(size*512);
            }
        }
        None
    }

    #[inline]
    pub fn size_kilobytes(&self) -> Option<u64> {
        self.size_bytes().map(|v| v/1000)
    }

    #[inline]
    pub fn size_megabytes(&self) -> Option<u64> {
        self.size_kilobytes().map(|v| v/1000)
    }
    
    #[inline]
    pub fn size_gigabytes(&self) -> Option<u64> {
        self.size_megabytes().map(|v| v/1000)
    }
}
