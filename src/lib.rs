#![no_std]
#![warn(clippy::pedantic)]

macro_rules! make_endian {
    ($($type:ty),*) => {
        $(
            impl Endian<$type> for $type {
                fn to_be(&self) -> $type {
                    <$type>::to_be(*self)
                }

                fn to_le(&self) -> $type {
                    <$type>::to_le(*self)
                }

                fn from_be(value: $type) -> $type {
                    <$type>::from_be(value)
                }

                fn from_le(value: $type) -> $type {
                    <$type>::from_le(value)
                }
            }
        )*
    };
}

macro_rules! impl_traits {
    ($($type:ident),*) => {
        $(
            impl<T: Endian<T>> From<T> for $type<T> {
                #[inline]
                fn from(value: T) -> Self {
                    Self::new(value)
                }
            }
        )*
    };
}

pub trait Endian<T>
where
    Self: Into<T> + Copy + Clone + Send + Sync,
{
    /// Converts `self` to big endian from the target’s endianness.
    fn to_be(&self) -> T;

    /// Converts `self` to little endian from the target’s endianness.
    fn to_le(&self) -> T;

    /// Converts `value` from big endian to the target’s endianness.
    fn from_be(value: T) -> T;

    /// Converts `value` from little endian to the target’s endianness.
    fn from_le(value: T) -> T;
}

make_endian!(u8, u16, u32, u64, u128, usize);

#[derive(Default, Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct BigEndian<T: Endian<T>>(T);

impl<T: Endian<T>> BigEndian<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value.to_be())
    }

    /// Converts `self` from big endian to the target’s endianness.
    #[inline]
    pub fn to_native(&self) -> T {
        T::from_be(self.0)
    }

    #[inline]
    pub fn to_bits(&self) -> T {
        self.0
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct LittleEndian<T: Endian<T>>(T);

impl<T: Endian<T>> LittleEndian<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value.to_le())
    }

    /// Converts `self` from little endian to the target’s endianness.
    #[inline]
    pub fn to_native(&self) -> T {
        T::from_le(self.0)
    }

    #[inline]
    pub fn to_bits(&self) -> T {
        self.0
    }
}

impl_traits!(LittleEndian, BigEndian);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_to_native() {
        let value = BigEndian::new(12345u64);
        assert_eq!(value, 12345u64.into());
        assert_eq!(value.to_native(), 12345u64);

        let value = LittleEndian::new(12345u64);
        assert_eq!(value, 12345u64.into());
        assert_eq!(value.to_native(), 12345u64);
    }

    #[test]
    fn new_to_bits() {
        let value = BigEndian::new(0xfeu64);
        if cfg!(byte_endian = "big_endian") {
            assert_eq!(value.to_bits(), 0xfeu64);
        } else {
            assert_eq!(value.to_bits(), 0xfe00000000000000u64);
        }

        let value = LittleEndian::new(0xfeu64);
        if cfg!(byte_endian = "big_endian") {
            assert_eq!(value.to_bits(), 0xfe00000000000000u64);
        } else {
            assert_eq!(value.to_bits(), 0xfeu64);
        }
    }
}
