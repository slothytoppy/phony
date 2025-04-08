use crate::memory::{self};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Address(u32);

macro_rules! impl_address {
    ($($variant:ty),* $(,)?) => {

        //impl TryFrom<$($variant)?> for Address {
        //    type Error = crate::memory::Error;
        //
        //    fn try_from(value: $($variant)?) -> Result<Address, Self::Error> {
        //        Ok(Address(value as u32))
        //    }
        //}

        //impl TryFrom<&$($variant)?> for Address{
        //    type Error = crate::memory::Error;
        //
        //    fn try_from(value: &$($variant)?) -> Result<Address, Self::Error> {
        //        Address::try_from(*value)
        //    }
        //}

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
