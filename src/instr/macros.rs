#[derive(Copy, Clone)]
#[doc(hidden)]
pub enum __Void {}

impl From<__Void> for ! {
    fn from(value: __Void) -> Self {
        match value {}
    }
}

#[doc(hidden)]
pub use paste::paste as _paste;

#[doc(hidden)]
pub use cmli_derive::compile_error_with_span;

#[macro_export]
#[doc(hidden)]
macro_rules! __unwrap_property {
    ($name:ident, $variant_name:ident, $value:expr) => {
        match $value {
            ::core::result::Result::Ok(val) => val,
            ::core::result::Result::Err(_) => ::core::panic!(::core::concat!(
                "Required property ",
                ::core::stringify!($name),
                " not defined for variant ",
                ::core::stringify!($variant)
            )),
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __def_extra_properties {
    {
        $enum_name:ident {
            $($var_name:ident),* $(,)?
        }
    } => {

    };
    {
        $enum_name:ident [$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_default:expr)? ),* $(,)?] {
            $($var_name:ident $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),* $(,)?
        }
    } => {
        const _: () = {
            struct __StaticPropertiesOrDefault {
                $(
                    #[allow(unused_parens)]
                    $extra_properties_names: ::core::result::Result<$extra_properties_tys, ($($crate::instr::macros::__Void ${ignore($extra_properties_default)})?)>
                ),*
            }

            const __DEFAULT: __StaticPropertiesOrDefault = __StaticPropertiesOrDefault {
                $($extra_properties_names: ($(::core::result::Result::Ok::<$extra_properties_tys, _>($extra_properties_default),)? ::core::result::Result::<$extra_properties_tys, ()>::Err(()),).0),*
            };

            struct __StaticProperties {
                $($extra_properties_names: $extra_properties_tys),*
            }

            macro_rules! __unwrap_properties {
                ($$var_name:ident, $$base:expr) => {
                    const {
                        let __base = $$base;
                        let properties = __StaticProperties {
                            $($extra_properties_names: $crate::__unwrap_property!($extra_properties_names, $$var_name, (__base).$extra_properties_names)),*
                        };
                        properties
                    }
                };
            }

            $crate::instr::macros::_paste!{
                $(
                    #[allow(non_upper_case_globals)]
                    const [<__PROPERTIES___ $var_name>]: __StaticProperties = __unwrap_properties!($var_name, __StaticPropertiesOrDefault {
                        $($($extra_properties_value_names: ::core::result::Result::Ok($extra_properties_value),)*)?
                        ..__DEFAULT
                    });
                )*

                const fn __static_properties(this: &$enum_name) -> __StaticProperties {
                    match *this {
                        $($enum_name:: $var_name => [<__PROPERTIES___ $var_name>]),*
                    }
                }
            }


            impl $enum_name {
                $(
                    pub const fn $extra_properties_names(&self) -> $extra_properties_tys{
                        __static_properties(self)
                            .$extra_properties_names
                    }
                )*
            }
        };
    };
}

#[macro_export]
macro_rules! instr_set {
    {
        $(#[$meta:meta])*
        instructions $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {

        }
    } => {

    };
    {
        $(#[$meta:meta])*
        prefixes $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? : $opcode_ty:ty {
            $($prefix_name:ident ($name:literal) = $opcode:literal  $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),* $(,)?
        }
    } => {
        $(#[$meta])*
        #[derive($crate::instr::PrefixId, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $enum_name {
            $($prefix_name),*
        }

        impl $enum_name {
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self:: $prefix_name => $name),*
                }
            }

            pub fn from_name(name: &'static str) -> Option<Self> {
                match name {
                    $($name => Some(Self:: $prefix_name),)*
                    _ => None
                }
            }

            pub const fn opcode(&self) -> $opcode_ty {
                match self {
                    $(Self:: $prefix_name => $opcode),*
                }
            }
        }

        $crate::__def_extra_properties!{
            $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),* ])? {
                $($prefix_name $(($($extra_properties_value_names : $extra_properties_value),*))?),*
            }
        }
    };
    {
        $(#[$meta:meta])*
        encoding $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {
            $($encoding_name:ident $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),* $(,)?
        }
    } => {
        $(#[$meta])*
        #[derive($crate::instr::EncodingId, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $enum_name {
            $($encoding_name),*
        }

        $crate::__def_extra_properties!{
            $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),* ])? {
                $($encoding_name $(($($extra_properties_value_names : $extra_properties_value),*))?),*
            }
        }

        #[allow(unused_imports)]
        use $enum_name::*;
    };
    {
        $(#[$meta:meta])*
        registers $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? : $regno_ty:ty {
            classes $class_enum_name:ident $([$($class_properties_names:ident : $class_properties_tys:ty $(= $class_properties_defauit:expr)? ),* $(,)?])? {
                $($class_name:ident $(($($class_properties_value_names:ident : $class_properties_value:expr),* $(,)?))? {
                    $($var_name:ident ($regno:literal) $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),+ $(,)?
                }),* $(,)?
            }
        }
    } => {
        $(#[$meta])*
        #[derive($crate::instr::RegisterId, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $enum_name {
            $($(#[allow(non_camel_case_types)] $var_name,)*)*
        }

        impl $enum_name {
            pub const fn name(&self) -> &'static str {
                match *self {
                    $($(Self:: $var_name => ::core::stringify!($var_name),)*)*
                }
            }

            pub const fn class(&self) -> $class_enum_name {
                match *self {
                    $($(Self :: $var_name)|+ => $class_enum_name :: $class_name),*
                }
            }

            pub const fn regno(&self) -> $regno_ty {
                match *self {
                    $($(Self:: $var_name => $regno,)*)*
                }
            }
        }

        $crate::__def_extra_properties! {
            $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),* ])? {
                $($($var_name $(($($extra_properties_value_names : $extra_properties_value),*))?,)*)*
            }
        }

        #[allow(unused_imports)]
        use $enum_name::*;

        $(#[$meta])*
        #[derive($crate::instr::RegisterClassId, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $class_enum_name {
            $($class_name),*
        }

        $crate::__def_extra_properties! {
            $class_enum_name $([$($class_properties_names : $class_properties_tys $(= $class_properties_defauit)? ),* ])? {
                $($class_name $(($($class_properties_value_names : $class_properties_value),*))?),*
            }
        }

        #[allow(unused_imports)]
        use $class_enum_name::*;
    };
    {
        $(#[$meta:meta])*
        features $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {
            $($var_name:ident ($feature_name:literal) $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),* $(,)?
        }
    } => {
        $(#[$meta])*
        #[derive($crate::arch::FeatureId, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $enum_name {
            $($var_name,)*
        }
        #[allow(unused_imports)]
        use $enum_name::*;

        impl $enum_name {
            pub fn feature_name(&self) -> &'static str {
                match self {
                    $(Self:: $var_name => $feature_name),*
                }
            }

            pub fn feature_from_name(name: &'static str) -> ::core::option::Option<Self> {
                match name {
                    $($feature_name => ::core::option::Option::Some(Self:: $var_name),)*
                    _ => ::core::option::Option::None,
                }
            }
        }

        $crate::__def_extra_properties!{
            $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),* ])? {
                $($var_name $(($($extra_properties_value_names : $extra_properties_value),*))?,)*
            }
        }
    };
    (
        $(#[$meta:meta])*
        $kind:ident $enum_name:ident $([$($first_extra_properties_names:ident : $first_extra_properties_tys:ty $(= $first_extra_properties_defauit:expr)? ),* $(,)?])? $(: $first_spec_ty:ty)? {
            $($first_tt:tt)*
        }
    ) => {
        $crate::instr::macros::compile_error_with_span!($kind, ::core::concat!("Unexpected def kind ", ::core::stringify!($kind)));
    };
    {
        $(#[$first_meta:meta])*
        $first_kind:ident $first_enum_name:ident $([$($first_extra_properties_names:ident : $first_extra_properties_tys:ty $(= $first_extra_properties_defauit:expr)? ),* $(,)?])? $(: $first_spec_ty:ty)? {
            $($first_tt:tt)*
        }
        $(
            $(#[$meta:meta])*
            $kind:ident $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? $(: $spec_ty:ty)?  {
                $($tt:tt)*
            }
        )+
    } => {
        $crate::instr_set!{
            $(#[$first_meta])*
            $first_kind $first_enum_name $([$($first_extra_properties_names : $first_extra_properties_tys $(= $first_extra_properties_defauit)? ),*])? $(: $first_spec_ty)?  {
                $($first_tt)*
            }
        }
        $($crate::instr_set!{
            $(#[$meta])*
            $kind $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),*])? $(: $spec_ty)?  {
                $($tt)*
            }
        })*
    };
    {} => {};
}
