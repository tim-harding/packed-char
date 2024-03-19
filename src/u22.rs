use std::{
    borrow::Borrow,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Deref,
};

/// A 22-bit unsigned integer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U22(u32);

impl Display for U22 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl U22 {
    /// The largest value that can be expressed by this type
    pub const MAX: u32 = !(u32::MAX << 22);

    /// Creates a new 22-bit integer from the given 32-bit integer if it is small enough to fit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::{U22, U22FromU32Error};
    /// assert_eq!(U22::from_u32(42).map(U22::as_u32), Ok(42));
    /// assert_eq!(U22::from_u32(U22::MAX).map(U22::as_u32), Ok(U22::MAX));
    /// assert_eq!(U22::from_u32(U22::MAX + 1), Err(U22FromU32Error(U22::MAX + 1)));
    /// ```
    pub const fn from_u32(n: u32) -> Result<Self, U22FromU32Error> {
        if n > Self::MAX {
            Err(U22FromU32Error(n))
        } else {
            Ok(Self(n))
        }
    }

    /// Creates a new 22-bit integer from the given 32-bit integer.
    ///
    /// # Safety
    ///
    /// The provided integer must be no greater than [`U22::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::U22;
    /// let u22 = unsafe { U22::from_u32_unchecked(42) };
    /// assert_eq!(u22.as_u32(), 42);
    /// ```
    pub const unsafe fn from_u32_unchecked(n: u32) -> Self {
        Self(n)
    }

    /// Gets the 22-bit integer as a 32-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use packed_char::U22;
    /// let u22 = U22::from_u32(42).unwrap();
    /// assert_eq!(u22.as_ref(), &42);
    /// ```
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for U22 {
    type Error = U22FromU32Error;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        Self::from_u32(n)
    }
}

impl From<U22> for u32 {
    fn from(u22: U22) -> Self {
        u22.0
    }
}

impl AsRef<u32> for U22 {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl Borrow<u32> for U22 {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

impl Deref for U22 {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Error type for 32-bit to 22-bit integer conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U22FromU32Error(
    /// The `u32` that failed to be converted to a [`U22`].
    pub u32,
);

impl Display for U22FromU32Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} exceeds U22::MAX", self.0)
    }
}

impl Error for U22FromU32Error {}
