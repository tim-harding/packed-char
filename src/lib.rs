//! [`PackedChar`] allows either a `char` or up to 22 bits of other information to be stored in 32
//! bits of space.
//!
//! # Details
//!
//! To determine what type of data a [`PackedChar`] holds, we take advantage of the valid ranges
//! for a `char`, which are `0..0xD800` and `0xDFFF..0x10FFFF` (see the documentation for
//! [`char`]). The range `0xD800..=0xDFFF` contains surrogate code points, which are not valid
//! UTF-8 characters. We store `char`s in their normal representation. To store a [`U22`] without
//! overlapping valid `char` ranges, we first split it into two 11-bit chunks. The left chunk is
//! stored in the leading bits that `char`s never overlap with. The right chunk is stored in the
//! trailing bits, which do overlap the bits used by `char`s. To make this work, we make note of
//! the bit pattern in the surrogate range:
//!
//! ```text
//! 1101100000000000
//! 1101111111111111
//! ```
//!
//! Since the leading 5 bits are constant for this range, we set them along with the left and right
//! chunks of our 22 bits:
//!
//! ```text
//! 11111111111  00000    11011            11111111111
//! left chunk | unused | surrogate mask | right chunk
//! ```
//!
//! Now if we mask out the left chunk, the remaining bit pattern will never be a valid char because
//! it falls in the surrogate range. This disambiguates what the [`PackedChar`] contains.

mod u22;
pub use u22::{U22FromU32Error, U22};

use std::fmt::{self, Debug, Formatter};

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
    const CHAR_MASK: u32 = !Self::LEADING_MASK;
    const MAX_U22_LEADING: u32 = U22::MAX.leading_zeros();

    pub const fn from_char(c: char) -> Self {
        Self(c as u32)
    }

    pub const fn from_u22(u22: U22) -> Self {
        let n = u22.as_u32();
        let leading = (n << Self::MAX_U22_LEADING) & Self::LEADING_MASK;
        let trailing = n & Self::TRAILING_MASK;
        Self(leading | trailing | Self::SURROGATE_MASK)
    }

    pub fn contents(self) -> PackedCharContents {
        let c = self.0 & Self::CHAR_MASK;
        if !(Self::SURROGATE_LOW..=Self::SURROGATE_HIGH).contains(&c) {
            // TODO: Make this function const when from_u32_unchecked as const
            // is stablized.
            PackedCharContents::Char(unsafe { char::from_u32_unchecked(c) })
        } else {
            let i = self.0 & !Self::SURROGATE_MASK;
            let trailing = i & Self::TRAILING_MASK;
            let leading = i & Self::LEADING_MASK;
            PackedCharContents::U22(unsafe {
                U22::from_u32_unchecked(trailing | (leading >> Self::MAX_U22_LEADING))
            })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackedCharContents {
    Char(char),
    U22(U22),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_back_chars() {
        let test_chars = ['\0', '\u{D7FF}', '\u{E000}', '\u{10FFFF}', 'a', '1', '🫠'];
        for c in test_chars {
            let packed = PackedChar::from_char(c);
            assert_eq!(packed.contents(), PackedCharContents::Char(c));
        }
    }

    #[test]
    fn gets_back_indices() {
        let test_indices = [U22::MAX, 0x3FFFFFu32, 0, 69, 420, 0b1010101010101010101010];
        for i in test_indices {
            let packed = PackedChar::try_from(i).unwrap();
            assert_eq!(
                packed.contents(),
                PackedCharContents::U22(U22::try_from(i).unwrap())
            );
        }
    }

    #[test]
    fn fails_out_of_bounds_indices() {
        let test_indices = [U22::MAX + 1, 0b10101010101010101010101010101010];
        for i in test_indices {
            let packed = PackedChar::try_from(i);
            assert_eq!(packed, Err(U22FromU32Error(i)));
        }
    }
}
