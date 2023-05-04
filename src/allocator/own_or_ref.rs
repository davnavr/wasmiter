use core::convert::AsRef;

#[cfg(feature = "alloc")]
use alloc::borrow::{Cow, ToOwned};

/// A value or a reference to a value.
///
/// This is essentially a version of
/// [`Cow<'a, B>`](https://doc.rust-lang.org/alloc/borrow/enum.Cow.html) without requring `B`
/// to implement [`ToOwned`](https://doc.rust-lang.org/alloc/borrow/trait.ToOwned.html).
#[derive(Debug)]
pub enum OwnOrRef<'a, B: ?Sized, O: AsRef<B>> {
    /// A reference to a value.
    Reference(&'a B),
    /// An owned value.
    Owned(O),
}

impl<B: ?Sized, O: AsRef<B>> core::ops::Deref for OwnOrRef<'_, B, O> {
    type Target = B;

    fn deref(&self) -> &B {
        match self {
            Self::Reference(reference) => reference,
            Self::Owned(owned) => owned.as_ref(),
        }
    }
}

impl<B: ?Sized, O: AsRef<B>> AsRef<B> for OwnOrRef<'_, B, O> {
    #[inline]
    fn as_ref(&self) -> &B {
        self
    }
}

impl<B: ?Sized, O: AsRef<B>> core::borrow::Borrow<B> for OwnOrRef<'_, B, O> {
    #[inline]
    fn borrow(&self) -> &B {
        self
    }
}

impl<B: ?Sized + core::fmt::Display, O: AsRef<B>> core::fmt::Display for OwnOrRef<'_, B, O> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <B as core::fmt::Display>::fmt(self, f)
    }
}

impl<'a, B: ?Sized, O: AsRef<B>> From<&'a B> for OwnOrRef<'a, B, O> {
    #[inline]
    fn from(reference: &'a B) -> Self {
        Self::Reference(reference)
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for OwnOrRef<'_, str, alloc::string::String> {
    #[inline]
    fn from(string: alloc::string::String) -> Self {
        Self::Owned(string)
    }
}

#[cfg(feature = "alloc")]
impl<'a, B, O> From<OwnOrRef<'a, B, O>> for Cow<'a, B>
where
    B: ToOwned<Owned = O> + ?Sized + 'a,
    O: AsRef<B> + core::borrow::Borrow<B>,
{
    fn from(value: OwnOrRef<'a, B, O>) -> Self {
        match value {
            OwnOrRef::Owned(owned) => Cow::Owned(owned),
            OwnOrRef::Reference(borrow) => Cow::Borrowed(borrow),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, B> From<Cow<'a, B>> for OwnOrRef<'a, B, B::Owned>
where
    B: ToOwned + ?Sized + 'a,
    B::Owned: AsRef<B> + core::borrow::Borrow<B>,
{
    fn from(value: Cow<'a, B>) -> Self {
        match value {
            Cow::Owned(owned) => Self::Owned(owned),
            Cow::Borrowed(borrow) => Self::Reference(borrow),
        }
    }
}

impl<B: ?Sized, O: AsRef<B> + Clone> Clone for OwnOrRef<'_, B, O> {
    fn clone(&self) -> Self {
        match self {
            Self::Reference(reference) => Self::Reference(reference),
            Self::Owned(owned) => Self::Owned(owned.clone()),
        }
    }
}

impl<B1, O1, B2, O2> PartialEq<OwnOrRef<'_, B2, O2>> for OwnOrRef<'_, B1, O1>
where
    B1: ?Sized + PartialEq<B2>,
    O1: AsRef<B1>,
    B2: ?Sized,
    O2: AsRef<B2>,
{
    #[inline]
    fn eq(&self, other: &OwnOrRef<'_, B2, O2>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<B: ?Sized + Eq, O: AsRef<B>> Eq for OwnOrRef<'_, B, O> {}

impl<B: ?Sized + core::hash::Hash, O: AsRef<B>> core::hash::Hash for OwnOrRef<'_, B, O> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        <B as core::hash::Hash>::hash(self, state)
    }
}
