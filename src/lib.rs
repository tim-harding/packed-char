//! [`PackedChar`] stores either a [`char`] or a [`U22`] in 32 bits of space.
//!
//! # Examples
//!
//! ```
//! use packed_char::{PackedChar, U22, Contents};
//! # use packed_char::U22FromU32Error;
//! # fn main() -> Result<(), U22FromU32Error> {
//! assert_eq!(PackedChar::from('a').contents(), Contents::Char('a'));
//! assert_eq!(PackedChar::try_from(42)?.contents(), Contents::U22(U22::from_u32(42)?));
//! # Ok(()) }
//! ```

#![no_std]

mod u22;
pub use u22::{U22FromU32Error, U22};

use core::fmt::{self, Debug, Formatter};

/// Stores either a `char` or a [`U22`] in 32 bits of space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PackedChar(u32);

impl PackedChar {
    const SURROGATE_LOW: u32 = 0xD800;
    const SURROGATE_HIGH: u32 = 0xDFFF;
    const SURROGATE_MASK: u32 = Self::SURROGATE_LOW & Self::SURROGATE_HIGH;
    const LEADING: u32 = (char::MAX as u32).leading_zeros(); // 11
    const LEADING_MASK: u32 = !(u32::MAX >> Self::LEADING);
    const TRAILING: u32 = Self::SURROGATE_LOW.trailing_zeros(); // 11
    const TRAILING_MASK: u32 = !(u32::MAX << Self::TRAILING);
    const MAX_U22_LEADING: u32 = U22::MAX.leading_zeros();

    /// Creates a new value from the given `char`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::{PackedChar, Contents};
    /// let pack = PackedChar::from_char('a');
    /// assert_eq!(pack.contents(), Contents::Char('a'));
    /// ```
    pub const fn from_char(c: char) -> Self {
        Self(c as u32)
    }

    /// Creates a new value from the given `u22`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::{PackedChar, Contents, U22};
    /// let u22 = U22::from_u32(42).unwrap();
    /// let pack = PackedChar::from_u22(u22);
    /// assert_eq!(pack.contents(), Contents::U22(u22));
    /// ```
    pub const fn from_u22(u22: U22) -> Self {
        let n = u22.as_u32();
        let leading = (n << Self::MAX_U22_LEADING) & Self::LEADING_MASK;
        let trailing = n & Self::TRAILING_MASK;
        Self(leading | trailing | Self::SURROGATE_MASK)
    }

    /// Gets the stored value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::{PackedChar, Contents, U22, U22FromU32Error};
    /// # fn main() -> Result<(), U22FromU32Error> {
    /// let pack = PackedChar::try_from(42)?;
    /// assert_eq!(pack.contents(), Contents::U22(U22::from_u32(42)?));
    ///
    /// let pack = PackedChar::from('a');
    /// assert_eq!(pack.contents(), Contents::Char('a'));
    /// # Ok(())
    /// # }
    /// ```
    pub const fn contents(self) -> Contents {
        match char::from_u32(self.0) {
            Some(c) => Contents::Char(c),
            None => {
                let trailing = self.0 & Self::TRAILING_MASK;
                let leading = self.0 & Self::LEADING_MASK;
                let u22 = trailing | (leading >> Self::MAX_U22_LEADING);
                // SAFETY: Valid by construction since we reversed the storage procedure.
                Contents::U22(unsafe { U22::from_u32_unchecked(u22) })
            }
        }
    }
}

impl Debug for PackedChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.contents())
    }
}

impl From<char> for PackedChar {
    fn from(c: char) -> Self {
        Self::from_char(c)
    }
}

impl From<U22> for PackedChar {
    fn from(u22: U22) -> Self {
        Self::from_u22(u22)
    }
}

impl TryFrom<u32> for PackedChar {
    type Error = U22FromU32Error;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        let u22 = U22::from_u32(n)?;
        Ok(Self::from_u22(u22))
    }
}

/// The contents of a [`PackedChar`].
///
/// Returned from [`PackedChar::contents`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Contents {
    Char(char),
    U22(U22),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_back_chars() {
        let test_chars = [
            '\0',
            '\u{D7FF}',
            '\u{E000}',
            // Char containing surrogate mask
            #[allow(clippy::unusual_byte_groupings)]
            char::from_u32(0b1_11011_11111111111).unwrap(),
            // Char not containing surrogate mask
            #[allow(clippy::unusual_byte_groupings)]
            char::from_u32(0b1_00000_11111111111).unwrap(),
            char::REPLACEMENT_CHARACTER,
            char::MAX,
            'a',
            '1',
            '🫠',
        ];
        for c in test_chars {
            let packed = PackedChar::from_char(c);
            assert_eq!(packed.contents(), Contents::Char(c));
        }
    }

    #[test]
    fn gets_back_ints() {
        let ints = [U22::MAX, 0x3FFFFF, 0, 42, 0b1010101010101010101010];
        for i in ints {
            let packed = PackedChar::try_from(i).unwrap();
            assert_eq!(packed.contents(), Contents::U22(U22::try_from(i).unwrap()));
        }
    }

    #[test]
    fn fails_out_of_bounds_indices() {
        let ints = [U22::MAX + 1, u32::MAX, 0b10101010101010101010101010101010];
        for i in ints {
            let packed = PackedChar::try_from(i);
            assert_eq!(packed, Err(U22FromU32Error(i)));
        }
    }
}
