macro_rules! __impl_addr_hash_eq {
    (for<$($binder:lifetime),* $(,)?> $ty:ty) => {
        impl<$($binder),*> ::core::hash::Hash for $ty {
            fn hash<__H: ::core::hash::Hasher>(&self, __state: &mut __H) {
                core::ptr::hash(self as *const Self as *const (), __state)
            }
        }

        impl<$($binder),*> ::core::cmp::PartialEq for $ty {
            fn eq(&self, __other: &Self) -> bool {
                core::ptr::addr_eq(self, __other)
            }
        }

        impl<$($binder),*> ::core::cmp::Eq for $ty{}
    };
}

use std::num::NonZeroU64;

pub(crate) use __impl_addr_hash_eq;

macro_rules! impl_singleton_hash_eq {
    ($main:path) => {
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::marker::Send + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::marker::Sync + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::marker::Send + ::core::marker::Sync + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::panic::RefUnwindSafe + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::marker::Send + ::core::panic::RefUnwindSafe + '__a);
        crate::helpers::__impl_addr_hash_eq!(for<'__a> dyn $main + ::core::marker::Send + ::core::marker::Sync + ::core::panic::RefUnwindSafe + '__a);
    }
}

pub(crate) use impl_singleton_hash_eq;

macro_rules! impl_identity_debug_display{
    ($main:path) => {
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::marker::Send + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::marker::Sync + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::marker::Send + ::core::marker::Sync + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::panic::RefUnwindSafe + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::marker::Send + ::core::panic::RefUnwindSafe + '__a => ::core::stringify!($main));
        crate::helpers::__impl_identity_debug_diplay!(for<'__a> dyn $main + ::core::marker::Send + ::core::marker::Sync + ::core::panic::RefUnwindSafe + '__a => ::core::stringify!($main));
    }
}

pub(crate) use impl_identity_debug_display;

macro_rules! __impl_identity_debug_diplay {
    (for<$($binder:lifetime),* $(,)?> $ty:ty => $e:expr) => {
        impl<$($binder),*> ::core::fmt::Debug for $ty {
            fn fmt(&self, __fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                const __DEBUG_TY_NAME: &str = $e;
                let __name = <$ty as crate::traits::IdentityName>::name(self);
                __fmt.debug_tuple(__DEBUG_TY_NAME)
                    .field(&__name)
                    .finish_non_exhaustive()

            }
        }

        impl<$($binder),*> ::core::fmt::Display for $ty {
            fn fmt(&self, __fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let __name = <$ty as crate::traits::IdentityName>::name(self);
                __fmt.write_str(__name)
            }
        }
    }
}

pub(crate) use __impl_identity_debug_diplay;

#[derive(Debug)]
pub enum IdDowncastError {
    WrongType,
    MalformedValue,
}

macro_rules! def_id_type {
    ($ty_name:ident) => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub struct $ty_name(::core::num::NonZeroU64, u64);

        impl $ty_name {
            #[doc(hidden)]
            pub const fn __new_unchecked(ty: ::core::num::NonZeroU64, n: u64) -> Self {
                Self(ty, n)
            }

            pub fn new<I: $crate::traits::IntoId<Self>>(val: I) -> Self {
                Self(I::__ID_TYPE, val.__into_id())
            }

            pub fn downcast<I: $crate::traits::IntoId<Self>>(
                self,
            ) -> Result<I, $crate::helpers::IdDowncastError> {
                if self.0 == I::__ID_TYPE {
                    I::__try_downcast(self.1)
                        .ok_or($crate::helpers::IdDowncastError::MalformedValue)
                } else {
                    Err($crate::helpers::IdDowncastError::WrongType)
                }
            }
        }

        impl $crate::traits::into_id::IdType for $ty_name {}

        pub use cmli_derive::$ty_name;
    };
}

pub(crate) use def_id_type;
