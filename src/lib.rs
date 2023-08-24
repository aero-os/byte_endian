#![no_std]
#![warn(clippy::pedantic)]

macro_rules! impl_traits {
    // Implements `Endian` for the given endian types and implements the
    // necessary traits for the big and little endian types ($l and $b).
    ($($endian_type:ident),+ => $l:ident, $b:ident) => {
        $(
            impl Endian<$endian_type> for $endian_type {
                fn to_be(&self) -> $endian_type {
                    <$endian_type>::to_be(*self)
                }

                fn to_le(&self) -> $endian_type {
                    <$endian_type>::to_le(*self)
                }

                fn from_be(value: $endian_type) -> $endian_type {
                    <$endian_type>::from_be(value)
                }

                fn from_le(value: $endian_type) -> $endian_type {
                    <$endian_type>::from_le(value)
                }
            }
        )+

        impl_traits!(@make_impl $($endian_type),+ => $l);
        impl_traits!(@make_impl $($endian_type),+ => $b);
    };

    // Implements `From<T> for $type<T>` and `From<$type<T>> for T` where T
    // is a subtype of Endian<T> and $type is either big or little endian.
    (@make_impl $($endian_type:ident),+ => $type:ident) => {
        impl<T: Endian<T>> From<T> for $type<T> {
            #[inline]
            fn from(value: T) -> Self {
                Self::new(value)
            }
        }

        $(
            impl From<$type<$endian_type>> for $endian_type {
                #[inline]
                fn from(value: $type<$endian_type>) -> Self {
                    value.to_native()
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
    pub fn to_native(self) -> T {
        T::from_be(self.0)
    }

    #[inline]
    pub fn to_bits(self) -> T {
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
    pub fn to_native(self) -> T {
        T::from_le(self.0)
    }

    #[inline]
    pub fn to_bits(self) -> T {
        self.0
    }
}

impl_traits!(u8, u16, u32, u64, u128, usize => LittleEndian, BigEndian);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inside_packed() {
        // This tests that no references are created by calling any of the methods on
        // the endian types. Since, creating references to a packed field (which is not
        // aligned) is undefined behaviour (`E0793`).
        #[repr(C, packed)]
        struct Packet(BigEndian<u64>, LittleEndian<u64>);

        let packet = Packet(0xfe.into(), 0xfe.into());

        assert_eq!(packet.0.to_native(), 0xfe);
        if cfg!(byte_endian = "big_endian") {
            assert_eq!(packet.0.to_bits(), 0xfeu64);
        } else {
            assert_eq!(packet.0.to_bits(), 0xfe00000000000000u64);
        }

        assert_eq!(packet.1.to_native(), 0xfe);
        if cfg!(byte_endian = "big_endian") {
            assert_eq!(packet.1.to_bits(), 0xfe00000000000000u64);
        } else {
            assert_eq!(packet.1.to_bits(), 0xfeu64);
        }
    }

    #[test]
    fn new_to_native() {
        let be_value = BigEndian::new(12345u64);
        assert_eq!(be_value, 12345u64.into());
        assert_eq!(be_value.to_native(), 12345u64);

        let be_native: u64 = be_value.into();
        assert_eq!(be_native, 12345u64);

        let le_value = LittleEndian::new(12345u64);
        assert_eq!(le_value, 12345u64.into());
        assert_eq!(le_value.to_native(), 12345u64);

        let native: u64 = le_value.into();
        assert_eq!(native, 12345u64);
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
