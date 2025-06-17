use std::{hash::Hash, num::NonZeroU64};

/// Subtrait for traits that have a name method that identifies a specific object uniquely
pub trait IdentityName {
    fn name(&self) -> &str;
}

/// Marker subtrait for traits that expect its implementors to be unique - that is, logically two equivalent instances (according to [`core::cmp::Eq`]).
/// Further, the trait may assume that instances of other implementors will compare unequal
///
/// ## Invariants
/// Every natural implementor of this trait (that is, not a `dyn Trait` type) must implement the [`Hash`][core::hash::Hash] and [`Eq`] traits and satisfy the invariants thereof.
/// In particular, two instances of the type that coexist at different addresses at any given time must compare unequal.
///
/// Note that because [`Unique`] is a safe trait, this is a logic/correctness invariant only:
/// Violating it can result in unexpected results from using subtraits, but will not cause undefined behaviour.
pub trait Unique: unique::HasHashEq {}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be converted to `{Id}`",
    note = "Add `#[derive({Id})]` to `{Self}`",
    note = "This can only be added to a `Copy` enum type that implements `Eq` and `Hash`"
)]
pub trait IntoId<Id: into_id::IdType>: Copy + Eq + Hash {
    #[doc(hidden)]
    const __ID_TYPE: NonZeroU64;
    #[doc(hidden)]
    fn __into_id(self) -> u64;
    #[doc(hidden)]
    fn __try_downcast(val: u64) -> Option<Self>;
}

mod unique {
    pub trait HasHashEq {}

    impl<R: ?Sized + ::core::cmp::Eq + ::core::hash::Hash> HasHashEq for R {}
}

pub(crate) mod into_id {

    #[diagnostic::on_unimplemented(
        message = "`{Self}` is not a valid ID type",
        note = "Only specific types can be used with `IntoId`"
    )]
    pub trait IdType: Copy + Eq + ::core::hash::Hash {}
}

pub unsafe trait AsU64: Sized {
    #[doc(hidden)]
    unsafe fn __value_from(v: u64) -> Option<Self>;
}

unsafe impl AsU64 for u64 {
    unsafe fn __value_from(v: u64) -> Option<Self> {
        Some(v)
    }
}
unsafe impl AsU64 for NonZeroU64 {
    unsafe fn __value_from(v: u64) -> Option<Self> {
        Self::new(v)
    }
}
