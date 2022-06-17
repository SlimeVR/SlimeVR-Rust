//! Contains trivial "newtype" wrappers

use crate::prelude::*;

/// A newtype on `T` that indicates that it is a global transform.
#[derive(Debug, PartialEq, Default)]
pub struct Global<T: private::Sealed>(pub T);

/// Implements `From<T> for $ident<T>`
macro_rules! impl_helper {
    ($ident:ident) => {
        impl<T: private::Sealed> From<T> for $ident<T> {
            fn from(other: T) -> Self {
                Self(other)
            }
        }
    };
}
impl_helper!(Global);
impl_helper!(Local);

/// A newtype on `T` that indicates that it is a local transform.
#[derive(Debug, PartialEq, Default)]
pub struct Local<T: private::Sealed>(pub T);

mod private {
    use super::*;

    /// Private helper trait to limit the types that can go in [`Global`] or [`Local`].
    ///
    /// For more info about this pattern, see
    /// [here](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
    pub trait Sealed {}
    impl Sealed for Translation {}
    impl Sealed for UnitQuat {}
    impl Sealed for Point {}
}
