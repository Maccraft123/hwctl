use std::{
    io,
};

pub struct Brightness();

impl Brightness {
    #[inline]
    pub fn delta(val: i8) -> Result<u8, io::Error> {
        Ok(9)
    }

    #[inline]
    pub fn set(val: u8) -> Result<u8, io::Error> {
        Ok(0)
    }
}
