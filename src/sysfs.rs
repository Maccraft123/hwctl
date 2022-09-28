use std::{
    io,
    fs,
    path::PathBuf,
    ops,
};

fn map<T: Copy + PartialOrd + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + ops::Add<Output = T>>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T{
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub trait SysfsDevice {
    fn class() -> String;

    fn path(&self) -> PathBuf;
    fn enumerate_all() -> Result<Vec<Self>, io::Error>
        where Self: Sized + Send + Sync
    {
        let mut out = Vec::new();
        let class_path = format!("/sys/class/{}/", Self::class());
        let devices = fs::read_dir(class_path)?;
        for device in devices {
            if let Some(new) = Self::try_from_path(device?.path()) {
                out.push(new);
            }
        }
        Ok(out)
    }

    fn dev_path(&self) -> Option<PathBuf> {
        None
    }

    fn from_path(path: PathBuf) -> Self;

    fn try_from_path(path: PathBuf) -> Option<Self> 
        where Self: Sized + Send + Sync
    {
        Some(Self::from_path(path))
    }

    fn get(&self, key: &str) -> Result<String, io::Error> {
        let path = format!("{}/{}", self.path().to_string_lossy(), key);
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    fn get_i32(&self, key: &str) -> Result<Option<i32>, io::Error> {
        let path = format!("{}/{}", self.path().to_string_lossy(), key);
        let string = fs::read_to_string(path)?.trim().to_string();
        Ok(i32::from_str_radix(&string, 10).ok())
    }

    fn get_device(&self, key: &str) -> Result<String, io::Error> {
        let path = format!("{}/device/{}", self.path().to_string_lossy(), key);
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    fn set<T>(&self, key: &str, value: T) -> Result<(), io::Error>
        where T: ToString
    {
        let path = format!("{}/{}", self.path().to_string_lossy(), key);
        fs::write(path, value.to_string())
    }
}

#[derive(Debug)]
pub struct Backlight {
    path: PathBuf,
}

impl SysfsDevice for Backlight {
    fn class() -> String {
        "backlight".to_string()
    }
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
    fn from_path(path: PathBuf) -> Backlight {
        Self {path}
    }
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
        self.get_i32("max_brightness").map(|v| v.unwrap_or(0))
    }

    fn cur_val(&self) -> Result<u8, io::Error> {
        let val = i32::from_str_radix(&self.get("brightness")?, 10).unwrap_or(0);
        Ok(self.map_to_u8(val)?)
    }

    #[inline]
    pub fn inc_bl(&self, val: i16) -> Result<(), io::Error> {
        let new = if val >= 0 {
            u8::saturating_add(self.cur_val()?, val.try_into().unwrap())
        } else {
            u8::saturating_sub(self.cur_val()?, (val * -1).try_into().unwrap())
        };

        self.set_bl(new)
    }
    #[inline]
    pub fn set_bl(&self, val: u8) -> Result<(), io::Error> {
        self.set("brightness", self.map_from_u8(val)?)
    }
}

#[derive(Debug)]
pub struct Block {
    path: PathBuf,
    device: String,
}

impl SysfsDevice for Block {
    fn class() -> String {
        "block".to_string()
    }
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
    fn from_path(path: PathBuf) -> Block {
        let device = path.file_name().unwrap_or_default().to_str().unwrap().to_string();
        Self {
            path,
            device,
        }
    }
    fn dev_path(&self) -> Option<PathBuf> {
        let path = PathBuf::from("/dev/").join(&self.device);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

impl Block {
    pub fn fancy_name(&self) -> Option<String> {
        // trim because there can be lots of spaces on either side and there's a newline randomly
        let model = self.get_device("model").unwrap_or_default();
        let vendor = self.get_device("vendor").unwrap_or_default();

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

    pub fn is_partition(&self) -> Result<bool, io::Error> {
        // didn't find any other way
        let has_start = self.get("start").is_ok();
        let has_partition = self.get("partition").is_ok();
        Ok(has_partition || has_start)
    }

    pub fn partitions(&self) -> Vec<Block> {
        let mut ret = Vec::new();
        let self_dir_iter = fs::read_dir(&self.path).unwrap();
        for entry in self_dir_iter {
            let entry = entry.unwrap();
            let entryname = entry.file_name().into_string().unwrap();
            if entryname.starts_with(&self.device) {
                let path = PathBuf::from("/sys/class/block/".to_string() + &entryname);
                ret.push(Block::from_path(path))
            }
        }
        ret
    }

    pub fn size_bytes(&self) -> Option<u64> {
        if let Ok(Some(val)) = self.get_i32("size") {
            Some(val as u64 * 512)
        } else {
            None
        }
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
