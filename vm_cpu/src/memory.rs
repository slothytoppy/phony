#[derive(Debug)]
pub struct Page {
    data: Vec<u8>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            data: vec![0; 4096],
        }
    }
}

impl Page {
    pub fn new(amount: PointerLen) -> Self {
        let len: u32 = u32::from(amount);
        Self {
            data: vec![0; len as usize],
        }
    }
}

#[derive(Debug, Default)]
pub struct Pager {
    pages: Vec<Page>,
}

impl Pager {
    pub fn alloc(&mut self, amount: u32) -> Pointer {
        let len = PointerLen::from(amount);
        self.pages.push(Page::new(len));
        let mut ptr = Pointer(0);
        ptr.set_page(self.pages.len().saturating_sub(1) as u8);

        ptr.set_len(len);

        ptr
    }

    pub fn memcpy(&mut self, ptr: Pointer, bytes: &[u8]) {
        let id = ptr.page();

        let page = self.pages.get_mut(id as usize).unwrap();

        for (i, byte) in bytes.iter().enumerate() {
            let by = page.data.get_mut(i).unwrap();
            *by = *byte;
        }
    }

    pub fn write(&mut self, ptr: Pointer, idx: u32, bytes: &[u8]) {
        let id = ptr.page();

        let page = self.pages.get_mut(id as usize).unwrap();

        for (i, byte) in bytes.iter().enumerate() {
            let by = page.data.get_mut(idx as usize + i).unwrap();
            *by = *byte;
        }
    }
}

#[derive(Debug, Default)]
pub struct CpuMemory {
    memory: Pager,
}

pub struct Pointer(u32);

impl Debug for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "page:{:?} ", self.page())?;
        write!(f, "len:{:?}", self.len())?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
/// the lower 24 bits of the u32 from a Pointer
pub struct PointerLen(u32);

impl std::ops::Add<u32> for PointerLen {
    type Output = PointerLen;
    fn add(self, rhs: u32) -> Self::Output {
        let rhs = PointerLen::from(rhs);
        PointerLen::from(self.0 + rhs.0)
    }
}

impl std::ops::BitOrAssign for PointerLen {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 &= 0xFF00_0000;
        self.0 |= rhs.0;
    }
}

impl PointerLen {
    fn len(&self) -> u32 {
        self.0 & 0x00FF_FFFF
    }

    const fn max() -> u32 {
        u32::MAX >> 8
    }
}

impl From<u32> for PointerLen {
    fn from(value: u32) -> Self {
        let len = value & 0x00FF_FFFF;
        assert!(len < PointerLen::max());
        PointerLen(len)
    }
}

impl From<PointerLen> for u32 {
    fn from(value: PointerLen) -> Self {
        value.0
    }
}

impl Pointer {
    fn page(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    fn set_page(&mut self, id: u8) {
        self.0 &= 0x00FF_FFFF;
        self.0 |= (id as u32) << 24; // Set upper 8 bits
    }

    fn len(&self) -> PointerLen {
        PointerLen::from(self.0)
    }

    fn set_len(&mut self, len: PointerLen) {
        let mut lhs = PointerLen::from(self.0);
        lhs |= len;

        self.0 |= lhs.0;
    }
}

impl CpuMemory {
    pub fn memcpy(&mut self, ptr: Pointer, bytes: Option<&[u8]>) {}
    pub fn alloc(&mut self, amount: u32) {}

    pub fn read<A>(&self, address: A) -> Result<u8, Error>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }

    pub fn read_u16<A>(&self, address: A) -> Result<u16, Error>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }

    pub fn read_u32<A>(&self, address: A) -> Result<u32, Error>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }

    pub fn write<A>(&mut self, address: A, byte: impl Into<u8>) -> Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        todo!()
    }

    pub fn write_bytes<'a, A>(
        &mut self,
        address: A,
        bytes: impl Into<&'a [u8]>,
    ) -> Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        let addr = address.into();
        let bytes: &[u8] = bytes.into();

        for (i, byte) in bytes.iter().enumerate() {
            self.write(addr + Address::from(i), *byte)?;
        }

        Ok(())
    }

    pub fn write_u16<A>(&mut self, address: A, num: u16) -> Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        let bytes = num.to_le_bytes();

        self.write_bytes(address, &bytes as &[u8])
    }

    pub fn write_u32<A>(&mut self, address: A, num: u32) -> Result<(), Error>
    where
        A: Into<Address> + Copy,
    {
        let bytes = num.to_le_bytes();

        self.write_bytes(address, &bytes as &[u8])
    }

    pub fn get(&self, bytes: std::ops::Range<Address>) -> Result<&[u8], Error> {
        todo!()
    }
}

use std::{fmt::Debug, ops::Range};

use tracing::{error, info, warn};

use crate::{
    memory::{self},
    registers::WordSize,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct Address(pub u32);

macro_rules! impl_address {
    ($($variant:ty),* $(,)?) => {

        impl From<Address> for $($variant)?{
            fn from(value: Address) -> $($variant)?{

                value.0 as $($variant)?
            }
        }

        impl From<$($variant)? > for Address{
            fn from(value: $($variant)?) -> Address {
                Address(value as u32)
            }
        }

        impl From<&$($variant)? > for Address{
            fn from(value: &$($variant)?) -> Address {
                Address(*value as u32)
            }
        }


    }
}

impl_address!(u8);
impl_address!(u16);
impl_address!(u32);
impl_address!(i32);
impl_address!(usize);

impl Address {
    pub fn next(&self) -> Result<Address, memory::Error> {
        let Some(addr) = self.0.checked_add(1) else {
            return Err(memory::Error::StackOverflow);
        };
        Ok(Address(addr))
    }

    pub fn prev(&self) -> Result<Address, memory::Error> {
        let Some(addr) = self.0.checked_sub(1) else {
            return Err(memory::Error::StackUnderflow);
        };
        Ok(Address(addr))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::ops::Add for Address {
    type Output = Address;

    fn add(self, rhs: Self) -> Self::Output {
        Address::from(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Address {
    type Output = Address;

    fn sub(self, rhs: Self) -> Self::Output {
        Address::from(self.0 - rhs.0)
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidAddress(u32),
    StackOverflow,
    StackUnderflow,
}

#[cfg(test)]
mod test {
    use tracing::{info, level_filters::LevelFilter};
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::memory::PointerLen;

    use super::{Pager, Pointer};

    fn setup_logger() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(LevelFilter::INFO)
            .finish()
            .try_init();
    }

    #[test]
    fn ptr_len() {
        setup_logger();

        let mut ptr = Pointer(0);
        ptr.set_page(10);
        ptr.set_len(crate::memory::PointerLen::from(1));

        assert_eq!(ptr.len().len(), 1);
        assert_eq!(ptr.page(), 10);
        info!(?ptr);
    }

    #[test]
    fn memcpy() {
        setup_logger();

        let mut mem = Pager::default();
        let ptr = mem.alloc(10);
        mem.memcpy(ptr, &[5; 10]);
        assert_eq!(mem.pages[0].data, [5; 10]);
        info!(?mem);
    }

    #[test]
    fn write() {
        setup_logger();

        let mut mem = Pager::default();
        let ptr = mem.alloc(10);
        mem.write(ptr, 5, &[5; 5]);
        assert_eq!(mem.pages[0].data, [0, 0, 0, 0, 0, 5, 5, 5, 5, 5]);
    }

    #[test]
    fn max() {
        setup_logger();
        let max = PointerLen::max();
        info!(?max);
        assert_eq!(max, 16777215)
    }
}
