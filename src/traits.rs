use std::{
    any::Any,
    hash::Hasher,
    num::{NonZero, NonZeroU64},
};

pub trait Name {
    fn name(&self) -> &'static str;
}

pub const unsafe trait AsRawId: Sized + Eq + 'static {
    #[doc(hidden)]
    const TYPE: NonZeroU64;

    #[doc(hidden)]
    fn into_raw_id(self) -> u64;

    #[doc(hidden)]
    fn from_raw_id(id: u64) -> Option<Self>;
}

#[doc(hidden)]
pub const unsafe trait TryAsU64Raw {
    fn is_valid(val: u64) -> bool;
}

unsafe impl const TryAsU64Raw for u64 {
    fn is_valid(_: u64) -> bool {
        true
    }
}

unsafe impl const TryAsU64Raw for NonZeroU64 {
    fn is_valid(val: u64) -> bool {
        val != 0
    }
}

#[doc(hidden)]
pub const fn try_from_u64<T: const TryAsU64Raw>(val: u64) -> Option<T> {
    if T::is_valid(val) {
        Some(unsafe { crate::mem::transmute(val) })
    } else {
        None
    }
}

#[doc(hidden)]
pub const fn hash_string_const(base: u64, v: &str) -> u64 {
    const fn siphalfround(v: &mut [u64; 4], r0: u32, r1: u32) {
        let temp = v[0] ^ v[1];

        v[1] = v[1].rotate_left(r0).wrapping_add(temp);
        v[0] = v[2] ^ v[3];
        v[3] = v[3].rotate_left(r1).wrapping_add(v[0]);
        v[2] = temp.rotate_left(32);
    }

    const fn sipround(v: &mut [u64; 4]) {
        siphalfround(v, 13, 16);
        siphalfround(v, 17, 21);
    }

    let st = v.as_bytes();

    let mut i = 0;

    let mut state = [
        0x736f6d6570736575 ^ base,
        0x646f72616e646f6d,
        0x6c7967656e657261 ^ base,
        0x7465646279746573,
    ];

    while i + 7 < st.len() {
        let m = u64::from_le_bytes([
            st[i],
            st[i + 1],
            st[i + 2],
            st[i + 3],
            st[i + 4],
            st[i + 5],
            st[i + 6],
            st[i + 7],
        ]);
        state[3] ^= m;

        sipround(&mut state);
        sipround(&mut state);
        state[0] ^= m;

        i += 8;
    }

    let mut buf = [0xFFu8; 8];

    let mut j = 0;
    while i + j < st.len() {
        buf[j] = st[i + j];
        j += 1;
    }
    let m = u64::from_le_bytes(buf);
    state[3] ^= m;
    sipround(&mut state);
    sipround(&mut state);
    state[0] ^= m;
    state[2] ^= 0xFF;
    sipround(&mut state);
    sipround(&mut state);
    sipround(&mut state);
    sipround(&mut state);

    state[0] ^ state[1] ^ state[2] ^ state[3]
}

pub const trait AsId<T: IdType>: [const] AsRawId {}
pub const unsafe trait IdType: Copy + core::hash::Hash + Eq {
    #[doc(hidden)]
    fn into_raw_parts(self) -> (NonZeroU64, u64);
    #[doc(hidden)]
    fn from_raw_parts(ty: NonZeroU64, discrim: u64) -> Self;

    fn new<T: [const] AsId<Self>>(val: T) -> Self {
        Self::from_raw_parts(T::TYPE, val.into_raw_id())
    }

    fn downcast<T: [const] AsId<Self>>(self) -> Option<T> {
        let (ty, discrim) = self.into_raw_parts();

        if ty.get() == T::TYPE.get() {
            T::from_raw_id(discrim)
        } else {
            None
        }
    }
}

#[doc(hidden)]
pub const fn raw_id_type(x: u64) -> NonZeroU64 {
    match NonZeroU64::new(x) {
        Some(x) => x,
        None => unsafe { NonZeroU64::new_unchecked(!0) },
    }
}

mod macros {
    #[macro_export]
    macro_rules! AsRawId {
        derive() ($(#[$meta:meta])* $vis:vis enum $name:ident {
            $($var_name:ident $(= $discrim:expr)?),*
            $(,)?
        }) => {
            unsafe impl const $crate::traits::AsRawId for $name {
                const TYPE: ::core::num::NonZeroU64 = $crate::traits::raw_id_type($crate::traits::hash_string_const($crate::macros::rand_u64!(enum $name {
                    $($var_name $(= $discrim)?),*
                }), ::core::concat!(::core::module_path!(), "::", ::core::stringify!($name), $("\0", ::core::stringify!($var_name)),*)));

                fn into_raw_id(self) -> u64 {
                    self as u64
                }

                fn from_raw_id(x: u64) -> Option<Self> {
                    $(const $var_name: u64 = $name::$var_name as u64;)*

                    match x {
                        $($var_name => Some(Self::$var_name),)*
                        _ => None
                    }
                }
            }
        };


        derive() ($(#[$meta:meta])* $vis:vis struct $name:ident ($field_ty:ty);) => {
            const _: () {
                const fn __test::<__T: $crate::traits::TryAsU64Raw>() {}
                __test::<$field_ty>();
            };
            unsafe impl const $crate::traits::AsRawId for $name {
                const TYPE: ::core::num::NonZeroU64 = $crate::traits::raw_id_type($crate::traits::hash_string_const($crate::macros::rand_u64!(struct $name ($field_ty);), ::core::module_path!(), "::", ::core::stringify!($name),));

                fn into_raw_id(self) -> u64 {
                    unsafe { $crate::mem::transmute(self.0)}
                }

                fn from_raw_id(x: u64) -> Option<Self> {
                    $crate::traits::try_from_u64::<$field_ty>().map($name)
                }
            }
        };

    }

    pub use AsRawId;

    #[macro_export]
    macro_rules! IdType {
        derive() ($(#[$meta:meta])* $vis:vis struct $name:ident($nz_ty:ty, u64);) => {
            unsafe impl const $crate::traits::IdType for $name {
                fn into_raw_parts(self) -> (::core::num::NonZeroU64, u64) {
                    (self.0, self.1)
                }

                fn from_raw_parts(ty: ::core::num::NonZeroU64, discrim: u64) -> Self {
                    Self(ty, discrim)
                }
            }
        };
    }
    pub use IdType;

    #[macro_export]
    macro_rules! Name {
        derive() ($(#[$meta:meta])* $vis:vis struct $name:ident $($_rest:tt)*) => {
            impl $crate::traits::Name for $name{
                fn name(&self) -> &'static str {
                    ::core::stringify!($name)
                }
            }
        };

        derive() ($(#[$meta:meta])* $vis:vis enum $name:ident {
            $($var_name:ident $( = $discrim:expr)?),*
            $(,)?
        }) => {
            impl $crate::traits::Name for $name{
                fn name(&self) -> &'static str {
                    match self {
                        $(Self::$var_name => ::core::stringify!($var_name)),*
                    }
                }
            }
        };
    }

    pub use Name;
}

pub use macros::*;
